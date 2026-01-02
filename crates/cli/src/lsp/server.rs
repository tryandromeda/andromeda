// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::backend::AndromedaBackend;
use tower_lsp::{LspService, Server};

/// Start the Language Server Protocol server
pub fn run_server() -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    rt.block_on(async {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let (service, socket) = LspService::build(AndromedaBackend::new).finish();

        Server::new(stdin, stdout, socket).serve(service).await;
    });

    Ok(())
}
