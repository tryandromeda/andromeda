// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::config::{AndromedaConfig, ConfigManager, TaskDefinition};
use crate::error::AndromedaError;
use console::Style;
use std::collections::HashSet;
use std::path::Path;
use std::process::{Command, Stdio};

/// Boxed result type to avoid large error variants
type Result<T> = std::result::Result<T, Box<AndromedaError>>;

/// Task runner for executing defined tasks
pub struct TaskRunner {
    config: AndromedaConfig,
    config_dir: std::path::PathBuf,
}

impl TaskRunner {
    /// Create a new task runner with the given configuration
    pub fn new(config: AndromedaConfig, config_dir: std::path::PathBuf) -> Self {
        Self { config, config_dir }
    }

    /// List all available tasks
    pub fn list_tasks(&self) -> Result<()> {
        if self.config.tasks.is_empty() {
            println!("No tasks defined.");
            return Ok(());
        }

        let style_title = Style::new().bold().cyan();
        let style_task = Style::new().bold().green();
        let style_command = Style::new().dim();
        let style_desc = Style::new().italic().yellow();

        println!("{}", style_title.apply_to("Available tasks:"));
        println!();

        for (name, task) in &self.config.tasks {
            print!("- {}", style_task.apply_to(name));

            if let Some(desc) = task.description() {
                println!(" - {}", style_desc.apply_to(desc));
            } else {
                println!();
            }

            println!("    {}", style_command.apply_to(task.command()));

            if !task.dependencies().is_empty() {
                let deps = task.dependencies().join(", ");
                println!("    Dependencies: {}", style_command.apply_to(deps));
            }

            println!();
        }

        Ok(())
    }

    /// Run a specific task
    pub fn run_task(&self, task_name: &str) -> Result<()> {
        if !self.config.tasks.contains_key(task_name) {
            return Err(Box::new(AndromedaError::runtime_error(
                format!("Task '{task_name}' not found"),
                None,
                None,
                None,
                None,
            )));
        }

        // Resolve task dependencies and get execution order
        let execution_order = self.resolve_task_dependencies(task_name)?;

        let style_task = Style::new().bold().green();

        // Execute tasks in dependency order
        for current_task_name in execution_order {
            let task = &self.config.tasks[&current_task_name];

            println!("Running task: {}", style_task.apply_to(&current_task_name));

            self.execute_task(&current_task_name, task)?;
        }

        Ok(())
    }

    /// Resolve task dependencies and return execution order
    fn resolve_task_dependencies(&self, task_name: &str) -> Result<Vec<String>> {
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        let mut execution_order = Vec::new();

        self.visit_task(
            task_name,
            &mut visited,
            &mut temp_visited,
            &mut execution_order,
        )?;

        Ok(execution_order)
    }

    /// Visit a task and its dependencies (topological sort with cycle detection)
    fn visit_task(
        &self,
        task_name: &str,
        visited: &mut HashSet<String>,
        temp_visited: &mut HashSet<String>,
        execution_order: &mut Vec<String>,
    ) -> Result<()> {
        if temp_visited.contains(task_name) {
            return Err(Box::new(AndromedaError::runtime_error(
                format!("Circular dependency detected involving task '{task_name}'"),
                None,
                None,
                None,
                None,
            )));
        }

        if visited.contains(task_name) {
            return Ok(());
        }

        let task = self.config.tasks.get(task_name).ok_or_else(|| {
            Box::new(AndromedaError::runtime_error(
                format!("Task '{task_name}' not found"),
                None,
                None,
                None,
                None,
            ))
        })?;

        temp_visited.insert(task_name.to_string());

        // Visit dependencies first
        for dep in task.dependencies() {
            self.visit_task(dep, visited, temp_visited, execution_order)?;
        }

        temp_visited.remove(task_name);
        visited.insert(task_name.to_string());
        execution_order.push(task_name.to_string());

        Ok(())
    }

    /// Execute a single task
    fn execute_task(&self, task_name: &str, task: &TaskDefinition) -> Result<()> {
        let command_str = task.command();

        // Set up working directory
        let working_dir = if let Some(cwd) = task.cwd() {
            self.config_dir.join(cwd)
        } else {
            self.config_dir.clone()
        };

        // Parse and execute the command
        let mut cmd = if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", command_str]);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.args(["-c", command_str]);
            cmd
        };

        // Set working directory
        cmd.current_dir(&working_dir);

        // Set environment variables
        for (key, value) in task.env() {
            cmd.env(key, value);
        }

        // Inherit stdio for interactive tasks
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let style_command = Style::new().dim();
        println!("  {}", style_command.apply_to(command_str));

        // Execute the command
        let status = cmd.status().map_err(|e| {
            Box::new(AndromedaError::runtime_error(
                format!("Failed to execute task '{task_name}': {e}"),
                None,
                None,
                None,
                None,
            ))
        })?;

        if !status.success() {
            let exit_code = status.code().unwrap_or(-1);
            return Err(Box::new(AndromedaError::runtime_error(
                format!("Task '{task_name}' failed with exit code {exit_code}"),
                None,
                None,
                None,
                None,
            )));
        }

        println!();
        Ok(())
    }
}

/// Run a task using the configuration found in the current directory
pub fn run_task(task_name: Option<String>) -> Result<()> {
    // Load configuration
    let config = ConfigManager::load_or_default(None);

    // Find config directory
    let config_dir = if let Some((config_path, _)) = ConfigManager::find_config_file(None) {
        config_path.parent().unwrap_or(Path::new(".")).to_path_buf()
    } else {
        std::env::current_dir().map_err(|e| {
            Box::new(AndromedaError::config_error(
                "Failed to get current directory".to_string(),
                None,
                Some(e),
            ))
        })?
    };

    let task_runner = TaskRunner::new(config, config_dir);

    match task_name {
        Some(name) => task_runner.run_task(&name),
        None => task_runner.list_tasks(),
    }
}
