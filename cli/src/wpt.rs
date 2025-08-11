use std::path::PathBuf;
use std::process::Command;
use std::collections::HashSet;
use std::io::Write;
use chrono::{TimeZone, Utc};
use serde_json::Value;
use crate::error::{AndromedaError, Result};

pub fn run_wpt_command(
    wpt_path: Option<PathBuf>,
    threads: usize,
    filter: Option<String>,
    skip: Option<String>,
    suite: Option<String>,
    save_results: bool,
    output_dir: Option<PathBuf>,
) -> Result<()> {
    let wpt_dir = wpt_path.unwrap_or_else(|| PathBuf::from("tests/wpt"));

    // Load skip.json to filter out skipped suites
    let skip_path = std::path::PathBuf::from("tests/skip.json");
    let skipped_suites: std::collections::HashSet<String> = if skip_path.exists() {
        match std::fs::read_to_string(&skip_path) {
            Ok(skip_content) => {
                match serde_json::from_str::<Value>(&skip_content) {
                    Ok(skip_data) => {
                        skip_data["skip"]
                            .as_array()
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default()
                    }
                    Err(_) => std::collections::HashSet::new(),
                }
            }
            Err(_) => std::collections::HashSet::new(),
        }
    } else {
        std::collections::HashSet::new()
    };

    // Import and use the WPT runner
    let mut cmd = Command::new("cargo");

    // Get the absolute path to tests/Cargo.toml
    let manifest_path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("tests")
        .join("Cargo.toml");

    cmd.args(&["run", "--bin", "wpt_test_runner", "--manifest-path"]);
    cmd.arg(&manifest_path);
    cmd.args(&["--", "run"]);

    if let Some(suite_name) = suite {
        if suite_name == "all" {
            // Run all suites except those in skip.json
            run_all_suites_except_skipped(&skipped_suites, threads, &wpt_dir, filter, skip, save_results, output_dir)?;
            return Ok(());
        } else {
            // Check if the specified suite is in the skip list
            if skipped_suites.contains(&suite_name) {
                println!("⏭️  Skipping suite '{}' as it's in skip.json", suite_name);
                println!("✅ All tests completed (suite was skipped)");
                return Ok(());
            }
            cmd.arg(suite_name);
        }
    } else {
        // If no suite specified, show available suites (excluding skipped ones) and exit
        show_available_suites_filtered(&skipped_suites)?;
        return Ok(());
    }

    cmd.arg("--threads").arg(threads.to_string());
    cmd.arg("--wpt-dir").arg(&wpt_dir);

    if let Some(filter_pattern) = filter {
        cmd.arg("--filter").arg(filter_pattern);
    }

    if let Some(skip_pattern) = skip {
        cmd.arg("--skip").arg(skip_pattern);
    }

    // Only pass --output if explicitly requested
    if save_results {
        if let Some(output_path) = output_dir {
            cmd.arg("--output").arg(output_path);
        } else {
            cmd.arg("--output").arg("tests/results");
        }
    }
    // Otherwise, don't save results (no --output argument)

    let status = cmd.status().map_err(|e| {
        AndromedaError::runtime_error(
            format!("Failed to run WPT runner: {}", e),
            None,
            None,
            None,
            None,
        )
    })?;

    if !status.success() {
        return Err(AndromedaError::runtime_error(
            "WPT runner failed".to_string(),
            None,
            None,
            None,
            None,
        ));
    }

    Ok(())
}

