//! WPT Test Harness Builder

#[derive(Debug, Clone)]
pub struct WptHarnessBuilder {
    optimize_console_log: bool,
}

impl Default for WptHarnessBuilder {
    fn default() -> Self {
        Self {
            optimize_console_log: true,
        }
    }
}

impl WptHarnessBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn optimize_console_log(mut self, enable: bool) -> Self {
        self.optimize_console_log = enable;
        self
    }

    pub fn build_test_wrapper(&self, test_content: &str) -> String {
        let mut harness = String::new();

        harness.push_str(&self.build_core_harness());

        if self.optimize_console_log {
            harness.push_str(&self.build_console_optimization());
        }

        harness.push_str("\n");
        harness.push_str(test_content);

        harness.push_str(&self.build_result_output());

        harness
    }

    fn load_js_file(&self, file_path: &str) -> Result<String, std::io::Error> {
        std::fs::read_to_string(file_path)
    }

    fn build_core_harness(&self) -> String {
        self.load_js_file("js/wpt_harness.js")
            .unwrap_or_else(|_| include_str!("js/wpt_harness.js").to_string())
    }

    fn build_console_optimization(&self) -> String {
        self.load_js_file("js/console_optimization.js")
            .unwrap_or_else(|_| include_str!("js/console_optimization.js").to_string())
    }

    fn build_result_output(&self) -> String {
        self.load_js_file("js/result_output.js")
            .unwrap_or_else(|_| include_str!("js/result_output.js").to_string())
    }
}
