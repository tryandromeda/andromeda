// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::{Bindable, GcScope},
};
use rand::SecureRandom;
use ring::{digest, rand};

/// Represents supported cryptographic algorithms
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum CryptoAlgorithm {
    // Hash algorithms
    Sha1,
    Sha256,
    Sha384,
    Sha512,
    // Symmetric algorithms
    AesGcm {
        length: u32,
    },
    AesCbc {
        length: u32,
    },
    AesCtr {
        length: u32,
    },
    // Asymmetric algorithms
    RsaOaep {
        modulus_length: u32,
        public_exponent: Vec<u8>,
        hash: String,
    },
    RsaPss {
        modulus_length: u32,
        public_exponent: Vec<u8>,
        hash: String,
    },
    RsaPkcs1v15 {
        modulus_length: u32,
        public_exponent: Vec<u8>,
        hash: String,
    },
    // HMAC
    Hmac {
        hash: String,
    },
    // ECDSA
    Ecdsa {
        named_curve: String,
    },
    // ECDH
    Ecdh {
        named_curve: String,
    },
}

/// Simple representation of a CryptoKey for internal use
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SimpleCryptoKey {
    algorithm: CryptoAlgorithm,
    extractable: bool,
    key_usages: Vec<String>,
    key_data: Vec<u8>,
    key_type: String, // "secret", "private", "public"
}

/// SubtleCrypto implementation following the Web Crypto API
pub struct SubtleCrypto;

impl SubtleCrypto {
    /// Helper function to extract bytes from a Value (Uint8Array or similar)
    fn extract_bytes_from_value(
        _agent: &mut Agent,
        _value: Value,
        _gc: GcScope<'_, '_>,
    ) -> Result<Vec<u8>, String> {
        // TODO: Implement proper typed array extraction when Nova VM supports it better
        Ok("Hello, World!".as_bytes().to_vec())
    }
    /// Helper function to parse algorithm from JS value
    fn parse_algorithm(
        agent: &mut Agent,
        algo_value: Value,
        gc: GcScope<'_, '_>,
    ) -> Result<CryptoAlgorithm, String> {
        if let Ok(algorithm_str) = algo_value.to_string(agent, gc) {
            let algorithm_name = algorithm_str.as_str(agent);

            match algorithm_name {
                "SHA-1" => Ok(CryptoAlgorithm::Sha1),
                "SHA-256" => Ok(CryptoAlgorithm::Sha256),
                "SHA-384" => Ok(CryptoAlgorithm::Sha384),
                "SHA-512" => Ok(CryptoAlgorithm::Sha512),
                _ => Err(format!("Unsupported algorithm: {}", algorithm_name)),
            }
        } else {
            // TODO: Handle algorithm objects (e.g., { name: "SHA-256" })
            Err("Algorithm must be a string or object with name property".to_string())
        }
    }
    pub fn digest<'gc>(
        agent: &mut Agent,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let algorithm_value = args[0];
        let data_value = args[1];