fn run_all_suites_except_skipped(
    skipped_suites: &HashSet<String>,
    threads: usize,
    wpt_dir: &PathBuf,
    filter: Option<String>,
    skip: Option<String>,
    save_results: bool,
    output_dir: Option<PathBuf>,
) -> Result<()> {
    // Get all available suites from wpt directory
    let mut available_suites = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(wpt_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map_or(false, |t| t.is_dir()) {
                if let Some(name) = entry.file_name().to_str() {
                    if !name.starts_with('.') && !name.starts_with("common") && !skipped_suites.contains(name) {
                        available_suites.push(name.to_string());
                    }
                }
            }
        }
    }
    
    available_suites.sort();
    
    if available_suites.is_empty() {
        println!("No available suites to run (all suites are either skipped or don't exist)");
        return Ok(());
    }
    
    println!("🏃 Running {} suites (excluding {} skipped suites)", 
        available_suites.len(), 
        skipped_suites.len()
    );
    println!("Suites to run: {}", available_suites.join(", "));
    println!("═══════════════════════════════════════════════════════");
    
    let mut overall_success = true;
    let mut suite_results = Vec::new();
    
    for (index, suite_name) in available_suites.iter().enumerate() {
        println!("\n🔄 [{}/{}] Running suite: {}", 
            index + 1, 
            available_suites.len(), 
            suite_name
        );
        println!("{}", "─".repeat(50));
        std::io::stdout().flush().unwrap();
        
        let mut cmd = Command::new("cargo");
        
        // Get the absolute path to tests/Cargo.toml
        let manifest_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("tests")
            .join("Cargo.toml");
        
        cmd.args(&["run", "--bin", "wpt_test_runner", "--manifest-path"]);
        cmd.arg(&manifest_path);
        cmd.args(&["--", "run"]);
        cmd.arg(suite_name);
        cmd.arg("--threads").arg(threads.to_string());
        cmd.arg("--wpt-dir").arg(wpt_dir);
        
        if let Some(ref filter_pattern) = filter {
            cmd.arg("--filter").arg(filter_pattern);
        }
        
        if let Some(ref skip_pattern) = skip {
            cmd.arg("--skip").arg(skip_pattern);
        }
        
        if save_results {
            if let Some(ref output_path) = output_dir {
                cmd.arg("--output").arg(output_path);
            } else {
                cmd.arg("--output").arg("tests/results");
            }
        }
        
        let status = cmd.status().map_err(|e| {
            AndromedaError::runtime_error(
                format!("Failed to run WPT runner for suite {}: {}", suite_name, e),
                None,
                None,
                None,
                None,
            )
        })?;
        
        let suite_successful = status.success();
        suite_results.push((suite_name.clone(), suite_successful));
        
        if suite_successful {
            println!("✅ Suite '{}' completed successfully", suite_name);
        } else {
            println!("❌ Suite '{}' failed", suite_name);
            overall_success = false;
        }
        
        println!("📋 Moving to next suite...");
        std::io::stdout().flush().unwrap();
    }
    
    // Print summary
    println!("\n📊 All Suites Summary");
    println!("══════════════════════════════════════════");
    
    let mut successful_suites = 0;
    let mut failed_suites = 0;
    
    for (suite_name, success) in &suite_results {
        if *success {
            println!("✅ {}", suite_name);
            successful_suites += 1;
        } else {
            println!("❌ {}", suite_name);
            failed_suites += 1;
        }
    }
    
    println!("\n📈 Final Results:");
    println!("   ✅ Successful: {}", successful_suites);
    println!("   ❌ Failed: {}", failed_suites);
    println!("   ⏭️ Skipped: {}", skipped_suites.len());
    println!("   📦 Total Available: {}", available_suites.len() + skipped_suites.len());
    
    if overall_success {
        println!("\n🎉 All suites completed successfully!");
        Ok(())
    } else {
        Err(AndromedaError::runtime_error(
            format!("{} suite(s) failed out of {}", failed_suites, available_suites.len()),
            None,
            None,
            None,
            None,
        ))
    }
}


fn show_available_suites_filtered(skipped_suites: &std::collections::HashSet<String>) -> Result<()> {
    println!("No suite specified. Available WPT suites:");

    // List available suites from wpt directory
    let wpt_dir_path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("tests")
        .join("wpt");

    if let Ok(entries) = std::fs::read_dir(&wpt_dir_path) {
        let mut available_suites = Vec::new();
        let mut skipped_count = 0;
        
        for entry in entries.flatten() {
            if entry.file_type().map_or(false, |t| t.is_dir()) {
                if let Some(name) = entry.file_name().to_str() {
                    if !name.starts_with('.') && !name.starts_with("common") {
                        if skipped_suites.contains(name) {
                            skipped_count += 1;
                        } else {
                            available_suites.push(name.to_string());
                        }
                    }
                }
            }
        }
        available_suites.sort();

        for suite in available_suites.iter().take(20) {
            println!("  {}", suite);
        }
        if available_suites.len() > 20 {
            println!("  ... and {} more", available_suites.len() - 20);
        }
        
        if skipped_count > 0 {
            println!("\n⏭️  {} suite(s) are skipped (see tests/skip.json)", skipped_count);
        }
    }

    println!("\nUsage: andromeda wpt --suite <suite_name>");
    println!("       andromeda wpt --suite all  # Run all suites except those in skip.json");
    println!("Example: andromeda wpt --suite console");
    Ok(())
}

