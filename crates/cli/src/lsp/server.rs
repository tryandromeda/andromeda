// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![allow(clippy::result_large_err)]

use crate::error::{CliError, CliResult};
use crate::lsp::backend::AndromedaBackend;
use tower_lsp::{LspService, Server};

/// Start the Language Server Protocol server
pub fn run_server() -> CliResult<()> {
    let runtime = tokio::runtime::Handle::try_current()
        .map_err(|error| CliError::lsp_error("tokio_runtime", error.to_string(), Some(error)))?;

    runtime.block_on(async {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let (service, socket) = LspService::build(AndromedaBackend::new).finish();

        Server::new(stdin, stdout, socket).serve(service).await;
    });

    Ok(())
}
