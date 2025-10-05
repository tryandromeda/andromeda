// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::Extension;

#[derive(Default)]
pub struct FetchExt;

impl FetchExt {
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn new_extension() -> Extension {
        Extension {
            name: "fetch",
            ops: vec![],
            storage: None,
            files: vec![
                include_str!("./body/inner_body.ts"),
                include_str!("./body/extract.ts"),
                include_str!("./body/consume.ts"),
                include_str!("./body/body_mixin.ts"),
                include_str!("./headers/mod.ts"),
                include_str!("./cors.ts"),
                include_str!("./response_filter.ts"),
                include_str!("./request/mod.ts"),
                include_str!("./response/mod.ts"),
                include_str!("./fetch/mod.ts"),
            ],
        }
    }
}
