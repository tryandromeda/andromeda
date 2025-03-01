use andromeda_core::Extension;

#[derive(Default)]
pub struct HeadersExt;

impl HeadersExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "headers",
            ops: vec![],
            storage: None,
            files: vec![include_str!("./mod.ts")],
        }
    }
}