pub fn display_metrics(detailed: bool, trends: bool) -> Result<()> {
    use std::fs;

    let metrics_path = std::path::PathBuf::from("metrics.json");

    if !metrics_path.exists() {
        println!("📊 No metrics available yet. Run 'andromeda wpt' to generate metrics.");
        return Ok(());
    }

    let content = fs::read_to_string(&metrics_path).map_err(|e| {
        AndromedaError::runtime_error(
            format!("Failed to read metrics.json: {}", e),
            None,
            None,
            None,
            None,
        )
    })?;

    let metrics: Value = serde_json::from_str(&content).map_err(|e| {
        AndromedaError::runtime_error(
            format!("Failed to parse metrics.json: {}", e),
            None,
            None,
            None,
            None,
        )
    })?;

    display_metrics_overview(&metrics)?;

    if trends {
        display_trend_information(&metrics)?;
    }

    if detailed {
        display_suite_details(&metrics)?;
    }

    display_last_run_info(&metrics)?;
    Ok(())
}

fn display_metrics_overview(metrics: &Value) -> Result<()> {
    println!("📊 Andromeda Runtime Metrics");
    println!("═══════════════════════════════");

    if let Some(last_updated) = metrics["last_updated"].as_u64() {
        let dt = Utc
            .timestamp_opt(last_updated as i64, 0)
            .single()
            .unwrap_or_else(Utc::now);
        println!("📅 Last Updated: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
    }

    println!(
        "📦 Version: {}",
        metrics["version"].as_str().unwrap_or("unknown")
    );
    println!();

    // WPT Overall Results with Visual Indicators
    let wpt_overall = &metrics["wpt"]["overall"];
    println!("🌐 WPT Overall Results");
    println!("─────────────────────────");

    let total_tests = wpt_overall["total_tests"].as_u64().unwrap_or(0);
    let pass = wpt_overall["pass"].as_u64().unwrap_or(0);
    let fail = wpt_overall["fail"].as_u64().unwrap_or(0);
    let crash = wpt_overall["crash"].as_u64().unwrap_or(0);
    let timeout = wpt_overall["timeout"].as_u64().unwrap_or(0);
    let skip = wpt_overall["skip"].as_u64().unwrap_or(0);
    let pass_rate = wpt_overall["pass_rate"].as_f64().unwrap_or(0.0);
    let duration = wpt_overall["duration_seconds"].as_f64().unwrap_or(0.0);

    // Visual progress bar for overall status
    let status_icon = if pass_rate >= 90.0 {
        "🟢"
    } else if pass_rate >= 70.0 {
        "🟡"
    } else {
        "🔴"
    };

    let progress_width = 30;
    let filled = ((pass_rate / 100.0) * progress_width as f64) as usize;
    let empty = progress_width - filled;
    let progress_bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

    println!("{} Overall Status: {:.1}%", status_icon, pass_rate);
    println!("   {}", progress_bar);
    println!();

    println!("📈 Total Tests: {}", total_tests);
    println!("✅ Pass: {} ({:.1}%)", pass, pass_rate);
    println!("❌ Fail: {}", fail);
    println!("💥 Crash: {}", crash);
    println!("⏱️  Timeout: {}", timeout);
    println!("⏭️  Skip: {}", skip);
    println!("⚡ Duration: {:.2}s", duration);
    println!();
    
    // Show failed tests if any
    let total_failures = fail + crash + timeout;
    if total_failures > 0 {
        display_failed_tests_summary(metrics)?;
    }

    Ok(())
}

fn display_trend_information(metrics: &Value) -> Result<()> {
    let trend = &metrics["wpt"]["trend"];
    println!("📊 Trends");
    println!("─────────");

    let pass_rate_change = trend["pass_rate_change"].as_f64().unwrap_or(0.0);
    let tests_change = trend["tests_change"].as_i64().unwrap_or(0);
    let performance_change = trend["performance_change"].as_f64().unwrap_or(0.0);

    println!("📈 Pass Rate Change: {:.1}%", pass_rate_change);
    println!("📊 Test Count Change: {}", tests_change);
    println!("⚡ Performance Change: {:.2}s", performance_change);
    println!();

    Ok(())
}

fn display_suite_details(metrics: &Value) -> Result<()> {
    println!("🎯 Suite Details");
    println!("─────────────────");

    if let Some(suites) = metrics["wpt"]["suites"].as_object() {
        for (suite_name, suite_data) in suites {
            let suite_total = suite_data["total_tests"].as_u64().unwrap_or(0);
            let suite_pass = suite_data["pass"].as_u64().unwrap_or(0);
            let suite_pass_rate = suite_data["pass_rate"].as_f64().unwrap_or(0.0);
            let suite_duration = suite_data["duration_seconds"].as_f64().unwrap_or(0.0);

            if suite_total > 0 {
                println!(
                    "  📂 {}: {}/{} ({:.1}%) - {:.2}s",
                    suite_name, suite_pass, suite_total, suite_pass_rate, suite_duration
                );
            }
        }
    }
    println!();

    Ok(())
}

fn display_last_run_info(metrics: &Value) -> Result<()> {
    let wpt_overall = &metrics["wpt"]["overall"];
    
    if let Some(last_run) = wpt_overall["last_run"].as_u64() {
        let dt = Utc
            .timestamp_opt(last_run as i64, 0)
            .single()
            .unwrap_or_else(Utc::now);
        println!("🕐 Last WPT Run: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
    } else {
        println!("🕐 Last WPT Run: Never");
    }

    Ok(())
}

fn display_failed_tests_summary(_metrics: &Value) -> Result<()> {
    use std::fs;
    use std::path::PathBuf;
    
    // Load skip.json to filter out skipped suites
    let skip_path = PathBuf::from("tests/skip.json");
    let skipped_suites: HashSet<String> = if skip_path.exists() {
        match fs::read_to_string(&skip_path) {
            Ok(skip_content) => {
                match serde_json::from_str::<Value>(&skip_content) {
                    Ok(skip_data) => {
                        skip_data["skip"]
                            .as_array()
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default()
                    }
                    Err(_) => HashSet::new(),
                }
            }
            Err(_) => HashSet::new(),
        }
    } else {
        HashSet::new()
    };
    
    // Try to load the expectation.json file for expected failures
    let expectation_path = PathBuf::from("tests/expectation.json");
    
    if expectation_path.exists() {
        let content = fs::read_to_string(&expectation_path).map_err(|e| {
            AndromedaError::runtime_error(
                format!("Failed to read expectation.json: {}", e),
                None,
                None,
                None,
                None,
            )
        })?;
        
        let expectations: Value = serde_json::from_str(&content).map_err(|e| {
            AndromedaError::runtime_error(
                format!("Failed to parse expectation.json: {}", e),
                None,
                None,
                None,
                None,
            )
        })?;
        
        if let Some(suites) = expectations["suites"].as_object() {
            println!("\n❌ Expected Test Failures (from expectation.json)");
            println!("══════════════════════════════════════════════");
            
            let mut total_failures = 0;
            let mut by_type: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            
            for (suite_name, suite_data) in suites {
                // Skip if this suite is in skip.json
                if skipped_suites.contains(suite_name) {
                    continue;
                }
                
                if let Some(expectations) = suite_data["expectations"].as_object() {
                    if !expectations.is_empty() {
                        println!("\n  📂 {} Suite ({} expected failures):", suite_name, expectations.len());
                        
                        // Group by expectation type
                        let mut suite_by_type: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
                        for (test_name, expectation) in expectations {
                            let exp_type = expectation.as_str().unwrap_or("UNKNOWN");
                            suite_by_type.entry(exp_type.to_string())
                                .or_insert_with(Vec::new)
                                .push(test_name.clone());
                            *by_type.entry(exp_type.to_string()).or_insert(0) += 1;
                            total_failures += 1;
                        }
                        
                        // Display by type
                        for (exp_type, tests) in &suite_by_type {
                            let icon = match exp_type.as_str() {
                                "CRASH" => "💥",
                                "FAIL" => "❌",
                                "TIMEOUT" => "⏱️",
                                "SKIP" => "⏭️",
                                _ => "❓",
                            };
                            
                            println!("    {} {} ({} tests):", icon, exp_type, tests.len());
                            for test in tests.iter().take(3) {
                                println!("      • {}", test);
                            }
                            if tests.len() > 3 {
                                println!("      ... and {} more", tests.len() - 3);
                            }
                        }
                    }
                }
            }
            
            // Also display stats if available
            for (suite_name, suite_data) in suites {
                // Skip if this suite is in skip.json
                if skipped_suites.contains(suite_name) {
                    continue;
                }
                
                if let Some(stats) = suite_data["stats"].as_object() {
                    let total = stats["total"].as_u64().unwrap_or(0);
                    let crash = stats["crash"].as_u64().unwrap_or(0);
                    let fail = stats["fail"].as_u64().unwrap_or(0);
                    let timeout = stats["timeout"].as_u64().unwrap_or(0);
                    
                    if total > 0 && (crash > 0 || fail > 0 || timeout > 0) {
                        if crash == total {
                            println!("\n  ⚠️  {} Suite: All {} tests are currently crashing", suite_name, total);
                        } else if crash + fail + timeout > 0 {
                            println!("\n  📊 {} Suite Statistics:", suite_name);
                            if crash > 0 { println!("    💥 Crash: {}/{}", crash, total); }
                            if fail > 0 { println!("    ❌ Fail: {}/{}", fail, total); }
                            if timeout > 0 { println!("    ⏱️ Timeout: {}/{}", timeout, total); }
                        }
                    }
                }
            }
            
            if total_failures > 0 {
                println!("\n  📊 Overall Summary:");
                println!("  ─────────────────");
                
                // Display summary from expectation.json if available
                if let Some(summary) = expectations["summary"].as_object() {
                    let total_tests = summary["total_tests"].as_u64().unwrap_or(0);
                    let total_failures = summary["total_failures"].as_u64().unwrap_or(0);
                    
                    println!("  Total Tests: {}", total_tests);
                    println!("  Total Failures: {} ({:.1}%)", 
                        total_failures, 
                        if total_tests > 0 { (total_failures as f64 / total_tests as f64) * 100.0 } else { 0.0 }
                    );
                    
                    if let Some(breakdown) = summary["breakdown"].as_object() {
                        println!("\n  Breakdown by Type:");
                        for (exp_type, count) in breakdown {
                            if let Some(count_val) = count.as_u64() {
                                if count_val > 0 {
                                    let icon = match exp_type.as_str() {
                                        "crash" => "💥",
                                        "fail" => "❌",
                                        "timeout" => "⏱️",
                                        "skip" => "⏭️",
                                        _ => "❓",
                                    };
                                    println!("  {} {}: {}", icon, exp_type.to_uppercase(), count_val);
                                }
                            }
                        }
                    }
                } else {
                    // Fallback to simple count
                    println!("  Total Expected Failures: {}", total_failures);
                    for (exp_type, count) in &by_type {
                        let icon = match exp_type.as_str() {
                            "CRASH" => "💥",
                            "FAIL" => "❌",
                            "TIMEOUT" => "⏱️",
                            "SKIP" => "⏭️",
                            _ => "❓",
                        };
                        println!("  {} {}: {}", icon, exp_type, count);
                    }
                }
            }
        }
        
        // Display last updated time if available
        if let Some(last_updated) = expectations["last_updated"].as_u64() {
            let dt = chrono::Utc
                .timestamp_opt(last_updated as i64, 0)
                .single()
                .unwrap_or_else(chrono::Utc::now);
            println!("\n  📅 Expectations Last Updated: {}", dt.format("%Y-%m-%d %H:%M:%S UTC"));
        }
    } else {
        println!("\n❌ No expectation.json file found");
        println!("   Run tests to generate expectations for failed tests");
    }
    
    Ok(())
}