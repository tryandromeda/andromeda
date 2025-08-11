// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod headers;
mod request;
mod response;

use andromeda_core::Extension;
pub use headers::*;
pub use request::*;
pub use response::*;

#[derive(Default)]
pub struct FetchExt;

impl FetchExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "fetch",
            ops: vec![],
            storage: None,
            files: vec![
                include_str!("./headers/mod.ts"),
                include_str!("./request/mod.ts"),
                include_str!("./response/mod.ts"),
                include_str!("./fetch/mod.ts"),
            ],
        }
    }
}
