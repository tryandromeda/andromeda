// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::Serialize;

/// A single window event queued for delivery to JS on the next poll.
#[derive(Debug, Clone, Serialize)]
pub struct SerializedWindowEvent {
    /// Target window rid.
    pub rid: u32,
    /// Event type name, intentionally DOM-aligned.
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// Event-specific payload — shape depends on `kind`.
    pub detail: EventDetail,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum EventDetail {
    Empty {},
    Resize {
        width: u32,
        height: u32,
        #[serde(rename = "scaleFactor")]
        scale_factor: f64,
    },
    Key {
        key: String,
        code: String,
        /// Legacy numeric keyCode per the MDN table. See
        /// https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode
        #[serde(rename = "keyCode")]
        key_code: u32,
        /// Alias of `keyCode` — kept for compat with old code that reads
        /// `event.which`.
        which: u32,
        /// 0 = standard, 1 = left modifier, 2 = right modifier, 3 = numpad.
        location: u8,
        #[serde(rename = "altKey")]
        alt_key: bool,
        #[serde(rename = "ctrlKey")]
        ctrl_key: bool,
        #[serde(rename = "metaKey")]
        meta_key: bool,
        #[serde(rename = "shiftKey")]
        shift_key: bool,
        repeat: bool,
        /// Always `false` in this runtime since we don't have IME support, but included for completeness.
        #[serde(rename = "isComposing")]
        is_composing: bool,
    },
    Mouse {
        x: f64,
        y: f64,
        button: i32,
        buttons: u32,
        #[serde(rename = "altKey")]
        alt_key: bool,
        #[serde(rename = "ctrlKey")]
        ctrl_key: bool,
        #[serde(rename = "metaKey")]
        meta_key: bool,
        #[serde(rename = "shiftKey")]
        shift_key: bool,
    },
}

impl SerializedWindowEvent {
    pub fn close(rid: u32) -> Self {
        Self {
            rid,
            kind: "close",
            detail: EventDetail::Empty {},
        }
    }

    pub fn resize(rid: u32, width: u32, height: u32, scale_factor: f64) -> Self {
        Self {
            rid,
            kind: "resize",
            detail: EventDetail::Resize {
                width,
                height,
                scale_factor,
            },
        }
    }
}
