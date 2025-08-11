use clap::{Args as ClapArgs, Parser as ClapParser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{PathBuf, Path},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use walkdir::WalkDir;

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
    pub last_updated: Option<u64>,
    pub description: String,
    pub suites: HashMap<String, SuiteExpectations>,
    pub global_expectations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteExpectations {
    pub expectations: HashMap<String, String>,
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
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
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
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
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
    #[serde(serialize_with = "serialize_duration", deserialize_with = "deserialize_duration")]
    pub total_duration: Duration,
}

// Helper functions for Duration serialization
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
    pub last_updated: Option<u64>,
    pub wpt: WptMetrics,
    pub build: BuildMetrics,
    pub runtime: RuntimeMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WptMetrics {
    pub overall: WptSuiteMetrics,
    pub suites: HashMap<String, WptSuiteMetrics>,
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
    pub last_run: Option<u64>,
    pub duration_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendMetrics {
    pub pass_rate_change: f64,
    pub tests_change: i32,
    pub performance_change: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetrics {
    pub last_successful_build: Option<u64>,
    pub build_count: u32,
    pub failed_builds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMetrics {
    pub startup_time_ms: Option<f64>,
    pub memory_usage_mb: Option<f64>,
    pub gc_collections: Option<u32>,
}

#[derive(Debug)]
pub struct WptRunner {
    wpt_path: PathBuf,
    results: Arc<Mutex<HashMap<String, WptSuiteResult>>>,
    threads: usize,
    filter: Option<String>,
    skip: Option<String>,
    timeout: Duration,
    output_dir: Option<PathBuf>,
    save_results: bool,
}

impl WptRunner {
    pub fn new(wpt_path: impl AsRef<Path>) -> Self {
        Self {
            wpt_path: wpt_path.as_ref().to_path_buf(),
            results: Arc::new(Mutex::new(HashMap::new())),
            threads: 4,
            filter: None,
            skip: None,
            timeout: Duration::from_secs(30),
            output_dir: None,
            save_results: false,
        }
    }

    pub fn with_threads(mut self, threads: usize) -> Self {
        self.threads = threads;
        self
    }

    pub fn with_filter(mut self, filter: String) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn with_skip(mut self, skip: String) -> Self {
        self.skip = Some(skip);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_output_dir(mut self, output_dir: impl AsRef<Path>) -> Self {
        self.output_dir = Some(output_dir.as_ref().to_path_buf());
        self.save_results = true;
        self
    }

    pub fn with_save_results(mut self, save: bool) -> Self {
        self.save_results = save;
        // Don't set a default output_dir, require it to be explicitly specified
        self
    }

    pub fn run_suite(&self, suite_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let suite_path = self.wpt_path.join(suite_name);
        
        if !suite_path.exists() {
            return Err(format!("Suite {} not found at {}", suite_name, suite_path.display()).into());
        }

        println!("Running WPT suite: {}", suite_name);
        std::io::stdout().flush().unwrap();
        let start_time = Instant::now();
        
        let mut test_files = Vec::new();
        
        println!("Scanning directory: {}", suite_path.display());
        std::io::stdout().flush().unwrap();
        
        for entry in WalkDir::new(&suite_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                let extension = path.extension();
                
                // Include various WPT test types
                if let Some(ext) = extension {
                    if ext == "html" || ext == "htm" || 
                       ext == "js" || ext == "any.js" || ext == "window.js" || ext == "worker.js" {
                        
                        let relative_path = path.strip_prefix(&self.wpt_path)?;
                        let test_name = relative_path.to_string_lossy().to_string();
                        
                        // Apply filtering
                        if let Some(ref filter) = self.filter {
                            if !test_name.contains(filter) {
                                continue;
                            }
                        }
                        
                        if let Some(ref skip) = self.skip {
                            if test_name.contains(skip) {
                                continue;
                            }
                        }
                        
                        test_files.push(path.to_path_buf());
                    }
                }
            }
        }

        println!("Found {} test files", test_files.len());
        std::io::stdout().flush().unwrap();
        
        let mut suite_result = WptSuiteResult {
            name: suite_name.to_string(),
            tests: Vec::new(),
            pass_count: 0,
            fail_count: 0,
            crash_count: 0,
            timeout_count: 0,
            skip_count: 0,
            total_duration: Duration::new(0, 0),
        };

        for test_file in &test_files {
            match self.run_test(test_file) {
                Ok(test_cases) => {
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
                Err(e) => {
                    eprintln!("Error running test {}: {}", test_file.display(), e);
                    let test_case = WptTestCase {
                        name: test_file.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        result: WptTestResult::Crash,
                        message: Some(e.to_string()),
                        duration: Duration::new(0, 0),
                    };
                    suite_result.crash_count += 1;
                    suite_result.tests.push(test_case);
                }
            }
        }

        let total_time = start_time.elapsed();
        suite_result.total_duration = total_time;

        self.results.lock().unwrap().insert(suite_name.to_string(), suite_result.clone());
        self.print_suite_result(&suite_result);
        
        // Save metrics and expectations after each suite
        // Individual result files are only saved if output_dir is specified
        if let Err(e) = self.save_run_results() {
            eprintln!("Warning: Failed to save results: {}", e);
        }
        
        Ok(())
    }

    fn run_test(&self, test_path: &Path) -> Result<Vec<WptTestCase>, Box<dyn std::error::Error>> {
        let test_name = test_path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let start_time = Instant::now();
        
        // For now, just mark all tests as "not implemented" to demonstrate the framework
        // In a real implementation, this would execute the test properly
        let result = WptTestResult::Crash;
        let message = Some("Test execution not yet implemented".to_string());
        let duration = start_time.elapsed();

        println!("    Running test: {}", test_name);
        std::io::stdout().flush().unwrap();

        Ok(vec![WptTestCase {
            name: test_name,
            result,
            message,
            duration,
        }])
    }

    fn print_suite_result(&self, result: &WptSuiteResult) {
        let total = result.tests.len();
        let pass_rate = if total > 0 { 
            (result.pass_count as f64 / total as f64) * 100.0 
        } else { 
            0.0 
        };

        println!("\n=== WPT Suite Results: {} ===", result.name);
        println!("Total tests: {}", total);
        println!("Pass: {} ({:.1}%)", result.pass_count, pass_rate);
        println!("Fail: {} ({:.1}%)", result.fail_count, (result.fail_count as f64 / total as f64) * 100.0);
        println!("Crash: {} ({:.1}%)", result.crash_count, (result.crash_count as f64 / total as f64) * 100.0);
        println!("Timeout: {} ({:.1}%)", result.timeout_count, (result.timeout_count as f64 / total as f64) * 100.0);
        println!("Skip: {} ({:.1}%)", result.skip_count, (result.skip_count as f64 / total as f64) * 100.0);
        println!("Total time: {:.2}s", result.total_duration.as_secs_f64());
        
        // Show detailed failure information
        let failures: Vec<_> = result.tests.iter()
            .filter(|t| matches!(t.result, WptTestResult::Fail | WptTestResult::Crash | WptTestResult::Timeout))
            .collect();
            
        if !failures.is_empty() {
            println!("\n❌ Failed Tests ({} total):", failures.len());
            println!("─────────────────────────");
            
            // Group failures by result type
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
            
            // Display failures by category
            if !failed_tests.is_empty() {
                println!("\n  Failed ({}):", failed_tests.len());
                for test in failed_tests.iter().take(5) {
                    println!("    • {}", test.name);
                    if let Some(ref msg) = test.message {
                        println!("      └─ {}", msg);
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
                        println!("      └─ {}", msg);
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
                        println!("      └─ {}", msg);
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

        // Explicitly scope the lock to avoid deadlock with save_run_results()
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

                let pass_rate = if suite_result.tests.len() > 0 {
                    (suite_result.pass_count as f64 / suite_result.tests.len() as f64) * 100.0
                } else {
                    0.0
                };
                
                println!("  {}: {}/{} ({:.1}%)", 
                    suite_name, 
                    suite_result.pass_count, 
                    suite_result.tests.len(), 
                    pass_rate
                );
            }
        } // Lock is released here before calling save_run_results()

        let overall_pass_rate = if total_tests > 0 {
            (total_pass as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        println!("\nOverall Summary:");
        println!("  Total: {}", total_tests);
        println!("  Pass: {} ({:.1}%)", total_pass, overall_pass_rate);
        println!("  Fail: {} ({:.1}%)", total_fail, (total_fail as f64 / total_tests as f64) * 100.0);
        println!("  Crash: {} ({:.1}%)", total_crash, (total_crash as f64 / total_tests as f64) * 100.0);
        println!("  Timeout: {} ({:.1}%)", total_timeout, (total_timeout as f64 / total_tests as f64) * 100.0);
        println!("  Skip: {} ({:.1}%)", total_skip, (total_skip as f64 / total_tests as f64) * 100.0);
        println!("  Total time: {:.2}s", total_duration.as_secs_f64());

        // Save final results (metrics and expectations always, individual files only if output_dir specified)
        println!("💾 Saving results...");
        std::io::stdout().flush().unwrap();
        self.save_run_results()?;
        println!("✅ Results saved!");
        std::io::stdout().flush().unwrap();
        
        Ok(())
    }

    fn save_run_results(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 Starting save_run_results...");
        std::io::stdout().flush().unwrap();
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        
        let mut total_tests = 0;
        let mut total_pass = 0;
        let mut total_fail = 0;
        let mut total_crash = 0;
        let mut total_timeout = 0;
        let mut total_skip = 0;
        let mut total_duration = Duration::new(0, 0);
        
        let mut suites = HashMap::new();
        
        // Explicitly scope the lock to ensure it's released early
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
        } // Lock is explicitly released here
        
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
        
        // Always update metrics and expectations (not in results directory)
        self.update_metrics(&run_result)?;
        self.update_expectations(&run_result)?;
        
        Ok(())
    }
    
    fn update_expectations(&self, run_result: &WptRunResult) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 Starting update_expectations...");
        std::io::stdout().flush().unwrap();
        let expectation_path = PathBuf::from("tests/expectation.json");
        
        // Load skip.json to filter out skipped suites
        let skip_path = PathBuf::from("tests/skip.json");
        let skipped_suites: Vec<String> = if skip_path.exists() {
            let skip_file = File::open(&skip_path)?;
            let skip_data: serde_json::Value = serde_json::from_reader(skip_file)?;
            skip_data["skip"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        
        let mut expectations = if expectation_path.exists() {
            let file = File::open(&expectation_path)?;
            serde_json::from_reader(file).unwrap_or_else(|_| ExpectationFile {
                version: "1.0.0".to_string(),
                last_updated: None,
                description: "WPT test expectations for Andromeda Runtime".to_string(),
                suites: HashMap::new(),
                global_expectations: HashMap::new(),
            })
        } else {
            ExpectationFile {
                version: "1.0.0".to_string(),
                last_updated: None,
                description: "WPT test expectations for Andromeda Runtime".to_string(),
                suites: HashMap::new(),
                global_expectations: HashMap::new(),
            }
        };
        
        // Update expectations based on test results, excluding skipped suites
        // Don't clear existing expectations - merge them instead
        for (suite_name, suite_result) in &run_result.suites {
            // Skip this suite if it's in the skip list
            if skipped_suites.contains(suite_name) {
                continue;
            }
            
            let suite_expectations = expectations.suites.entry(suite_name.clone())
                .or_insert(SuiteExpectations {
                    expectations: HashMap::new(),
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
                
                // Only record non-passing tests
                if test.result != WptTestResult::Pass {
                    suite_expectations.expectations.insert(test.name.clone(), expectation.to_string());
                }
            }
        }
        
        expectations.last_updated = Some(run_result.timestamp);
        
        println!("📝 Creating expectations file...");
        std::io::stdout().flush().unwrap();
        let file = File::create(expectation_path)?;
        println!("📄 Writing expectations to file...");
        std::io::stdout().flush().unwrap();
        serde_json::to_writer_pretty(file, &expectations)?;
        println!("💾 Expectations file saved successfully!");
        std::io::stdout().flush().unwrap();
        
        Ok(())
    }
    
    fn update_metrics(&self, run_result: &WptRunResult) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Starting update_metrics...");
        std::io::stdout().flush().unwrap();
        // Always save metrics to tests/metrics.json, not in results directory
        let metrics_file = PathBuf::from("tests/metrics.json");
        
        // Load skip.json to filter out skipped suites
        let skip_path = PathBuf::from("tests/skip.json");
        let skipped_suites: Vec<String> = if skip_path.exists() {
            let skip_file = File::open(&skip_path)?;
            let skip_data: serde_json::Value = serde_json::from_reader(skip_file)?;
            skip_data["skip"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
            
        let mut metrics = if metrics_file.exists() {
            let file = File::open(&metrics_file)?;
            serde_json::from_reader(file).unwrap_or_else(|_| Metrics::default())
        } else {
            Metrics::default()
        };
            
        // Merge suite metrics - add new results to existing ones
        for (suite_name, suite_result) in &run_result.suites {
            // Skip this suite if it's in the skip list
            if skipped_suites.contains(suite_name) {
                continue;
            }
            
            let pass_rate = if suite_result.tests.len() > 0 {
                (suite_result.pass_count as f64 / suite_result.tests.len() as f64) * 100.0
            } else {
                0.0
            };
            
            // Insert or update suite metrics
            metrics.wpt.suites.insert(suite_name.clone(), WptSuiteMetrics {
                total_tests: suite_result.tests.len(),
                pass: suite_result.pass_count,
                fail: suite_result.fail_count,
                crash: suite_result.crash_count,
                timeout: suite_result.timeout_count,
                skip: suite_result.skip_count,
                pass_rate,
                last_run: Some(run_result.timestamp),
                duration_seconds: suite_result.total_duration.as_secs_f64(),
            });
        }
        
        // Calculate overall metrics from all current suite metrics
        let mut total_tests = 0;
        let mut total_pass = 0;
        let mut total_fail = 0;
        let mut total_crash = 0;
        let mut total_timeout = 0;
        let mut total_skip = 0;
        let mut total_duration = 0.0;
        
        for suite_metrics in metrics.wpt.suites.values() {
            total_tests += suite_metrics.total_tests;
            total_pass += suite_metrics.pass;
            total_fail += suite_metrics.fail;
            total_crash += suite_metrics.crash;
            total_timeout += suite_metrics.timeout;
            total_skip += suite_metrics.skip;
            total_duration += suite_metrics.duration_seconds;
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
            last_run: Some(run_result.timestamp),
            duration_seconds: total_duration,
            };
            
        metrics.last_updated = Some(run_result.timestamp);
        
        println!("📝 Creating metrics file...");
        std::io::stdout().flush().unwrap();
        let file = File::create(metrics_file)?;
        println!("📄 Writing metrics to file...");
        std::io::stdout().flush().unwrap();
        serde_json::to_writer_pretty(file, &metrics)?;
        println!("💾 Metrics file saved successfully!");
        std::io::stdout().flush().unwrap();
        
        Ok(())
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            project: "andromeda".to_string(),
            version: "0.1.0".to_string(),
            last_updated: None,
            wpt: WptMetrics {
                overall: WptSuiteMetrics::default(),
                suites: HashMap::new(),
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
            last_run: None,
            duration_seconds: 0.0,
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

impl Default for BuildMetrics {
    fn default() -> Self {
        Self {
            last_successful_build: None,
            build_count: 0,
            failed_builds: 0,
        }
    }
}

impl Default for RuntimeMetrics {
    fn default() -> Self {
        Self {
            startup_time_ms: None,
            memory_usage_mb: None,
            gc_collections: None,
        }
    }
}

fn is_test_file(file_name: &str) -> bool {
    // Include HTML files, JavaScript files, and worker files
    (file_name.ends_with(".html") || file_name.ends_with(".htm") ||
     file_name.ends_with(".js") || file_name.ends_with(".any.js") ||
     file_name.ends_with(".window.js") || file_name.ends_with(".worker.js")) &&
    // Exclude helper files and references
    !file_name.contains("-ref.") && !file_name.contains("_FIXTURE")
}

fn count_test_files(path: PathBuf) -> usize {
    WalkDir::new(&path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry.file_name()
                .to_str()
                .map_or(false, is_test_file)
        })
        .count()
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
}

#[derive(ClapArgs, Debug)]
struct RunArgs {
    /// WPT suite(s) to run (e.g., fetch, dom, streams)
    #[arg()]
    suites: Vec<String>,
    
    /// Filter tests by pattern
    #[arg(long)]
    filter: Option<String>,
    
    /// Skip tests matching pattern
    #[arg(long)]
    skip: Option<String>,
    
    /// Number of parallel threads
    #[arg(short, long, default_value = "4")]
    threads: usize,
    
    /// Test timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,
    
    /// Output directory for results
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    
    /// Path to WPT directory
    #[arg(long, default_value = "./wpt")]
    wpt_dir: PathBuf,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(ClapArgs, Debug)]
struct ReportArgs {
    /// Results directory
    #[arg(short, long, default_value = "./results")]
    results_dir: PathBuf,
    
    /// Show detailed failures
    #[arg(long)]
    detailed: bool,
}

use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = WptArgs::parse();
    
    match args.command {
        Commands::Run(run_args) => {
            let mut runner = WptRunner::new(&run_args.wpt_dir)
                .with_threads(run_args.threads)
                .with_timeout(Duration::from_secs(run_args.timeout));
                
            if let Some(filter) = run_args.filter {
                runner = runner.with_filter(filter);
            }
            
            if let Some(skip) = run_args.skip {
                runner = runner.with_skip(skip);
            }
            
            if let Some(output) = run_args.output {
                runner = runner.with_output_dir(output);
            }
            // Only save results if output directory is explicitly specified
            
            // Load skip.json to filter out skipped suites
            let skip_path = PathBuf::from("tests/skip.json");
            let skipped_suites: Vec<String> = if skip_path.exists() {
            let skip_file = File::open(&skip_path)?;
            let skip_data: serde_json::Value = serde_json::from_reader(skip_file)?;
            skip_data["skip"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
            
            // Check if "all" is specified to run all suites
            let suites_to_run = if run_args.suites.len() == 1 && run_args.suites[0] == "all" {
                // Get all available suites, excluding skipped ones
                let mut all_suites = Vec::new();
                if let Ok(entries) = std::fs::read_dir(&run_args.wpt_dir) {
                    for entry in entries.flatten() {
                        if entry.file_type().map_or(false, |t| t.is_dir()) {
                            if let Some(name) = entry.file_name().to_str() {
                                if !name.starts_with('.') && !name.starts_with("common") && !skipped_suites.contains(&name.to_string()) {
                                    all_suites.push(name.to_string());
                                }
                            }
                        }
                    }
                }
                all_suites.sort();
                println!("Running {} WPT suites (excluding {} skipped)...", all_suites.len(), skipped_suites.len());
                all_suites
            } else if run_args.suites.is_empty() {
                println!("No suites specified. Available suites:");
                if let Ok(entries) = std::fs::read_dir(&run_args.wpt_dir) {
                    for entry in entries.flatten() {
                        if entry.file_type().map_or(false, |t| t.is_dir()) {
                            if let Some(name) = entry.file_name().to_str() {
                                if !name.starts_with('.') {
                                    let count = count_test_files(entry.path());
                                    println!("  {} ({} tests)", name, count);
                                }
                            }
                        }
                    }
                }
                return Ok(());
            } else {
                // Filter out skipped suites from the provided list
                run_args.suites.clone()
                    .into_iter()
                    .filter(|s| {
                        if skipped_suites.contains(s) {
                            println!("Skipping suite '{}' as it's in skip.json", s);
                            false
                        } else {
                            true
                        }
                    })
                    .collect()
            };
            
            // Run the selected suites
            for suite in &suites_to_run {
                println!("\nRunning suite: {}", suite);
                runner.run_suite(suite)?;
            }
            runner.print_summary()?;
            println!("🎉 WPT run completed!");
            std::io::stdout().flush().unwrap();
        }
        
        Commands::Report(report_args) => {
            // Generate report from metrics
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
            println!("Fail: {} ({:.1}%)", overall.fail, 
                if overall.total_tests > 0 { (overall.fail as f64 / overall.total_tests as f64) * 100.0 } else { 0.0 });
            println!("Crash: {} ({:.1}%)", overall.crash,
                if overall.total_tests > 0 { (overall.crash as f64 / overall.total_tests as f64) * 100.0 } else { 0.0 });
            println!("Timeout: {} ({:.1}%)", overall.timeout,
                if overall.total_tests > 0 { (overall.timeout as f64 / overall.total_tests as f64) * 100.0 } else { 0.0 });
            println!("Skip: {} ({:.1}%)", overall.skip,
                if overall.total_tests > 0 { (overall.skip as f64 / overall.total_tests as f64) * 100.0 } else { 0.0 });
            
            if report_args.detailed {
                println!("\nSuite breakdown:");
                for (suite_name, suite_metrics) in &metrics.wpt.suites {
                    println!("  {}: {}/{} ({:.1}%)", 
                        suite_name, 
                        suite_metrics.pass, 
                        suite_metrics.total_tests, 
                        suite_metrics.pass_rate
                    );
                    
                    // Show failure counts
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
    }
    
    Ok(())
}