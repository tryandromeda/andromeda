// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod subtle;
use andromeda_core::{Extension, ExtensionOp};
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::{Bindable, GcScope},
};
use rand::RngCore;

pub use subtle::SubtleCrypto;

/// Crypto extension for Andromeda.
/// This extension provides access to cryptographic functions following the Web Crypto API.
#[derive(Default)]
pub struct CryptoExt;

#[cfg_attr(feature = "hotpath", hotpath::measure_all)]
impl CryptoExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "crypto",
            ops: vec![
                // Crypto interface operations
                ExtensionOp::new(
                    "internal_crypto_getRandomValues",
                    Self::internal_crypto_get_random_values,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_crypto_randomUUID",
                    Self::internal_crypto_random_uuid,
                    0,
                    false,
                ),
                // SubtleCrypto core operations
                ExtensionOp::new(
                    "internal_subtle_digest",
                    Self::internal_subtle_digest,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_subtle_generateKey",
                    Self::internal_subtle_generate_key,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "internal_subtle_importKey",
                    Self::internal_subtle_import_key,
                    5,
                    false,
                ),
                ExtensionOp::new(
                    "internal_subtle_exportKey",
                    Self::internal_subtle_export_key,
                    2,
                    false,
                ),
                ExtensionOp::new(
                    "internal_subtle_encrypt",
                    Self::internal_subtle_encrypt,
                    3,
                    false,
                ),
                ExtensionOp::new(
                    "internal_subtle_decrypt",
                    Self::internal_subtle_decrypt,
                    3,
                    false,
                ),
                ExtensionOp::new("internal_subtle_sign", Self::internal_subtle_sign, 3, false),
                ExtensionOp::new(
                    "internal_subtle_verify",
                    Self::internal_subtle_verify,
                    4,
                    false,
                ),
                // Key derivation operations
                ExtensionOp::new(
                    "internal_subtle_deriveKey",
                    Self::internal_subtle_derive_key,
                    5,
                    false,
                ),
                ExtensionOp::new(
                    "internal_subtle_deriveBits",
                    Self::internal_subtle_derive_bits,
                    3,
                    false,
                ),
                // Key wrapping operations
                ExtensionOp::new(
                    "internal_subtle_wrapKey",
                    Self::internal_subtle_wrap_key,
                    4,
                    false,
                ),
                ExtensionOp::new(
                    "internal_subtle_unwrapKey",
                    Self::internal_subtle_unwrap_key,
                    7,
                    false,
                ),
                // Crypto key operations
                ExtensionOp::new(
                    "internal_cryptokey_create",
                    Self::internal_cryptokey_create,
                    5,
                    false,
                ),
                ExtensionOp::new(
                    "internal_cryptokey_get_type",
                    Self::internal_cryptokey_get_type,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_cryptokey_get_extractable",
                    Self::internal_cryptokey_get_extractable,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_cryptokey_get_algorithm",
                    Self::internal_cryptokey_get_algorithm,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_cryptokey_get_usages",
                    Self::internal_cryptokey_get_usages,
                    1,
                    false,
                ),
                // Array buffer operations for crypto
                ExtensionOp::new(
                    "internal_crypto_create_array_buffer",
                    Self::internal_crypto_create_array_buffer,
                    1,
                    false,
                ),
                ExtensionOp::new(
                    "internal_crypto_get_buffer_bytes",
                    Self::internal_crypto_get_buffer_bytes,
                    1,
                    false,
                ),
            ],
            storage: None,
            files: vec![include_str!("./mod.ts")],
        }
    }
    fn internal_crypto_get_random_values<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: implement proper typed array handling for getRandomValues
        let _array_arg = args[0];

        let mut rng = rand::rng();
        let mut bytes = vec![0u8; 65536];
        rng.fill_bytes(&mut bytes);

        let random_data_base64 = base64_simd::STANDARD.encode_to_string(&bytes);

        Ok(nova_vm::ecmascript::types::String::from_string(
            agent,
            random_data_base64,
            gc.into_nogc(),
        )
        .unbind()
        .into())
    }
    fn internal_crypto_random_uuid<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let mut rng = rand::rng();
        let mut bytes = [0u8; 16];
        rng.fill_bytes(&mut bytes);

        // Set version (4) and variant bits according to RFC 4122
        bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
        bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 10

        let uuid = format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
            bytes[4],
            bytes[5],
            bytes[6],
            bytes[7],
            bytes[8],
            bytes[9],
            bytes[10],
            bytes[11],
            bytes[12],
            bytes[13],
            bytes[14],
            bytes[15]
        );

        Ok(
            nova_vm::ecmascript::types::String::from_string(agent, uuid, gc.nogc())
                .unbind()
                .into(),
        )
    }
    fn internal_subtle_digest<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        SubtleCrypto::digest(agent, args, gc)
    }

    fn internal_subtle_generate_key<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        SubtleCrypto::generate_key(agent, args, gc)
    }

    fn internal_subtle_import_key<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        SubtleCrypto::import_key(agent, args, gc)
    }

    fn internal_subtle_export_key<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        SubtleCrypto::export_key(agent, args, gc)
    }

    fn internal_subtle_encrypt<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        SubtleCrypto::encrypt(agent, args, gc)
    }

    fn internal_subtle_decrypt<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        SubtleCrypto::decrypt(agent, args, gc)
    }

    fn internal_subtle_sign<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        SubtleCrypto::sign(agent, args, gc)
    }

    fn internal_subtle_verify<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        SubtleCrypto::verify(agent, args, gc)
    }

    fn internal_subtle_derive_key<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        SubtleCrypto::derive_key(agent, args, gc)
    }

    fn internal_subtle_derive_bits<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        SubtleCrypto::derive_bits(agent, args, gc)
    }

    fn internal_subtle_wrap_key<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        SubtleCrypto::wrap_key(agent, args, gc)
    }

    fn internal_subtle_unwrap_key<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        SubtleCrypto::unwrap_key(agent, args, gc)
    }

    fn internal_cryptokey_create<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Implement CryptoKey creation
        let gc = gc.into_nogc();
        Err(agent
            .throw_exception_with_static_message(
                nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                "CryptoKey creation not yet implemented",
                gc,
            )
            .unbind())
    }

    fn internal_cryptokey_get_type<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Implement CryptoKey type getter
        Ok(
            nova_vm::ecmascript::types::String::from_string(agent, "secret".to_string(), gc.nogc())
                .unbind()
                .into(),
        )
    }

    fn internal_cryptokey_get_extractable<'gc>(
        _agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Implement CryptoKey extractable getter
        Ok(Value::Boolean(true))
    }

    fn internal_cryptokey_get_algorithm<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Implement CryptoKey algorithm getter
        Ok(
            nova_vm::ecmascript::types::String::from_string(
                agent,
                "AES-GCM".to_string(),
                gc.nogc(),
            )
            .unbind()
            .into(),
        )
    }

    fn internal_cryptokey_get_usages<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Implement CryptoKey usages getter
        Ok(nova_vm::ecmascript::types::String::from_string(
            agent,
            "encrypt,decrypt".to_string(),
            gc.nogc(),
        )
        .unbind()
        .into())
    }

    fn internal_crypto_create_array_buffer<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Implement proper ArrayBuffer creation from bytes
        let _bytes_arg = args[0];
        Ok(nova_vm::ecmascript::types::String::from_string(
            agent,
            "arraybuffer_placeholder".to_string(),
            gc.nogc(),
        )
        .unbind()
        .into())
    }

    fn internal_crypto_get_buffer_bytes<'gc>(
        agent: &mut Agent,
        _this: Value,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Implement proper bytes extraction from ArrayBuffer/TypedArray
        Ok(nova_vm::ecmascript::types::String::from_string(
            agent,
            "buffer_bytes_placeholder".to_string(),
            gc.nogc(),
        )
        .unbind()
        .into())
    }
}
