// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::sync::OnceLock;

/// Global LLM provider instance
static LLM_PROVIDER: OnceLock<Box<dyn LlmSuggestionProvider>> = OnceLock::new();

/// Configuration for LLM suggestions
#[derive(Debug, Clone)]
pub struct LlmSuggestionConfig {
    /// Whether LLM suggestions are enabled
    pub enabled: bool,
    /// Maximum time to wait for a suggestion (in milliseconds)
    pub timeout_ms: u64,
    /// The model ID to use (if applicable)
    pub model_id: Option<String>,
}

impl Default for LlmSuggestionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_ms: 10000, // 10 seconds default timeout
            model_id: None,
        }
    }
}

/// Error context provided to the LLM for generating suggestions
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// The error message
    pub error_message: String,
    /// The source code where the error occurred (if available)
    pub source_code: Option<String>,
    /// The file path (if available)
    pub file_path: Option<String>,
    /// The line number where the error occurred (if available)
    pub line_number: Option<u32>,
    /// The column number where the error occurred (if available)
    pub column_number: Option<u32>,
    /// The error type/code (e.g., "ReferenceError", "TypeError")
    pub error_type: Option<String>,
    /// Stack trace if available
    pub stack_trace: Option<String>,
}

impl ErrorContext {
    /// Create a new error context with just the error message
    pub fn new(error_message: impl Into<String>) -> Self {
        Self {
            error_message: error_message.into(),
            source_code: None,
            file_path: None,
            line_number: None,
            column_number: None,
            error_type: None,
            stack_trace: None,
        }
    }

    /// Add source code context
    pub fn with_source_code(mut self, source_code: impl Into<String>) -> Self {
        self.source_code = Some(source_code.into());
        self
    }

    /// Add file path
    pub fn with_file_path(mut self, file_path: impl Into<String>) -> Self {
        self.file_path = Some(file_path.into());
        self
    }

    /// Add line number
    pub fn with_line_number(mut self, line: u32) -> Self {
        self.line_number = Some(line);
        self
    }

    /// Add column number
    pub fn with_column_number(mut self, column: u32) -> Self {
        self.column_number = Some(column);
        self
    }

    /// Add error type
    pub fn with_error_type(mut self, error_type: impl Into<String>) -> Self {
        self.error_type = Some(error_type.into());
        self
    }

    /// Add stack trace
    pub fn with_stack_trace(mut self, stack_trace: impl Into<String>) -> Self {
        self.stack_trace = Some(stack_trace.into());
        self
    }
}

/// Result of an LLM suggestion request
#[derive(Debug, Clone)]
pub struct LlmSuggestion {
    /// The suggestion text
    pub suggestion: String,
    /// The provider that generated this suggestion
    pub provider_name: String,
    /// The model used (if applicable)
    pub model_id: Option<String>,
}

/// Errors that can occur when fetching LLM suggestions
#[derive(Debug)]
pub enum LlmSuggestionError {
    /// The LLM provider is not initialized
    NotInitialized,
    /// The provider failed to generate a suggestion
    ProviderError(String),
    /// The request timed out
    Timeout,
    /// The feature is disabled
    Disabled,
    /// Authentication error
    AuthenticationError(String),
    /// Network error
    NetworkError(String),
}

impl std::fmt::Display for LlmSuggestionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "LLM provider not initialized"),
            Self::ProviderError(msg) => write!(f, "LLM provider error: {}", msg),
            Self::Timeout => write!(f, "LLM suggestion request timed out"),
            Self::Disabled => write!(f, "LLM suggestions are disabled"),
            Self::AuthenticationError(msg) => write!(f, "LLM authentication error: {}", msg),
            Self::NetworkError(msg) => write!(f, "LLM network error: {}", msg),
        }
    }
}

impl std::error::Error for LlmSuggestionError {}

/// Implement this trait to add support for different LLM backends.
pub trait LlmSuggestionProvider: Send + Sync {
    /// Get the name of this provider
    fn name(&self) -> &str;

    /// Check if the provider is available and properly configured
    fn is_available(&self) -> bool;

    /// Get a suggestion for the given error context
    fn get_suggestion(&self, context: &ErrorContext) -> Result<LlmSuggestion, LlmSuggestionError>;
}

/// Initialize the global LLM provider
pub fn init_llm_provider(provider: Box<dyn LlmSuggestionProvider>) {
    let _ = LLM_PROVIDER.set(provider);
}

/// Check if an LLM provider is initialized
pub fn is_llm_initialized() -> bool {
    LLM_PROVIDER.get().is_some()
}

/// Get a suggestion for an error using the global LLM provider
pub fn get_error_suggestion(context: &ErrorContext) -> Option<LlmSuggestion> {
    LLM_PROVIDER
        .get()
        .and_then(|provider| provider.get_suggestion(context).ok())
}

/// Get a suggestion with explicit error handling
pub fn try_get_error_suggestion(
    context: &ErrorContext,
) -> Result<LlmSuggestion, LlmSuggestionError> {
    match LLM_PROVIDER.get() {
        Some(provider) => provider.get_suggestion(context),
        None => Err(LlmSuggestionError::NotInitialized),
    }
}

#[cfg(feature = "llm")]
pub mod copilot {
    use super::*;
    use copilot_client::{CopilotClient, CopilotError, Message};
    use std::sync::Mutex;

    /// GitHub Copilot-based suggestion provider
    pub struct CopilotSuggestionProvider {
        client: Mutex<Option<CopilotClient>>,
        model_id: String,
        config: LlmSuggestionConfig,
    }

