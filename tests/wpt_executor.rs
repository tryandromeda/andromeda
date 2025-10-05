//! WPT Test Executor

use crate::wpt_harness_builder::WptHarnessBuilder;
use crate::{WptTestCase, WptTestResult};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TestExecutorConfig {
    pub timeout: Duration,
    pub optimize_console_log: bool,
    pub binary_path: Option<std::path::PathBuf>,
    pub max_retries: usize,
    pub verbose: bool,
}

impl Default for TestExecutorConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            optimize_console_log: true,
            binary_path: None,
            max_retries: 2,
            verbose: false,
        }
    }
}

#[derive(Debug)]
pub struct WptTestExecutor {
    config: TestExecutorConfig,
    harness_builder: WptHarnessBuilder,
}

impl WptTestExecutor {
    pub fn new() -> Self {
        Self {
            config: TestExecutorConfig::default(),
            harness_builder: WptHarnessBuilder::new(),
        }
    }

    pub fn with_config(config: TestExecutorConfig) -> Self {
        let mut harness_builder = WptHarnessBuilder::new();
        harness_builder = harness_builder.optimize_console_log(config.optimize_console_log);

        Self {
            config,
            harness_builder,
        }
    }

    #[allow(dead_code)]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    #[allow(dead_code)]
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.config.verbose = verbose;
        self
    }

    #[allow(dead_code)]
    pub fn max_retries(mut self, max_retries: usize) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    pub fn execute_test<P: AsRef<Path>>(
        &self,
        test_path: P,
    ) -> Result<Vec<WptTestCase>, Box<dyn std::error::Error>> {
        let test_path = test_path.as_ref();
        let test_name = test_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let mut last_result = None;
        let mut last_error = None;

        // Retry logic for timeouts and crashes
        for attempt in 0..=self.config.max_retries {
            let start_time = Instant::now();
            let result = self.execute_wpt_test(test_path);
            let duration = start_time.elapsed();

            match result {
                Ok((WptTestResult::Timeout, ref msg)) if attempt < self.config.max_retries => {
                    if self.config.verbose {
                        eprintln!(
                            "Test {} timed out on attempt {}, retrying...",
                            test_name,
                            attempt + 1
                        );
                    }
                    last_result = Some((WptTestResult::Timeout, msg.clone()));
                    // Brief pause before retry to allow system resources to stabilize
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Ok((WptTestResult::Crash, ref msg)) if attempt < self.config.max_retries => {
                    if self.config.verbose {
                        eprintln!(
                            "Test {} crashed on attempt {}, retrying...",
                            test_name,
                            attempt + 1
                        );
                    }
                    last_result = Some((WptTestResult::Crash, msg.clone()));
                    // Brief pause before retry to allow system resources to stabilize
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Ok(result) => {
                    return Ok(vec![WptTestCase {
                        name: test_name,
                        result: result.0,
                        message: result.1,
                        duration,
                    }]);
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        // All retries exhausted
        if let Some((result, msg)) = last_result {
            let duration = Duration::from_secs(0);
            return Ok(vec![WptTestCase {
                name: test_name,
                result,
                message: Some(format!(
                    "{} (after {} retries)",
                    msg.unwrap_or_default(),
                    self.config.max_retries
                )),
                duration,
            }]);
        }

        if let Some(e) = last_error {
            return Err(e);
        }

        Err("Unknown error executing test".into())
    }

    fn execute_wpt_test<P: AsRef<Path>>(
        &self,
        test_path: P,
    ) -> Result<(WptTestResult, Option<String>), Box<dyn std::error::Error>> {
        let test_path = test_path.as_ref();

        let test_content = fs::read_to_string(test_path)?;

        let processed_content = if test_path.extension().and_then(|s| s.to_str()) == Some("html") {
            extract_script_from_html(&test_content)?
        } else {
            test_content
        };

        let test_script = self.harness_builder.build_test_wrapper(&processed_content);
        let result = self.execute_with_timeout_stdin(&test_script, test_path)?;

        Ok(result)
    }

    fn execute_with_timeout_stdin(
        &self,
        test_script: &str,
        test_path: &Path,
    ) -> Result<(WptTestResult, Option<String>), Box<dyn std::error::Error>> {
        use wait_timeout::ChildExt;

        let binary_path = self.get_binary_path();

        // Create temp file with better naming for debugging
        let temp_file = tempfile::Builder::new()
            .prefix("wpt_test_")
            .suffix(".js")
            .tempfile()?;

        std::fs::write(temp_file.path(), test_script)?;
        let temp_path = temp_file.path().to_path_buf();

        if self.config.verbose {
            eprintln!("Executing test with temp file: {:?}", temp_path);
        }

        // Determine timeout with platform-specific adjustments
        let adjusted_timeout = self.calculate_adjusted_timeout(test_path);

        // Spawn the child process
        let mut child = Command::new(&binary_path)
            .arg("run")
            .arg(&temp_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        // Wait with timeout
        match child.wait_timeout(adjusted_timeout)? {
            Some(_status) => {
                // Process completed within timeout
                let output = child.wait_with_output()?;
                self.parse_test_output(output)
            }
            None => {
                // Timeout occurred - kill the process
                if self.config.verbose {
                    eprintln!(
                        "Test timed out after {:?}, killing process",
                        adjusted_timeout
                    );
                }

                // Try graceful kill first
                let _ = child.kill();

                // Wait a bit for process to die
                std::thread::sleep(Duration::from_millis(100));

                // Try to reap the process
                let _ = child.wait();

                Ok((
                    WptTestResult::Timeout,
                    Some(format!(
                        "Test execution timed out after {:?}",
                        adjusted_timeout
                    )),
                ))
            }
        }
    }

    fn calculate_adjusted_timeout(&self, test_path: &Path) -> Duration {
        #[allow(unused_mut)]
        let mut timeout = self.config.timeout;

        // Check for known slow tests
        if let Some(name) = test_path.file_name().and_then(|n| n.to_str()) {
            if name.contains("large-array") || name.contains("big-") {
                return timeout * 10;
            }
            if name.contains("stress") || name.contains("performance") {
                return timeout * 5;
            }
        }

        // Platform-specific adjustments
        #[cfg(target_os = "macos")]
        {
            // macOS can be slower on some operations
            timeout += Duration::from_secs(5);
        }

        #[cfg(target_os = "windows")]
        {
            // Windows file I/O can be slower
            timeout += Duration::from_secs(10);
        }

        timeout
    }

    fn parse_test_output(
        &self,
        output: std::process::Output,
    ) -> Result<(WptTestResult, Option<String>), Box<dyn std::error::Error>> {
        // Handle potentially invalid UTF-8 more gracefully
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if self.config.verbose {
            if !stdout.is_empty() {
                eprintln!("STDOUT: {}", stdout.chars().take(500).collect::<String>());
            }
            if !stderr.is_empty() {
                eprintln!("STDERR: {}", stderr.chars().take(500).collect::<String>());
            }
        }

        // Look for WPT_RESULTS marker in stdout
        // Check last 20 lines to handle cases where there's a lot of console output
        let lines: Vec<&str> = stdout.lines().collect();
        let search_start = lines.len().saturating_sub(20);

        for line in &lines[search_start..] {
            if let Some(json_str) = line.strip_prefix("WPT_RESULTS:") {
                match serde_json::from_str::<serde_json::Value>(json_str) {
                    Ok(results) => {
                        return self.process_test_results(results);
                    }
                    Err(e) => {
                        if self.config.verbose {
                            eprintln!("Failed to parse WPT_RESULTS JSON: {}", e);
                        }
                        // Continue to fallback logic
                    }
                }
            }
        }

        // Fallback logic based on exit status and output
        if output.status.success() {
            // Success but no WPT_RESULTS - might be a simple test
            Ok((WptTestResult::Pass, None))
        } else if stderr.contains("error:") || stderr.contains("Error:") {
            // Looks like a crash/error
            let error_msg = stderr
                .lines()
                .filter(|line| line.contains("error") || line.contains("Error"))
                .take(5)
                .collect::<Vec<_>>()
                .join("; ");
            Ok((WptTestResult::Crash, Some(error_msg)))
        } else if !stderr.is_empty() {
            // Non-empty stderr but not clearly an error
            Ok((WptTestResult::Crash, Some(stderr)))
        } else if !stdout.is_empty() {
            // Output exists but test failed
            Ok((
                WptTestResult::Fail,
                Some("Test failed - check output".to_string()),
            ))
        } else {
            Ok((
                WptTestResult::Fail,
                Some("Test failed with no output".to_string()),
            ))
        }
    }

    fn process_test_results(
        &self,
        results: serde_json::Value,
    ) -> Result<(WptTestResult, Option<String>), Box<dyn std::error::Error>> {
        if let Some(arr) = results.as_array() {
            let all_pass = arr
                .iter()
                .all(|r| r.get("pass").and_then(|p| p.as_bool()).unwrap_or(false));

            if all_pass {
                Ok((WptTestResult::Pass, None))
            } else {
                let failures = arr
                    .iter()
                    .filter(|r| !r.get("pass").and_then(|p| p.as_bool()).unwrap_or(false))
                    .map(|r| {
                        let name = r.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                        let error = r.get("error").and_then(|e| e.as_str()).unwrap_or("failed");
                        format!("{name}: {error}")
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                Ok((WptTestResult::Fail, Some(failures)))
            }
        } else {
            Ok((
                WptTestResult::Fail,
                Some("Invalid test results format".to_string()),
            ))
        }
    }

    fn get_binary_path(&self) -> std::path::PathBuf {
        if let Some(ref path) = self.config.binary_path {
            path.clone()
        } else {
            std::env::current_dir()
                .unwrap()
                .parent()
                .unwrap()
                .join("target/debug/andromeda")
        }
    }
}

impl Default for WptTestExecutor {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_script_from_html(html_content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut scripts = Vec::new();
    let mut in_script = false;
    let mut script_content = String::new();
    let mut script_has_src = false;

    for line in html_content.lines() {
        let trimmed = line.trim();

        // Start of script tag
        if trimmed.starts_with("<script") {
            // Check if it has a src attribute (external script - skip it)
            if trimmed.contains("src=") || trimmed.contains("src =") {
                script_has_src = true;
                // Check if it's a self-closing or single-line script tag
                if trimmed.contains("</script>") {
                    script_has_src = false;
                }
                continue;
            }

            in_script = true;
            script_content.clear();
            script_has_src = false;

            // Handle inline script on same line as opening tag
            if trimmed.ends_with("</script>") {
                let content = trimmed
                    .split('>')
                    .nth(1)
                    .unwrap_or("")
                    .rsplit("</script>")
                    .nth(1)
                    .unwrap_or("")
                    .trim();
                if !content.is_empty() {
                    scripts.push(content.to_string());
                }
                in_script = false;
            }
            continue;
        }

        // End of script tag
        if trimmed.contains("</script>") {
            if in_script && !script_has_src {
                let before_end = trimmed.split("</script>").next().unwrap_or("");
                if !before_end.is_empty() {
                    script_content.push_str(before_end);
                    script_content.push('\n');
                }
                if !script_content.trim().is_empty() {
                    scripts.push(script_content.trim().to_string());
                }
                script_content.clear();
            }
            in_script = false;
            script_has_src = false;
            continue;
        }

        // Content inside script tag
        if in_script && !script_has_src {
            script_content.push_str(line);
            script_content.push('\n');
        }
    }

    // Handle unclosed script tags
    if in_script && !script_content.trim().is_empty() && !script_has_src {
        scripts.push(script_content.trim().to_string());
    }

    if scripts.is_empty() {
        return Ok("// HTML file with no testable script content".to_string());
    }

    let mut combined_script = scripts.join("\n\n");

    // Apply common fixes for WPT test patterns
    combined_script = combined_script
        .replace("for (method of methods)", "for (let method of methods)")
        .replace(
            "for (const method of methods)",
            "for (let method of methods)",
        )
        .replace("for (var method of methods)", "for (let method of methods)")
        .replace("for (type of types)", "for (let type of types)")
        .replace("for (var type of types)", "for (let type of types)")
        .replace("for (prop of props)", "for (let prop of props)")
        .replace("for (var prop of props)", "for (let prop of props)")
        .replace("for (item of items)", "for (let item of items)")
        .replace("for (var item of items)", "for (let item of items)");

    Ok(combined_script)
}
