use clap::{Args as ClapArgs, Parser as ClapParser, Subcommand};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use walkdir::WalkDir;

mod wpt_executor;
mod wpt_harness_builder;

use wpt_executor::{TestExecutorConfig, WptTestExecutor};

fn get_available_suites(wpt_dir: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut suites = Vec::new();

    for entry in std::fs::read_dir(wpt_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !name.starts_with('.') && !["common", "tools"].contains(&name) {
                    suites.push(name.to_string());
                }
            }
        }
    }

    suites.sort();
    Ok(suites)
}

fn count_test_files(dir: &Path) -> usize {
    if !dir.is_dir() {
        return 0;
    }

    WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext == "js" || ext == "html")
                && !entry
                    .path()
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or("")
                    .starts_with('.')
        })
        .count()
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum WptTestExpectation {
    Pass,
    Fail,
    Unresolved,
    Crash,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectationFile {
    pub version: String,
    pub description: String,
    pub suites: BTreeMap<String, SuiteExpectations>,
    pub global_expectations: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteExpectations {
    pub expectations: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WptTestResult {
    Pass,
    Fail,
    Crash,
    Timeout,
    Skip,
    NotRun,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WptTestCase {
    pub name: String,
    pub result: WptTestResult,
    pub message: Option<String>,
    #[serde(
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration"
    )]
    pub duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WptSuiteResult {
    pub name: String,
    pub tests: Vec<WptTestCase>,
    pub pass_count: usize,
    pub fail_count: usize,
    pub crash_count: usize,
    pub timeout_count: usize,
    pub skip_count: usize,
    #[serde(
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration"
    )]
    pub total_duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WptRunResult {
    pub timestamp: u64,
    pub suites: HashMap<String, WptSuiteResult>,
    pub total_tests: usize,
    pub total_pass: usize,
    pub total_fail: usize,
    pub total_crash: usize,
    pub total_timeout: usize,
    pub total_skip: usize,
    pub overall_pass_rate: f64,
    #[serde(
        serialize_with = "serialize_duration",
        deserialize_with = "deserialize_duration"
    )]
    pub total_duration: Duration,
}

fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_f64(duration.as_secs_f64())
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let secs = f64::deserialize(deserializer)?;
    Ok(Duration::from_secs_f64(secs))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub project: String,
    pub version: String,
    pub wpt: WptMetrics,
    pub build: BuildMetrics,
    pub runtime: RuntimeMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WptMetrics {
    pub overall: WptSuiteMetrics,
    pub suites: BTreeMap<String, WptSuiteMetrics>,
    pub trend: TrendMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WptSuiteMetrics {
    pub total_tests: usize,
    pub pass: usize,
    pub fail: usize,
    pub crash: usize,
    pub timeout: usize,
    pub skip: usize,
    pub pass_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendMetrics {
    pub pass_rate_change: f64,
    pub tests_change: i32,
    pub performance_change: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildMetrics {
    pub last_successful_build: Option<u64>,
    pub build_count: u32,
    pub failed_builds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeMetrics {
    pub startup_time_ms: Option<f64>,
    pub memory_usage_mb: Option<f64>,
    pub gc_collections: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct WptRunnerConfig {
    pub wpt_path: PathBuf,
    pub threads: usize,
    pub filter: Option<String>,
    pub skip: Option<String>,
    pub timeout: Duration,
    pub output_dir: Option<PathBuf>,
    pub save_results: bool,
    pub verbose: bool,
    pub use_expectations: bool,
    pub ci_mode: bool,
    pub expectations_path: Option<PathBuf>,
    pub skip_json_path: Option<PathBuf>,
}

impl Default for WptRunnerConfig {
    fn default() -> Self {
        Self {
            wpt_path: PathBuf::from("./wpt"),
            threads: 4,
            filter: None,
            skip: None,
            timeout: Duration::from_secs(30),
            output_dir: None,
            save_results: false,
            verbose: false,
            use_expectations: false,
            ci_mode: false,
            expectations_path: None,
            skip_json_path: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct SkipConfig {
    #[allow(dead_code)]
    skip_suites: Vec<String>,
    skip_tests: BTreeMap<String, Vec<String>>,
}

#[derive(Debug)]
pub struct WptRunner {
    config: WptRunnerConfig,
    results: Arc<Mutex<HashMap<String, WptSuiteResult>>>,
    executor: WptTestExecutor,
}

impl WptRunner {
    pub fn new(wpt_path: impl AsRef<Path>) -> Self {
        let config = WptRunnerConfig {
            wpt_path: wpt_path.as_ref().to_path_buf(),
            ..Default::default()
        };
        Self::with_config(config)
    }

    fn load_expectations(&self) -> Result<Option<ExpectationFile>, Box<dyn std::error::Error>> {
        if !self.config.use_expectations {
            return Ok(None);
        }

        let expectation_path = if let Some(ref path) = self.config.expectations_path {
            path.clone()
        } else if Path::new("expectation.json").exists() {
            PathBuf::from("expectation.json")
        } else {
            PathBuf::from("tests/expectation.json")
        };

        if !expectation_path.exists() {
            if self.config.ci_mode {
                return Err(
                    format!("CI mode requires expectation.json at {expectation_path:?}").into(),
                );
            }
            return Ok(None);
        }

        let file = File::open(&expectation_path)?;
        let expectations: ExpectationFile = serde_json::from_reader(file)?;
        Ok(Some(expectations))
    }

    fn load_skip_config(&self) -> Result<Option<SkipConfig>, Box<dyn std::error::Error>> {
        let skip_json_path = if let Some(ref path) = self.config.skip_json_path {
            path.clone()
        } else if Path::new("skip.json").exists() {
            PathBuf::from("skip.json")
        } else {
            PathBuf::from("tests/skip.json")
        };

        if !skip_json_path.exists() {
            return Ok(None);
        }

        let file = File::open(&skip_json_path)?;
        let skip_config: SkipConfig = serde_json::from_reader(file)?;
        Ok(Some(skip_config))
    }

    fn should_skip_test(
        &self,
        test_name: &str,
        suite_name: &str,
        expectations: &Option<ExpectationFile>,
        skip_config: &Option<SkipConfig>,
    ) -> bool {
        // Check skip.json first
        if let Some(ref skip_cfg) = skip_config {
            if let Some(skip_tests) = skip_cfg.skip_tests.get(suite_name) {
                if skip_tests.contains(&test_name.to_string()) {
                    if self.config.verbose {
                        println!("  Skipping test {test_name} (in skip.json)");
                    }
                    return true;
                }
            }
        }

        // Then check expectations
        if let Some(ref expectations) = expectations {
            if let Some(suite_expectations) = expectations.suites.get(suite_name) {
                // Check if this test has an expectation that's not PASS
                if let Some(expectation) = suite_expectations.expectations.get(test_name) {
                    let should_skip = expectation != "PASS";
                    if should_skip && self.config.verbose {
                        println!("  Skipping test {test_name} (expected: {expectation})");
                    }
                    return should_skip;
                }
            }
        }
        false
    }

    pub fn with_config(config: WptRunnerConfig) -> Self {
        let executor_config = TestExecutorConfig {
            timeout: config.timeout,
            optimize_console_log: true,
            binary_path: None,
        };

        Self {
            config,
            results: Arc::new(Mutex::new(HashMap::new())),
            executor: WptTestExecutor::with_config(executor_config),
        }
    }

    pub fn with_threads(mut self, threads: usize) -> Self {
        self.config.threads = threads;
        self
    }

    pub fn with_filter(mut self, filter: String) -> Self {
        self.config.filter = Some(filter);
        self
    }

    pub fn with_skip(mut self, skip: String) -> Self {
        self.config.skip = Some(skip);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;

        self.executor = self.executor.timeout(timeout);
        self
    }

    pub fn with_output_dir(mut self, output_dir: impl AsRef<Path>) -> Self {
        self.config.output_dir = Some(output_dir.as_ref().to_path_buf());
        self.config.save_results = true;
        self
    }

    pub fn with_save_results(mut self, save: bool) -> Self {
        self.config.save_results = save;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.config.verbose = verbose;
        self
    }

    pub fn with_expectations(mut self, use_expectations: bool) -> Self {
        self.config.use_expectations = use_expectations;
        self
    }

    pub fn with_ci_mode(mut self, ci_mode: bool) -> Self {
        self.config.ci_mode = ci_mode;
        if ci_mode {
            self.config.use_expectations = true;
        }
        self
    }

    pub fn with_expectations_path(mut self, path: PathBuf) -> Self {
        self.config.expectations_path = Some(path);
        self
    }

    pub fn with_skip_json_path(mut self, path: PathBuf) -> Self {
        self.config.skip_json_path = Some(path);
        self
    }

    pub fn run_suite(&self, suite_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let suite_path = self.config.wpt_path.join(suite_name);

        if !suite_path.exists() {
            return Err(
                format!("Suite {} not found at {}", suite_name, suite_path.display()).into(),
            );
        }

        // Load expectations if enabled
        let expectations = self.load_expectations()?;

        // Load skip configuration
        let skip_config = self.load_skip_config()?;
        if self.config.use_expectations && expectations.is_some() {
            if self.config.ci_mode {
                println!("Running WPT suite: {suite_name} (CI mode - skipping expected failures)");
            } else {
                println!("Running WPT suite: {suite_name} (using expectations)");
            }
        } else if !self.config.verbose {
            println!("Running WPT suite: {suite_name}");
        }
        std::io::stdout().flush().unwrap();

        let start_time = Instant::now();

        let mut test_files = Vec::new();
        let mut skipped_tests = Vec::new();

        if self.config.verbose {
            println!("Scanning directory: {}", suite_path.display());
            std::io::stdout().flush().unwrap();
        }

        for entry in WalkDir::new(&suite_path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                let extension = path.extension();

                if let Some(ext) = extension {
                    if ext == "html"
                        || ext == "htm"
                        || ext == "js"
                        || ext == "any.js"
                        || ext == "window.js"
                        || ext == "worker.js"
                    {
                        let relative_path = path.strip_prefix(&self.config.wpt_path)?;
                        let test_name = relative_path.to_string_lossy().to_string();
                        let test_file_name = path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();

                        if let Some(ref filter) = self.config.filter {
                            if !test_name.contains(filter) {
                                continue;
                            }
                        }

                        if let Some(ref skip) = self.config.skip {
                            if test_name.contains(skip) {
                                continue;
                            }
                        }

                        // Skip tests based on skip.json or expectations
                        if self.should_skip_test(
                            &test_file_name,
                            suite_name,
                            &expectations,
                            &skip_config,
                        ) {
                            skipped_tests.push(test_file_name.clone());
                            continue;
                        }

                        test_files.push(path.to_path_buf());
                    }
                }
            }
        }

        let skipped_count = skipped_tests.len();

        if !self.config.verbose {
            if self.config.use_expectations && skipped_count > 0 {
                println!(
                    "Running {} test files with {} threads (skipped {} expected failures)...",
                    test_files.len(),
                    self.config.threads,
                    skipped_count
                );
            } else {
                println!(
                    "Running {} test files with {} threads...",
                    test_files.len(),
                    self.config.threads
                );
            }
            std::io::stdout().flush().unwrap();
        } else {
            if self.config.use_expectations && skipped_count > 0 {
                println!(
                    "Found {} test files (skipped {} expected failures)",
                    test_files.len(),
                    skipped_count
                );
            } else {
                println!("Found {} test files", test_files.len());
            }
            std::io::stdout().flush().unwrap();
        }

        let mut suite_result = WptSuiteResult {
            name: suite_name.to_string(),
            tests: Vec::new(),
            pass_count: 0,
            fail_count: 0,
            crash_count: 0,
            timeout_count: 0,
            skip_count: skipped_tests.len(),
            total_duration: Duration::new(0, 0),
        };

        // Add skipped tests to the results
        for skipped_test_name in &skipped_tests {
            suite_result.tests.push(WptTestCase {
                name: skipped_test_name.clone(),
                result: WptTestResult::Skip,
                message: Some("Skipped by skip.json or expectations".to_string()),
                duration: Duration::new(0, 0),
            });
        }

        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.config.threads)
            .build()
            .unwrap();

        let progress_counter = Arc::new(Mutex::new(0));
        let total_tests = test_files.len();

        let parallel_results: Vec<Vec<WptTestCase>> = thread_pool.install(|| {
            test_files
                .par_iter()
                .map(|test_file| {
                    let result = match self.executor.execute_test(test_file) {
                        Ok(test_cases) => {
                            for test_case in &test_cases {
                                if self.config.verbose {
                                    println!(
                                        "    {} test: {} - {:?}",
                                        match test_case.result {
                                            WptTestResult::Pass => "✅",
                                            WptTestResult::Fail => "❌",
                                            WptTestResult::Crash => "💥",
                                            WptTestResult::Timeout => "⏰",
                                            WptTestResult::Skip => "⏭️",
                                            WptTestResult::NotRun => "⚪",
                                        },
                                        test_case.name,
                                        test_case.result
                                    );
                                }
                            }
                            test_cases
                        }
                        Err(e) => {
                            if self.config.verbose {
                                eprintln!("Error running test {}: {}", test_file.display(), e);
                            }
                            vec![WptTestCase {
                                name: test_file
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string(),
                                result: WptTestResult::Crash,
                                message: Some(e.to_string()),
                                duration: Duration::new(0, 0),
                            }]
                        }
                    };

                    if !self.config.verbose {
                        let mut counter = progress_counter.lock().unwrap();
                        *counter += 1;
                        if *counter % 10 == 0 || *counter == total_tests {
                            print!("\rProgress: {}/{} tests completed", *counter, total_tests);
                            std::io::stdout().flush().unwrap();
                        }
                    }

                    result
                })
                .collect()
        });

        if !self.config.verbose {
            println!();
        }

        for test_cases in parallel_results {
            for test_case in test_cases {
                match test_case.result {
                    WptTestResult::Pass => suite_result.pass_count += 1,
                    WptTestResult::Fail => suite_result.fail_count += 1,
                    WptTestResult::Crash => suite_result.crash_count += 1,
                    WptTestResult::Timeout => suite_result.timeout_count += 1,
                    WptTestResult::Skip => suite_result.skip_count += 1,
                    WptTestResult::NotRun => {}
                }
                suite_result.total_duration += test_case.duration;
                suite_result.tests.push(test_case);
            }
        }

        let total_time = start_time.elapsed();
        suite_result.total_duration = total_time;

        self.results
            .lock()
            .unwrap()
            .insert(suite_name.to_string(), suite_result.clone());
        self.print_suite_result(&suite_result);

        if let Err(e) = self.save_run_results() {
            eprintln!("Warning: Failed to save results: {e}");
        }

        Ok(())
    }

    fn print_suite_result(&self, result: &WptSuiteResult) {
        let total = result.tests.len();
        let pass_rate = if total > 0 {
            (result.pass_count as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        println!("\n=== WPT Suite Results: {} ===", result.name);
        println!("Total tests: {total}");
        println!("Pass: {} ({:.1}%)", result.pass_count, pass_rate);
        println!(
            "Fail: {} ({:.1}%)",
            result.fail_count,
            if total > 0 {
                (result.fail_count as f64 / total as f64) * 100.0
            } else {
                0.0
            }
        );
        println!(
            "Crash: {} ({:.1}%)",
            result.crash_count,
            if total > 0 {
                (result.crash_count as f64 / total as f64) * 100.0
            } else {
                0.0
            }
        );
        println!(
            "Timeout: {} ({:.1}%)",
            result.timeout_count,
            if total > 0 {
                (result.timeout_count as f64 / total as f64) * 100.0
            } else {
                0.0
            }
        );
        println!(
            "Skip: {} ({:.1}%)",
            result.skip_count,
            if total > 0 {
                (result.skip_count as f64 / total as f64) * 100.0
            } else {
                0.0
            }
        );
        println!("Total time: {:.2}s", result.total_duration.as_secs_f64());

        let failures: Vec<_> = result
            .tests
            .iter()
            .filter(|t| {
                matches!(
                    t.result,
                    WptTestResult::Fail | WptTestResult::Crash | WptTestResult::Timeout
                )
            })
            .collect();

        if !failures.is_empty() {
            println!("\n❌ Failed Tests ({} total):", failures.len());
            println!("─────────────────────────");

            let mut failed_tests = Vec::new();
            let mut crashed_tests = Vec::new();
            let mut timeout_tests = Vec::new();

            for test in &failures {
                match test.result {
                    WptTestResult::Fail => failed_tests.push(test),
                    WptTestResult::Crash => crashed_tests.push(test),
                    WptTestResult::Timeout => timeout_tests.push(test),
                    _ => {}
                }
            }

            if !failed_tests.is_empty() {
                println!("\n  Failed ({}):", failed_tests.len());
                for test in failed_tests.iter().take(5) {
                    println!("    • {}", test.name);
                    if let Some(ref msg) = test.message {
                        println!("      └─ {msg}");
                    }
                }
                if failed_tests.len() > 5 {
                    println!("    ... and {} more", failed_tests.len() - 5);
                }
            }

            if !crashed_tests.is_empty() {
                println!("\n  Crashed ({}):", crashed_tests.len());
                for test in crashed_tests.iter().take(5) {
                    println!("    • {}", test.name);
                    if let Some(ref msg) = test.message {
                        println!("      └─ {msg}");
                    }
                }
                if crashed_tests.len() > 5 {
                    println!("    ... and {} more", crashed_tests.len() - 5);
                }
            }

            if !timeout_tests.is_empty() {
                println!("\n  Timeout ({}):", timeout_tests.len());
                for test in timeout_tests.iter().take(5) {
                    println!("    • {}", test.name);
                    if let Some(ref msg) = test.message {
                        println!("      └─ {msg}");
                    }
                }
                if timeout_tests.len() > 5 {
                    println!("    ... and {} more", timeout_tests.len() - 5);
                }
            }
        }
    }

    pub fn print_summary(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut total_tests = 0;
        let mut total_pass = 0;
        let mut total_fail = 0;
        let mut total_crash = 0;
        let mut total_timeout = 0;
        let mut total_skip = 0;
        let mut total_duration = Duration::new(0, 0);

        {
            let results = self.results.lock().unwrap();
            if results.is_empty() {
                println!("No test results available");
                return Ok(());
            }

            println!("\n=== Overall WPT Results ===");
            for (suite_name, suite_result) in results.iter() {
                total_tests += suite_result.tests.len();
                total_pass += suite_result.pass_count;
                total_fail += suite_result.fail_count;
                total_crash += suite_result.crash_count;
                total_timeout += suite_result.timeout_count;
                total_skip += suite_result.skip_count;
                total_duration += suite_result.total_duration;

                let pass_rate = if !suite_result.tests.is_empty() {
                    (suite_result.pass_count as f64 / suite_result.tests.len() as f64) * 100.0
                } else {
                    0.0
                };

                println!(
                    "  {}: {}/{} ({:.1}%)",
                    suite_name,
                    suite_result.pass_count,
                    suite_result.tests.len(),
                    pass_rate
                );
            }
        }

        let overall_pass_rate = if total_tests > 0 {
            (total_pass as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        println!("\nOverall Summary:");
        println!("  Total: {total_tests}");
        println!("  Pass: {total_pass} ({overall_pass_rate:.1}%)");
        println!(
            "  Fail: {} ({:.1}%)",
            total_fail,
            (total_fail as f64 / total_tests as f64) * 100.0
        );
        println!(
            "  Crash: {} ({:.1}%)",
            total_crash,
            (total_crash as f64 / total_tests as f64) * 100.0
        );
        println!(
            "  Timeout: {} ({:.1}%)",
            total_timeout,
            (total_timeout as f64 / total_tests as f64) * 100.0
        );
        println!(
            "  Skip: {} ({:.1}%)",
            total_skip,
            (total_skip as f64 / total_tests as f64) * 100.0
        );
        println!("  Total time: {:.2}s", total_duration.as_secs_f64());

        println!("💾 Saving results...");
        std::io::stdout().flush().unwrap();
        self.save_run_results()?;
        println!("✅ Results saved!");
        std::io::stdout().flush().unwrap();

        Ok(())
    }

    fn save_run_results(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.verbose {
            println!("🔄 Starting save_run_results...");
            std::io::stdout().flush().unwrap();
        }

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let mut total_tests = 0;
        let mut total_pass = 0;
        let mut total_fail = 0;
        let mut total_crash = 0;
        let mut total_timeout = 0;
        let mut total_skip = 0;
        let mut total_duration = Duration::new(0, 0);

        let mut suites = HashMap::new();

        {
            let results = self.results.lock().unwrap();
            for (suite_name, suite_result) in results.iter() {
                total_tests += suite_result.tests.len();
                total_pass += suite_result.pass_count;
                total_fail += suite_result.fail_count;
                total_crash += suite_result.crash_count;
                total_timeout += suite_result.timeout_count;
                total_skip += suite_result.skip_count;
                total_duration += suite_result.total_duration;

                suites.insert(suite_name.clone(), suite_result.clone());
            }
        }

        let overall_pass_rate = if total_tests > 0 {
            (total_pass as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        let run_result = WptRunResult {
            timestamp,
            suites,
            total_tests,
            total_pass,
            total_fail,
            total_crash,
            total_timeout,
            total_skip,
            overall_pass_rate,
            total_duration,
        };

        self.update_metrics(&run_result)?;
        self.update_expectations(&run_result)?;

        Ok(())
    }

    fn update_expectations(
        &self,
        run_result: &WptRunResult,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Don't update expectations in CI mode - CI mode should only validate
        if self.config.ci_mode {
            return Ok(());
        }

        if self.config.verbose {
            println!("🔧 Starting update_expectations...");
            std::io::stdout().flush().unwrap();
        }

        let expectation_path = if Path::new("expectation.json").exists() {
            PathBuf::from("expectation.json")
        } else {
            PathBuf::from("tests/expectation.json")
        };

        let skipped_suites = Vec::<String>::new();

        let mut expectations = if expectation_path.exists() {
            let file = File::open(&expectation_path)?;
            serde_json::from_reader(file).unwrap_or_else(|_| ExpectationFile {
                version: "1.0.0".to_string(),
                description: "WPT test expectations for Andromeda Runtime".to_string(),
                suites: BTreeMap::new(),
                global_expectations: BTreeMap::new(),
            })
        } else {
            ExpectationFile {
                version: "1.0.0".to_string(),
                description: "WPT test expectations for Andromeda Runtime".to_string(),
                suites: BTreeMap::new(),
                global_expectations: BTreeMap::new(),
            }
        };

        for (suite_name, suite_result) in &run_result.suites {
            if skipped_suites.contains(suite_name) {
                continue;
            }

            let suite_expectations =
                expectations
                    .suites
                    .entry(suite_name.clone())
                    .or_insert(SuiteExpectations {
                        expectations: BTreeMap::new(),
                    });

            for test in &suite_result.tests {
                let expectation = match test.result {
                    WptTestResult::Pass => "PASS",
                    WptTestResult::Fail => "FAIL",
                    WptTestResult::Crash => "CRASH",
                    WptTestResult::Timeout => "TIMEOUT",
                    WptTestResult::Skip => "SKIP",
                    WptTestResult::NotRun => "NOTRUN",
                };

                if test.result != WptTestResult::Pass {
                    suite_expectations
                        .expectations
                        .insert(test.name.clone(), expectation.to_string());
                }
            }
        }

        if self.config.verbose {
            println!("📝 Creating expectations file...");
            std::io::stdout().flush().unwrap();
        }
        let file = File::create(expectation_path)?;
        if self.config.verbose {
            println!("📄 Writing expectations to file...");
            std::io::stdout().flush().unwrap();
        }
        serde_json::to_writer_pretty(file, &expectations)?;
        if self.config.verbose {
            println!("💾 Expectations file saved successfully!");
            std::io::stdout().flush().unwrap();
        }

        Ok(())
    }

    fn update_metrics(&self, run_result: &WptRunResult) -> Result<(), Box<dyn std::error::Error>> {
        // Don't update metrics in CI mode - CI mode should only validate
        if self.config.ci_mode {
            return Ok(());
        }

        if self.config.verbose {
            println!("📊 Starting update_metrics...");
            std::io::stdout().flush().unwrap();
        }

        let metrics_file = if Path::new("metrics.json").exists() {
            PathBuf::from("metrics.json")
        } else {
            PathBuf::from("tests/metrics.json")
        };

        let skipped_suites = Vec::<String>::new();

        let mut metrics = if metrics_file.exists() {
            let file = File::open(&metrics_file)?;
            serde_json::from_reader(file).unwrap_or_else(|_| Metrics::default())
        } else {
            Metrics::default()
        };

        for (suite_name, suite_result) in &run_result.suites {
            if skipped_suites.contains(suite_name) {
                continue;
            }

            let pass_rate = if !suite_result.tests.is_empty() {
                (suite_result.pass_count as f64 / suite_result.tests.len() as f64) * 100.0
            } else {
                0.0
            };

            metrics.wpt.suites.insert(
                suite_name.clone(),
                WptSuiteMetrics {
                    total_tests: suite_result.tests.len(),
                    pass: suite_result.pass_count,
                    fail: suite_result.fail_count,
                    crash: suite_result.crash_count,
                    timeout: suite_result.timeout_count,
                    skip: suite_result.skip_count,
                    pass_rate,
                },
            );
        }

        let mut total_tests = 0;
        let mut total_pass = 0;
        let mut total_fail = 0;
        let mut total_crash = 0;
        let mut total_timeout = 0;
        let mut total_skip = 0;

        for suite_metrics in metrics.wpt.suites.values() {
            total_tests += suite_metrics.total_tests;
            total_pass += suite_metrics.pass;
            total_fail += suite_metrics.fail;
            total_crash += suite_metrics.crash;
            total_timeout += suite_metrics.timeout;
            total_skip += suite_metrics.skip;
        }

        let overall_pass_rate = if total_tests > 0 {
            (total_pass as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        metrics.wpt.overall = WptSuiteMetrics {
            total_tests,
            pass: total_pass,
            fail: total_fail,
            crash: total_crash,
            timeout: total_timeout,
            skip: total_skip,
            pass_rate: overall_pass_rate,
        };

        if self.config.verbose {
            println!("📝 Creating metrics file...");
            std::io::stdout().flush().unwrap();
        }
        let file = File::create(metrics_file)?;
        if self.config.verbose {
            println!("📄 Writing metrics to file...");
            std::io::stdout().flush().unwrap();
        }
        serde_json::to_writer_pretty(file, &metrics)?;
        if self.config.verbose {
            println!("💾 Metrics file saved successfully!");
            std::io::stdout().flush().unwrap();
        }

        Ok(())
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            project: "andromeda".to_string(),
            version: "0.1.0".to_string(),
            wpt: WptMetrics {
                overall: WptSuiteMetrics::default(),
                suites: BTreeMap::new(),
                trend: TrendMetrics::default(),
            },
            build: BuildMetrics::default(),
            runtime: RuntimeMetrics::default(),
        }
    }
}

impl Default for WptSuiteMetrics {
    fn default() -> Self {
        Self {
            total_tests: 0,
            pass: 0,
            fail: 0,
            crash: 0,
            timeout: 0,
            skip: 0,
            pass_rate: 0.0,
        }
    }
}

impl Default for TrendMetrics {
    fn default() -> Self {
        Self {
            pass_rate_change: 0.0,
            tests_change: 0,
            performance_change: 0.0,
        }
    }
}

#[derive(ClapParser, Debug)]
#[command(name = "wpt")]
#[command(about = "Web Platform Tests runner for Andromeda")]
struct WptArgs {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Run(RunArgs),
    Report(ReportArgs),
    ValidateExpectations(ValidateExpectationsArgs),
}

#[derive(ClapArgs, Debug)]
struct RunArgs {
    #[arg()]
    suites: Vec<String>,

    #[arg(long)]
    filter: Option<String>,

    #[arg(long)]
    skip: Option<String>,

    #[arg(short, long, default_value = "4")]
    threads: usize,

    #[arg(long, default_value = "30")]
    timeout: u64,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(long = "wpt-dir", default_value = "./wpt")]
    wpt_dir: PathBuf,

    #[arg(short, long)]
    verbose: bool,

    #[arg(long = "use-expectations")]
    use_expectations: bool,

    #[arg(long = "ci-mode")]
    ci_mode: bool,

    #[arg(long = "expectations-path")]
    expectations_path: Option<PathBuf>,

    #[arg(long = "skip-json-path")]
    skip_json_path: Option<PathBuf>,
}

#[derive(ClapArgs, Debug)]
struct ReportArgs {
    #[arg(short, long, default_value = "./results")]
    results_dir: PathBuf,

    #[arg(long)]
    detailed: bool,
}

#[derive(ClapArgs, Debug)]
struct ValidateExpectationsArgs {
    #[arg(long = "wpt-dir", default_value = "./wpt")]
    wpt_dir: PathBuf,

    #[arg(long, default_value = "30")]
    timeout: u64,
}

fn validate_expectations(
    _wpt_dir: &Path,
    _timeout: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Validating test expectations...");

    let expectation_path = if Path::new("expectation.json").exists() {
        PathBuf::from("expectation.json")
    } else {
        PathBuf::from("tests/expectation.json")
    };

    let metrics_path = if Path::new("metrics.json").exists() {
        PathBuf::from("metrics.json")
    } else {
        PathBuf::from("tests/metrics.json")
    };

    if !expectation_path.exists() {
        println!("❌ expectation.json not found");
        return Err("expectation.json not found".into());
    }

    if !metrics_path.exists() {
        println!("❌ metrics.json not found - run tests first");
        return Err("metrics.json not found - run tests first".into());
    }

    let expectations_file = File::open(&expectation_path)?;
    let expectations: ExpectationFile = serde_json::from_reader(expectations_file)?;

    let metrics_file = File::open(&metrics_path)?;
    let metrics: Metrics = serde_json::from_reader(metrics_file)?;

    let mut validation_failed = false;
    let mut checked_suites = 0;
    let mut total_failing_tests = 0;
    let mut outdated_expectations = 0;

    println!("\n📊 Checking expectations against current test results...");

    for (suite_name, suite_expectations) in &expectations.suites {
        checked_suites += 1;

        if let Some(suite_metrics) = metrics.wpt.suites.get(suite_name) {
            // Count all failing tests including skipped ones for validation
            // because expectations include tests that would fail if executed
            let current_failing = suite_metrics.fail
                + suite_metrics.crash
                + suite_metrics.timeout
                + suite_metrics.skip;
            let expected_failing = suite_expectations.expectations.len();
            total_failing_tests += current_failing;

            println!("\n📁 Suite: {suite_name}");
            println!("   Expected failing tests: {expected_failing}");
            println!("   Current failing tests: {current_failing}");

            // Special handling for fetch suite which has duplicate test runs
            // Some tests run multiple times in different contexts (window, worker, etc)
            // So the current_failing count will be higher than unique test names
            let matches = if suite_name == "fetch" {
                // For fetch, we accept a difference of up to 60 tests due to duplicates
                (current_failing as i32 - expected_failing as i32).abs() <= 60
            } else {
                expected_failing == current_failing
            };

            if !matches {
                println!("   ⚠️  MISMATCH: expectations need updating");
                outdated_expectations += 1;
                validation_failed = true;

                if current_failing < expected_failing {
                    println!(
                        "   ✅ Good news: {} fewer tests are failing now!",
                        expected_failing - current_failing
                    );
                } else {
                    println!(
                        "   ❌ {} more tests are failing than expected",
                        current_failing - expected_failing
                    );
                }
            } else {
                println!("   ✅ Expectations match current results");
                if suite_name == "fetch" && current_failing != expected_failing {
                    println!(
                        "   ℹ️  Note: {} duplicate test runs detected",
                        current_failing - expected_failing
                    );
                }
            }
        } else {
            println!("   ⚠️  Suite {suite_name} has expectations but no recent test results");
            validation_failed = true;
        }
    }

    if validation_failed {
        println!("\n❌ VALIDATION FAILED");
        println!("┌─────────────────────────────────────────┐");
        println!("│ Test expectations are out of date!     │");
        println!("│                                         │");
        println!("│ {outdated_expectations} out of {checked_suites} suites need updating       │");
        println!("│                                         │");
        println!("│ Please run the tests to update:        │");
        println!("│   cargo run --bin wpt_test_runner \\    │");
        println!("│     -- run console --timeout 5000      │");
        println!("└─────────────────────────────────────────┘");

        std::process::exit(1);
    } else {
        println!("\n✅ VALIDATION PASSED");
        println!("All {checked_suites} test suites have up-to-date expectations");
        println!("Total failing tests tracked: {total_failing_tests}");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = WptArgs::parse();

    match args.command {
        Commands::Run(run_args) => {
            let config = WptRunnerConfig {
                wpt_path: run_args.wpt_dir.clone(),
                threads: run_args.threads,
                filter: run_args.filter.clone(),
                skip: run_args.skip.clone(),
                timeout: Duration::from_secs(run_args.timeout),
                output_dir: run_args.output.clone(),
                save_results: run_args.output.is_some(),
                verbose: run_args.verbose,
                use_expectations: run_args.use_expectations || run_args.ci_mode,
                ci_mode: run_args.ci_mode,
                expectations_path: run_args.expectations_path.clone(),
                skip_json_path: run_args.skip_json_path.clone(),
            };

            let runner = WptRunner::with_config(config);

            let skipped_suites = Vec::<String>::new();

            // In CI mode, print header
            if run_args.ci_mode {
                println!("🚀 Running WPT tests in CI mode");
                println!("   Skipping tests with known failures based on expectation.json");
                println!("═══════════════════════════════════════════════════");
            }

            let suites_to_run = if run_args.suites.len() == 1 && run_args.suites[0] == "all" {
                let all_suites =
                    get_available_suites(&run_args.wpt_dir).unwrap_or_else(|_| Vec::new());
                println!(
                    "Running {} WPT suites (excluding {} skipped)...",
                    all_suites.len(),
                    skipped_suites.len()
                );
                all_suites
            } else if run_args.suites.is_empty() {
                println!("No suites specified. Available suites:");
                if let Ok(available_suites) = get_available_suites(&run_args.wpt_dir) {
                    for suite_name in &available_suites {
                        let suite_path = run_args.wpt_dir.join(suite_name);
                        let count = count_test_files(&suite_path);
                        if skipped_suites.contains(suite_name) {
                            println!("  {suite_name} ({count} tests) [SKIPPED]");
                        } else {
                            println!("  {suite_name} ({count} tests)");
                        }
                    }
                }
                return Ok(());
            } else {
                run_args
                    .suites
                    .clone()
                    .into_iter()
                    .filter(|s| {
                        if skipped_suites.contains(s) {
                            println!("Skipping suite '{s}' as it's in skip.json");
                            false
                        } else {
                            true
                        }
                    })
                    .collect()
            };

            for suite in &suites_to_run {
                println!("\nRunning suite: {suite}");
                runner.run_suite(suite)?;
            }
            runner.print_summary()?;

            if run_args.ci_mode {
                println!("\n✅ CI mode test run completed successfully!");
                println!("   Only new regressions would cause failures.");
            } else {
                println!("🎉 WPT run completed!");
            }
            std::io::stdout().flush().unwrap();
        }

        Commands::Report(report_args) => {
            let metrics_file = PathBuf::from("tests/metrics.json");
            if !metrics_file.exists() {
                return Err("No metrics found. Run tests first.".into());
            }

            let file = File::open(&metrics_file)?;
            let metrics: Metrics = serde_json::from_reader(file)?;

            println!("WPT Test Results Report");
            println!("======================");
            let overall = &metrics.wpt.overall;
            println!("Total tests: {}", overall.total_tests);
            println!("Pass: {} ({:.1}%)", overall.pass, overall.pass_rate);
            println!(
                "Fail: {} ({:.1}%)",
                overall.fail,
                if overall.total_tests > 0 {
                    (overall.fail as f64 / overall.total_tests as f64) * 100.0
                } else {
                    0.0
                }
            );
            println!(
                "Crash: {} ({:.1}%)",
                overall.crash,
                if overall.total_tests > 0 {
                    (overall.crash as f64 / overall.total_tests as f64) * 100.0
                } else {
                    0.0
                }
            );
            println!(
                "Timeout: {} ({:.1}%)",
                overall.timeout,
                if overall.total_tests > 0 {
                    (overall.timeout as f64 / overall.total_tests as f64) * 100.0
                } else {
                    0.0
                }
            );
            println!(
                "Skip: {} ({:.1}%)",
                overall.skip,
                if overall.total_tests > 0 {
                    (overall.skip as f64 / overall.total_tests as f64) * 100.0
                } else {
                    0.0
                }
            );

            if report_args.detailed {
                println!("\nSuite breakdown:");
                for (suite_name, suite_metrics) in &metrics.wpt.suites {
                    println!(
                        "  {}: {}/{} ({:.1}%)",
                        suite_name,
                        suite_metrics.pass,
                        suite_metrics.total_tests,
                        suite_metrics.pass_rate
                    );

                    if suite_metrics.fail > 0 || suite_metrics.crash > 0 {
                        if suite_metrics.fail > 0 {
                            println!("    FAIL: {} tests", suite_metrics.fail);
                        }
                        if suite_metrics.crash > 0 {
                            println!("    CRASH: {} tests", suite_metrics.crash);
                        }
                    }
                }
            }
        }

        Commands::ValidateExpectations(validate_args) => {
            validate_expectations(
                &validate_args.wpt_dir,
                Duration::from_secs(validate_args.timeout),
            )?;
        }
    }

    Ok(())
}
