// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![allow(clippy::result_large_err)]

use super::backend::AndromedaBackend;
use crate::error::{CliError, CliResult};
use tower_lsp::{LspService, Server};

/// Start the Language Server Protocol server
pub fn run_server() -> CliResult<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CliError::lsp_error("tokio_runtime", e.to_string(), Some(e)))?;

    rt.block_on(async {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let (service, socket) = LspService::build(AndromedaBackend::new).finish();

        Server::new(stdin, stdout, socket).serve(service).await;
    });

    Ok(())
}
