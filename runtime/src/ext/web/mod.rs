// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use andromeda_core::{Extension, ExtensionOp};
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult, agent::ExceptionType},
        types::Value,
    },
    engine::context::{Bindable, GcScope, NoGcScope},
};

use std::time::{Instant, SystemTime, UNIX_EPOCH};

// TODO: Get the time origin from when the runtime starts
// Temporarily use a static lock to initialize it once
// This is a workaround until we have a proper runtime initialization
static TIME_ORIGIN: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

#[derive(Default)]
pub struct WebExt;

impl WebExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "web",
            ops: vec![
                ExtensionOp::new("internal_btoa", Self::internal_btoa, 1, false),
                ExtensionOp::new("internal_atob", Self::internal_atob, 1, false),
                ExtensionOp::new("internal_text_encode", Self::internal_text_encode, 1, false),
                ExtensionOp::new("internal_text_decode", Self::internal_text_decode, 3, false),
                ExtensionOp::new(
                    "internal_text_encode_into",
                    Self::internal_text_encode_into,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_performance_now",
                    Self::internal_performance_now,
                    0,
                    false,
                ),
                ExtensionOp::new(
                    "internal_performance_time_origin",
                    Self::internal_performance_time_origin,
                    0,
                    false,
                ),
                ExtensionOp::new(
                    "internal_navigator_user_agent",
                    Self::internal_navigator_user_agent,
                    0,
                    false,
                ),
            ],
            storage: None,
            files: vec![
                include_str!("./event.ts"),
                include_str!("./dom_exception.ts"),
                include_str!("./text_encoding.ts"),
                include_str!("./performance.ts"),
                include_str!("./queue_microtask.ts"),
                include_str!("./structured_clone.ts"),
                include_str!("./navigator.ts"),
            ],
        }
    }

    pub fn internal_btoa<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let input = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let rust_string = input
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();
        let gc = gc.into_nogc();
        for c in rust_string.chars() {
            if c as u32 > 0xFF {
                // TODO: Returning an InvalidCharacterError is the correct behavior.
                // ref: https://html.spec.whatwg.org/multipage/webappapis.html#atob
                return Err(agent.throw_exception(ExceptionType::Error, format!(
                    "InvalidCharacterError: The string to be encoded contains characters outside of the Latin1 range. Found: '{c}'"
                ), gc).unbind());
            }
        }
        Ok(Self::forgiving_base64_encode(
            agent,
            rust_string.as_bytes(),
            gc,
        ))
    }

    pub fn internal_atob<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let input = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let rust_string = input
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();
        let gc = gc.into_nogc();
        for c in rust_string.chars() {
            if c as u32 > 0xFF {
                // TODO: Returning an InvalidCharacterError is the correct behavior.
                // ref: https://html.spec.whatwg.org/multipage/webappapis.html#atob
                return Err(agent.throw_exception(ExceptionType::Error, format!(
                    "InvalidCharacterError: The string to be encoded contains characters outside of the Latin1 range. Found: '{c}'"
                ), gc).unbind());
            }
        }
        let mut bytes = rust_string.into_bytes();
        let decoded_len_value = Self::forgiving_base64_decode_inplace(agent, &mut bytes, gc)?;
        Ok(decoded_len_value)
    }

    /// See <https://infra.spec.whatwg.org/#forgiving-base64>
    pub fn forgiving_base64_encode(
        agent: &mut Agent,
        s: &[u8],
        gc: NoGcScope<'_, '_>,
    ) -> Value<'static> {
        let encoded_str = base64_simd::STANDARD.encode_to_string(s);
        Value::from_string(agent, encoded_str, gc).unbind()
    }

    /// See <https://infra.spec.whatwg.org/#forgiving-base64>
    fn forgiving_base64_decode_inplace(
        agent: &mut Agent,
        input: &mut [u8],
        gc: NoGcScope<'_, '_>,
    ) -> JsResult<'static, Value<'static>> {
        let decoded_bytes =
            match base64_simd::forgiving_decode_inplace(input) {
                Ok(decoded) => decoded,
                Err(_) => {
                    return Err(agent.throw_exception_with_static_message(
                    ExceptionType::Error,
                    "InvalidCharacterError: The string to be decoded is not correctly encoded.",
                    gc,
                ).unbind());
                }
            };
        Ok(Value::from_string(
            agent,
            String::from_utf8_lossy(decoded_bytes).to_string(),
            gc,
        )
        .unbind())
    }

    pub fn internal_text_encode<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let input = args.get(0).to_string(agent, gc.reborrow()).unbind()?;
        let rust_string = input
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();
        let gc = gc.into_nogc();

        // TextEncoder always uses UTF-8 encoding
        let bytes = rust_string.as_bytes();

        // For now, return a serialized representation that can be parsed on the JS side
        // Format: comma-separated byte values
        let bytes_str = bytes
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(",");

        Ok(Value::from_string(agent, bytes_str, gc).unbind())
    }
    pub fn internal_text_decode<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // args: [bytes_string, encoding, fatal]
        let bytes_arg = args.get(0);
        let encoding_arg = args.get(1);
        let fatal_arg = args.get(2);

        // Get encoding (default to "utf-8")
        let encoding = if encoding_arg.is_undefined() {
            "utf-8".to_string()
        } else {
            let enc_str = encoding_arg.to_string(agent, gc.reborrow()).unbind()?;
            enc_str
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .to_string()
        };

        // Parse bytes from comma-separated string format
        let bytes_str = bytes_arg.to_string(agent, gc.reborrow()).unbind()?;
        let bytes_string = bytes_str
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let gc_no = gc.into_nogc();

        // Normalize encoding name
        let encoding = encoding.to_lowercase().replace('_', "-");

        // Get fatal flag (default to false) - check if value is truthy
        let fatal = !fatal_arg.is_undefined() && !fatal_arg.is_null();

        let bytes: Vec<u8> = if bytes_string.is_empty() {
            Vec::new()
        } else {
            bytes_string
                .split(',')
                .filter_map(|s| s.trim().parse::<u8>().ok())
                .collect()
        };

        // Decode based on encoding
        let result_string = match encoding.as_str() {
            "utf-8" | "utf8" => {
                if fatal {
                    match std::str::from_utf8(&bytes) {
                        Ok(s) => s.to_string(),
                        Err(_) => {
                            return Err(agent
                                .throw_exception(
                                    ExceptionType::TypeError,
                                    "The encoded data was not valid UTF-8".to_string(),
                                    gc_no,
                                )
                                .unbind());
                        }
                    }
                } else {
                    String::from_utf8_lossy(&bytes).to_string()
                }
            }
            "utf-16le" | "utf-16" => {
                if bytes.len() % 2 != 0 {
                    if fatal {
                        return Err(agent
                            .throw_exception(
                                ExceptionType::TypeError,
                                "The encoded data was not valid UTF-16LE".to_string(),
                                gc_no,
                            )
                            .unbind());
                    } else {
                        String::from_utf8_lossy(&bytes).to_string()
                    }
                } else {
                    let utf16_pairs: Vec<u16> = bytes
                        .chunks_exact(2)
                        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                        .collect();

                    if fatal {
                        match String::from_utf16(&utf16_pairs) {
                            Ok(s) => s,
                            Err(_) => {
                                return Err(agent
                                    .throw_exception(
                                        ExceptionType::TypeError,
                                        "The encoded data was not valid UTF-16LE".to_string(),
                                        gc_no,
                                    )
                                    .unbind());
                            }
                        }
                    } else {
                        String::from_utf16_lossy(&utf16_pairs)
                    }
                }
            }
            "utf-16be" => {
                if bytes.len() % 2 != 0 {
                    if fatal {
                        return Err(agent
                            .throw_exception(
                                ExceptionType::TypeError,
                                "The encoded data was not valid UTF-16BE".to_string(),
                                gc_no,
                            )
                            .unbind());
                    } else {
                        String::from_utf8_lossy(&bytes).to_string()
                    }
                } else {
                    let utf16_pairs: Vec<u16> = bytes
                        .chunks_exact(2)
                        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                        .collect();

                    if fatal {
                        match String::from_utf16(&utf16_pairs) {
                            Ok(s) => s,
                            Err(_) => {
                                return Err(agent
                                    .throw_exception(
                                        ExceptionType::TypeError,
                                        "The encoded data was not valid UTF-16BE".to_string(),
                                        gc_no,
                                    )
                                    .unbind());
                            }
                        }
                    } else {
                        String::from_utf16_lossy(&utf16_pairs)
                    }
                }
            }
            "iso-8859-1" | "latin1" | "windows-1252" => {
                // ISO-8859-1/Latin-1: direct byte-to-char mapping for 0-255
                bytes.iter().map(|&b| b as char).collect()
            }
            _ => {
                // Unsupported encoding - default to UTF-8 replacement behavior
                if fatal {
                    return Err(agent
                        .throw_exception(
                            ExceptionType::RangeError,
                            format!("The encoding '{encoding}' is not supported"),
                            gc_no,
                        )
                        .unbind());
                } else {
                    String::from_utf8_lossy(&bytes).to_string()
                }
            }
        };

        Ok(Value::from_string(agent, result_string, gc_no).unbind())
    }

    pub fn internal_text_encode_into<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // args: [source_string, destination_bytes_string, destination_length]
        let source_arg = args.get(0);
        let dest_arg = args.get(1);
        let dest_len_arg = args.get(2);

        let source_str = source_arg.to_string(agent, gc.reborrow()).unbind()?;
        let source_string = source_str
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let dest_str = dest_arg.to_string(agent, gc.reborrow()).unbind()?;
        let dest_string = dest_str
            .as_str(agent)
            .expect("String is not valid UTF-8")
            .to_string();

        let dest_len_number = dest_len_arg.to_number(agent, gc.reborrow()).unbind()?;
        let dest_len = dest_len_number.into_f64(agent) as usize;

        let gc_no = gc.into_nogc();

        // Parse existing destination bytes
        let mut dest_bytes: Vec<u8> = if dest_string.is_empty() {
            vec![0; dest_len]
        } else {
            let mut bytes: Vec<u8> = dest_string
                .split(',')
                .filter_map(|s| s.trim().parse::<u8>().ok())
                .collect();
            bytes.resize(dest_len, 0);
            bytes
        };
        let source_bytes = source_string.as_bytes();

        // Copy as much as fits
        let copy_len = std::cmp::min(source_bytes.len(), dest_bytes.len());
        dest_bytes[..copy_len].copy_from_slice(&source_bytes[..copy_len]);

        // Calculate how many characters were read (not bytes)
        // We need to find the boundary of complete UTF-8 characters that fit
        let mut chars_read = 0;
        let mut bytes_processed = 0;

        for ch in source_string.chars() {
            let char_bytes = ch.len_utf8();
            if bytes_processed + char_bytes <= copy_len {
                chars_read += 1;
                bytes_processed += char_bytes;
            } else {
                break;
            }
        }

        // Return result as "bytes_string:read:written"
        let result_bytes_str = dest_bytes
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let result = format!("{result_bytes_str}:{chars_read}:{bytes_processed}");

        Ok(Value::from_string(agent, result, gc_no).unbind())
    }

    pub fn internal_performance_now<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let gc = gc.into_nogc();
        let origin = TIME_ORIGIN.get_or_init(Instant::now);

        let elapsed = origin.elapsed();
        let elapsed_ms =
            elapsed.as_secs_f64() * 1000.0 + elapsed.subsec_nanos() as f64 / 1_000_000.0;

        Ok(Value::from_f64(agent, elapsed_ms, gc).unbind())
    }

    pub fn internal_performance_time_origin<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let gc = gc.into_nogc();
        static TIME_ORIGIN_UNIX: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
        let origin_ms = *TIME_ORIGIN_UNIX.get_or_init(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64()
                * 1000.0
        });
        Ok(Value::from_f64(agent, origin_ms, gc).unbind())
    }

    pub fn internal_navigator_user_agent<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let gc = gc.into_nogc();
        let user_agent = format!(
            "Mozilla/5.0 ({}) AppleWebKit/537.36 (KHTML, like Gecko) Andromeda/1.0.0",
            Self::get_platform_string()
        );
        Ok(Value::from_string(agent, user_agent, gc).unbind())
    }

    fn get_platform_string() -> &'static str {
        #[cfg(target_os = "windows")]
        {
            if cfg!(target_arch = "x86_64") {
                "Windows NT 10.0; Win64; x64"
            } else if cfg!(target_arch = "aarch64") {
                "Windows NT 10.0; ARM64"
            } else {
                "Windows NT 10.0"
            }
        }
        #[cfg(target_os = "macos")]
        {
            // For web compatibility, we always report as Intel Mac regardless of actual architecture
            "Macintosh; Intel Mac OS X 10_15_7"
        }
        #[cfg(target_os = "linux")]
        {
            if cfg!(target_arch = "x86_64") {
                "X11; Linux x86_64"
            } else if cfg!(target_arch = "aarch64") {
                "X11; Linux aarch64"
            } else {
                "X11; Linux"
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            "Unknown"
        }
    }
}
