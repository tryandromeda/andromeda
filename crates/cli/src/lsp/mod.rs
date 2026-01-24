// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod backend;
pub mod capabilities;
pub mod completions;
pub mod diagnostic_converter;
pub mod options;
pub mod server;

use crate::error::{CliError, CliResult};
use server::run_server;

/// Start the LSP server
#[allow(clippy::result_large_err)]
pub fn run_lsp_server() -> CliResult<()> {
    env_logger::init();
    log::info!("Starting Andromeda Language Server");

    run_server().map_err(|e| CliError::runtime_error_simple(format!("LSP server failed: {e}")))
}
