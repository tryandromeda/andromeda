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

impl CryptoExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "crypto",
            ops: vec![
                ExtensionOp::new(
                    "internal_crypto_getRandomValues",
                    Self::internal_crypto_get_random_values,
                    1,
                ),
                ExtensionOp::new(
                    "internal_crypto_randomUUID",
                    Self::internal_crypto_random_uuid,
                    0,
                ),
                // SubtleCrypto operations
                ExtensionOp::new("internal_subtle_digest", Self::internal_subtle_digest, 2),
                ExtensionOp::new(
                    "internal_subtle_generateKey",
                    Self::internal_subtle_generate_key,
                    3,
                ),
                ExtensionOp::new(
                    "internal_subtle_importKey",
                    Self::internal_subtle_import_key,
                    5,
                ),
                ExtensionOp::new(
                    "internal_subtle_exportKey",
                    Self::internal_subtle_export_key,
                    2,
                ),
                ExtensionOp::new("internal_subtle_encrypt", Self::internal_subtle_encrypt, 3),
                ExtensionOp::new("internal_subtle_decrypt", Self::internal_subtle_decrypt, 3),
                ExtensionOp::new("internal_subtle_sign", Self::internal_subtle_sign, 3),
                ExtensionOp::new("internal_subtle_verify", Self::internal_subtle_verify, 4),
            ],
            storage: None,
            files: vec![include_str!("./mod.ts")],
        }
    }
    fn internal_crypto_get_random_values<'gc>(
        _agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let _array_arg = args[0];

        // For now, just return undefined and let TypeScript handle the array filling
        // The TypeScript side will handle the actual filling with secure random values
        // TODO: When Nova VM supports proper typed array manipulation, implement.

        // Generate some random bytes to ensure the RNG is working
        let mut rng = rand::rng();
        let _test_bytes = rng.next_u64(); // Just to verify RNG works

        // Return undefined (TypeScript will handle the array filling and return the original array)
        Ok(Value::Undefined)
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
}
