// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::Extension;

#[derive(Default)]
pub struct WebIDLExt;

impl WebIDLExt {
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn new_extension() -> Extension {
        Extension {
            name: "webidl",
            ops: vec![],
            storage: None,
            files: vec![include_str!("./mod.ts")],
        }
    }
}