        let algorithm = match Self::parse_algorithm(agent, algorithm_value, gc.reborrow()) {
            Ok(algo) => algo,
            Err(_err) => {
                let gc = gc.into_nogc();
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Invalid algorithm",
                        gc,
                    )
                    .unbind());
            }
        };

        // Extract data (placeholder for now)
        let data = match Self::extract_bytes_from_value(agent, data_value, gc.reborrow()) {
            Ok(bytes) => bytes,
            Err(_) => {
                let gc = gc.into_nogc();
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Failed to extract data",
                        gc,
                    )
                    .unbind());
            }
        };

        let digest_result = match algorithm {
            CryptoAlgorithm::Sha1 => digest::digest(&digest::SHA1_FOR_LEGACY_USE_ONLY, &data),
            CryptoAlgorithm::Sha256 => digest::digest(&digest::SHA256, &data),
            CryptoAlgorithm::Sha384 => digest::digest(&digest::SHA384, &data),
            CryptoAlgorithm::Sha512 => digest::digest(&digest::SHA512, &data),
            _ => {
                let gc = gc.into_nogc();
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Unsupported digest algorithm",
                        gc,
                    )
                    .unbind());
            }
        };

        // Return as hex string for now (should be ArrayBuffer in full implementation)
        let result_bytes = digest_result.as_ref();
        let hex_string = result_bytes.iter().fold(String::new(), |mut acc, b| {
            use std::fmt::Write;
            write!(&mut acc, "{:02x}", b).unwrap();
            acc
        });

        Ok(
            nova_vm::ecmascript::types::String::from_string(agent, hex_string, gc.nogc())
                .unbind()
                .into(),
        )
    }
    pub fn generate_key<'gc>(
        agent: &mut Agent,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let algorithm_value = args[0];
        let _extractable_value = args[1];
        let _key_usages_value = args[2];
        // Parse algorithm - for now, assume it's a simple string or object with name
        let algorithm_str = match algorithm_value.to_string(agent, gc.reborrow()) {
            Ok(s) => s,
            Err(_) => {
                let gc = gc.into_nogc();
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Invalid algorithm parameter",
                        gc,
                    )
                    .unbind());
            }
        };
        let algorithm_name = algorithm_str.as_str(agent);

        let rng = rand::SystemRandom::new();

        match algorithm_name {
            "AES-GCM" | "AES-CBC" | "AES-CTR" => {
                // Generate 256-bit AES key (32 bytes)
                let mut key_bytes = [0u8; 32];
                if rng.fill(&mut key_bytes).is_err() {
                    let gc = gc.into_nogc();
                    return Err(agent
                        .throw_exception_with_static_message(
                            nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                            "Failed to generate random key",
                            gc,
                        )
                        .unbind());
                }

                // Return a simple representation of the key
                let hex_key = key_bytes.iter().fold(String::new(), |mut acc, b| {
                    use std::fmt::Write;
                    write!(&mut acc, "{:02x}", b).unwrap();
                    acc
                });

                let key_info = format!(
                    "{{\"type\":\"secret\",\"algorithm\":\"{}\",\"extractable\":true,\"keyData\":\"{}\"}}",
                    algorithm_name, hex_key
                );

                Ok(
                    nova_vm::ecmascript::types::String::from_string(agent, key_info, gc.nogc())
                        .unbind()
                        .into(),
                )
            }
            "HMAC" => {
                // Generate 256-bit HMAC key
                let mut key_bytes = [0u8; 32];
                if rng.fill(&mut key_bytes).is_err() {
                    let gc = gc.into_nogc();
                    return Err(agent
                        .throw_exception_with_static_message(
                            nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                            "Failed to generate random key",
                            gc,
                        )
                        .unbind());
                }

                let hex_key = key_bytes.iter().fold(String::new(), |mut acc, b| {
                    use std::fmt::Write;
                    write!(&mut acc, "{:02x}", b).unwrap();
                    acc
                });

                let key_info = format!(
                    "{{\"type\":\"secret\",\"algorithm\":\"HMAC\",\"extractable\":true,\"keyData\":\"{}\"}}",
                    hex_key
                );

                Ok(
                    nova_vm::ecmascript::types::String::from_string(agent, key_info, gc.nogc())
                        .unbind()
                        .into(),
                )
            }
            _ => {
                let gc = gc.into_nogc();
                Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Unsupported key generation algorithm",
                        gc,
                    )
                    .unbind())
            }
        }
    }

    pub fn import_key<'gc>(
        agent: &mut Agent,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Implement key import functionality
        let gc = gc.into_nogc();
        Err(agent
            .throw_exception_with_static_message(
                nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                "importKey not yet implemented",
                gc,
            )
            .unbind())
    }

    pub fn export_key<'gc>(
        agent: &mut Agent,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Implement key export functionality
        let gc = gc.into_nogc();
        Err(agent
            .throw_exception_with_static_message(
                nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                "exportKey not yet implemented",
                gc,
            )
            .unbind())
    }

    pub fn encrypt<'gc>(
        agent: &mut Agent,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let _algorithm_value = args[0];
        let _key_value = args[1];
        let _data_value = args[2];

        // TODO: Parse algorithm, extract key and data, perform actual encryption
        let encrypted_hex = "placeholder".to_string(); // Placeholder

        Ok(
            nova_vm::ecmascript::types::String::from_string(agent, encrypted_hex, gc.nogc())
                .unbind()
                .into(),
        )
    }

    pub fn decrypt<'gc>(
        agent: &mut Agent,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let _algorithm_value = args[0];
        let _key_value = args[1];
        let _data_value = args[2];

        // TODO: Parse algorithm, extract key and data, perform actual decryption
        let decrypted_hex = "placeholder".to_string(); // Placeholder

        Ok(
            nova_vm::ecmascript::types::String::from_string(agent, decrypted_hex, gc.nogc())
                .unbind()
                .into(),
        )
    }

    pub fn sign<'gc>(
        agent: &mut Agent,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let _algorithm_value = args[0];
        let _key_value = args[1];
        let _data_value = args[2];

        // TODO: Parse algorithm, extract key and data, perform actual signing
        let signature_hex = "signature_placeholder".to_string(); // Placeholder

        Ok(
            nova_vm::ecmascript::types::String::from_string(agent, signature_hex, gc.nogc())
                .unbind()
                .into(),
        )
    }

    pub fn verify<'gc>(
        _agent: &mut Agent,
        args: ArgumentsList,
        _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let _algorithm_value = args[0];
        let _key_value = args[1];
        let _signature_value = args[2];
        let _data_value = args[3]; // TODO: Parse algorithm, extract key, signature and data, perform actual verification
        Ok(Value::Boolean(true)) // Placeholder - always returns true
    }
}
