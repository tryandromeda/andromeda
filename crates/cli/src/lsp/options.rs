// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::Url;

/// Configuration options for the Andromeda Language Server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    /// When to run lint diagnostics
    pub run: RunMode,
    /// Path to a custom configuration file
    pub config_path: Option<String>,
    /// How to handle unused disable directives
    pub unused_disable_directives: UnusedDisableDirectives,
    /// Special LSP flags
    pub flags: std::collections::HashMap<String, String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            run: RunMode::OnType,
            config_path: None,
            unused_disable_directives: UnusedDisableDirectives::Allow,
            flags: std::collections::HashMap::new(),
        }
    }
}

/// When to run lint diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RunMode {
    /// Run diagnostics on every file change
    OnType,
    /// Run diagnostics only when file is saved
    OnSave,
    /// Never run diagnostics automatically
    Never,
}

/// How to handle unused disable directives
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UnusedDisableDirectives {
    /// Allow unused disable directives
    Allow,
    /// Warn about unused disable directives
    Warn,
    /// Error on unused disable directives
    Error,
}

/// Workspace-specific options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceOptions {
    pub workspace_uri: Url,
    pub options: Options,
}
