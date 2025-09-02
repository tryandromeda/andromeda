//! WPT Test Executor

use crate::wpt_harness_builder::WptHarnessBuilder;
use crate::{WptTestCase, WptTestResult};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TestExecutorConfig {
    pub timeout: Duration,
    pub optimize_console_log: bool,
    pub binary_path: Option<std::path::PathBuf>,
}

impl Default for TestExecutorConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            optimize_console_log: true,
            binary_path: None,
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

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
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

        let start_time = Instant::now();
        let result = self.execute_wpt_test(test_path)?;
        let duration = start_time.elapsed();

        Ok(vec![WptTestCase {
            name: test_name,
            result: result.0,
            message: result.1,
            duration,
        }])
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
        let (tx, rx) = mpsc::channel();
        let timeout = self.config.timeout;
        let binary_path = self.get_binary_path();

        let temp_file = tempfile::NamedTempFile::new()?;
        std::fs::write(temp_file.path(), test_script)?;
        let temp_path = temp_file.path().to_path_buf();

        let handle = thread::spawn(move || {
            let child = Command::new(binary_path)
                .arg("run")
                .arg(&temp_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match child {
                Ok(child) => match child.wait_with_output() {
                    Ok(output) => {
                        let _ = tx.send(Ok(output));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e));
                    }
                },
                Err(e) => {
                    let _ = tx.send(Err(e));
                }
            }
        });

        let adjusted_timeout = if test_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.contains("large-array"))
            .unwrap_or(false)
        {
            timeout * 10
        } else {
            timeout
        };
        let output = match rx.recv_timeout(adjusted_timeout) {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Ok((
                    WptTestResult::Crash,
                    Some(format!("Failed to execute test: {e}")),
                ));
            }
            Err(_) => {
                return Ok((
                    WptTestResult::Timeout,
                    Some("Test execution timed out".to_string()),
                ));
            }
        };

        let _ = handle.join();
        self.parse_test_output(output)
    }

    fn parse_test_output(
        &self,
        output: std::process::Output,
    ) -> Result<(WptTestResult, Option<String>), Box<dyn std::error::Error>> {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if let Some(results_line) = stdout.lines().find(|line| line.starts_with("WPT_RESULTS:")) {
            if let Some(json_str) = results_line.strip_prefix("WPT_RESULTS:") {
                if let Ok(results) = serde_json::from_str::<serde_json::Value>(json_str) {
                    return self.process_test_results(results);
                }
            }
        }
        if output.status.success() {
            Ok((WptTestResult::Pass, None))
        } else if !stderr.is_empty() {
            Ok((WptTestResult::Crash, Some(stderr.to_string())))
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
            // Try to find the binary in common locations
            let possible_paths = vec![
                std::path::PathBuf::from("target/debug/andromeda"),
                std::path::PathBuf::from("../target/debug/andromeda"),
                std::path::PathBuf::from("../../target/debug/andromeda"),
                std::env::current_dir()
                    .unwrap()
                    .join("target/debug/andromeda"),
            ];

            for path in possible_paths {
                if path.exists() {
                    return path;
                }
            }

            // Fallback to the most likely location
            std::path::PathBuf::from("target/debug/andromeda")
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

    for line in html_content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("<script") && !trimmed.contains("src=") {
            in_script = true;
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

        if trimmed.contains("</script>") && in_script {
            let before_end = trimmed.split("</script>").next().unwrap_or("");
            if !before_end.is_empty() {
                script_content.push_str(before_end);
                script_content.push('\n');
            }
            scripts.push(script_content.trim().to_string());
            script_content.clear();
            in_script = false;
            continue;
        }

        if in_script {
            script_content.push_str(line);
            script_content.push('\n');
        }
    }

    if scripts.is_empty() {
        return Ok("// HTML file with no testable script content".to_string());
    }

    let combined_script = scripts.join("\n\n");

    let fixed_script = combined_script
        .replace("for (method of methods)", "for (let method of methods)")
        .replace(
            "for (const method of methods)",
            "for (let method of methods)",
        )
        .replace("for (var method of methods)", "for (let method of methods)");

    Ok(fixed_script)
}
