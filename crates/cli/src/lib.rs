// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Andromeda CLI library.
//!
//! This crate provides the command-line interface functionality for Andromeda,
//! including running, compiling, formatting, linting, and bundling JavaScript/TypeScript code.

pub mod bundle;
pub mod check;
pub mod compile;
pub mod config;
pub mod error;
pub mod format;
pub mod helper;
pub mod lint;
pub mod lsp;
pub mod repl;
pub mod run;
pub mod styles;
pub mod task;

// Re-export error types for convenience
pub use error::{CliError, CliResult, IntoCliResult};

// Keep old names for backwards compatibility
#[doc(hidden)]
#[deprecated(since = "0.2.0", note = "Use CliError instead")]
pub use error::CliError as AndromedaError;

#[doc(hidden)]
#[deprecated(since = "0.2.0", note = "Use CliResult instead")]
pub use error::CliResult as Result;