    impl CopilotSuggestionProvider {
        /// Create a new Copilot suggestion provider
        pub fn new(config: LlmSuggestionConfig) -> Result<Self, LlmSuggestionError> {
            let model_id = config
                .model_id
                .clone()
                .unwrap_or_else(|| "gpt-4o".to_string());

            Ok(Self {
                client: Mutex::new(None),
                model_id,
                config,
            })
        }

        /// Initialize the Copilot client asynchronously
        fn ensure_client_initialized(&self) -> Result<(), LlmSuggestionError> {
            let mut client_guard = self.client.lock().map_err(|e| {
                LlmSuggestionError::ProviderError(format!("Failed to acquire lock: {}", e))
            })?;

            if client_guard.is_some() {
                return Ok(());
            }

            // Create a new runtime for the async initialization
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| {
                    LlmSuggestionError::ProviderError(format!(
                        "Failed to create tokio runtime: {}",
                        e
                    ))
                })?;

            let client = rt.block_on(async {
                CopilotClient::from_env_with_models("andromeda/1.0.0".to_string()).await
            });

            match client {
                Ok(c) => {
                    *client_guard = Some(c);
                    Ok(())
                }
                Err(e) => Err(copilot_error_to_llm_error(e)),
            }
        }

        /// Build the prompt for the LLM
        fn build_prompt(&self, context: &ErrorContext) -> Vec<Message> {
            let system_prompt = r#"You are a helpful assistant integrated into the Andromeda JavaScript/TypeScript runtime. Your role is to provide concise, actionable suggestions to help developers fix errors in their code.

Guidelines:
- Be concise and direct - aim for 2-4 lines max
- Focus on the most likely fix
- If you suggest code changes, keep them minimal
- Don't repeat the error message
- Don't apologize or use filler phrases
- Use technical language appropriate for developers"#;

            let mut user_content = format!("Error: {}\n", context.error_message);

            if let Some(ref error_type) = context.error_type {
                user_content.push_str(&format!("Type: {}\n", error_type));
            }

            if let Some(ref file_path) = context.file_path {
                user_content.push_str(&format!("File: {}\n", file_path));
            }

            if let Some(line) = context.line_number {
                user_content.push_str(&format!("Line: {}\n", line));
            }

            if let Some(ref source) = context.source_code {
                // Limit source code to relevant portion (around 20 lines)
                let lines: Vec<&str> = source.lines().collect();
                let relevant_source = if lines.len() > 20 {
                    if let Some(line_num) = context.line_number {
                        let start = (line_num as usize).saturating_sub(10);
                        let end = (start + 20).min(lines.len());
                        lines[start..end].join("\n")
                    } else {
                        lines[..20].join("\n")
                    }
                } else {
                    source.clone()
                };
                user_content.push_str(&format!("\nCode:\n```\n{}\n```\n", relevant_source));
            }

            user_content.push_str("\nProvide a brief suggestion to fix this error:");

            vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: user_content,
                },
            ]
        }
    }

    impl LlmSuggestionProvider for CopilotSuggestionProvider {
        fn name(&self) -> &str {
            "GitHub Copilot"
        }

        fn is_available(&self) -> bool {
            self.config.enabled && self.ensure_client_initialized().is_ok()
        }

        fn get_suggestion(
            &self,
            context: &ErrorContext,
        ) -> Result<LlmSuggestion, LlmSuggestionError> {
            if !self.config.enabled {
                return Err(LlmSuggestionError::Disabled);
            }

            self.ensure_client_initialized()?;

            let client_guard = self.client.lock().map_err(|e| {
                LlmSuggestionError::ProviderError(format!("Failed to acquire lock: {}", e))
            })?;

            let client = client_guard
                .as_ref()
                .ok_or(LlmSuggestionError::NotInitialized)?;

            let messages = self.build_prompt(context);
            let model_id = self.model_id.clone();

            // Create a runtime for the async call
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| {
                    LlmSuggestionError::ProviderError(format!(
                        "Failed to create tokio runtime: {}",
                        e
                    ))
                })?;

            let timeout_duration = std::time::Duration::from_millis(self.config.timeout_ms);

            let result = rt.block_on(async {
                tokio::time::timeout(
                    timeout_duration,
                    client.chat_completion(messages, model_id.clone()),
                )
                .await
            });

            match result {
                Ok(Ok(response)) => {
                    let suggestion = response
                        .choices
                        .first()
                        .map(|choice| choice.message.content.clone())
                        .unwrap_or_else(|| "No suggestion available.".to_string());

                    Ok(LlmSuggestion {
                        suggestion,
                        provider_name: self.name().to_string(),
                        model_id: Some(self.model_id.clone()),
                    })
                }
                Ok(Err(e)) => Err(copilot_error_to_llm_error(e)),
                Err(_) => Err(LlmSuggestionError::Timeout),
            }
        }
    }

    /// Convert Copilot errors to LLM suggestion errors
    fn copilot_error_to_llm_error(error: CopilotError) -> LlmSuggestionError {
        match error {
            CopilotError::TokenError(msg) => LlmSuggestionError::AuthenticationError(format!(
                "GitHub token error: {}. Set GITHUB_TOKEN environment variable or configure GitHub CLI.",
                msg
            )),
            CopilotError::InvalidModel(model) => {
                LlmSuggestionError::ProviderError(format!("Invalid model: {}", model))
            }
            CopilotError::HttpError(msg) => LlmSuggestionError::NetworkError(msg),
            CopilotError::Other(msg) => {
                LlmSuggestionError::ProviderError(format!("Copilot error: {}", msg))
            }
        }
    }

    /// Initialize the global LLM provider with GitHub Copilot
    pub fn init_copilot_provider(config: LlmSuggestionConfig) -> Result<(), LlmSuggestionError> {
        let provider = CopilotSuggestionProvider::new(config)?;
        init_llm_provider(Box::new(provider));
        Ok(())
    }
}
