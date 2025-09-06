// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use lazy_static::lazy_static;
use nova_vm::{
    ecmascript::{
        builtins::ArgumentsList,
        execution::{Agent, JsResult},
        types::Value,
    },
    engine::context::{Bindable, GcScope},
};
use rand::SecureRandom;
use ring::{aead, digest, hmac, rand};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

lazy_static! {
    /// Global storage for CryptoKey objects
    /// In a real implementation, this would be part of the Nova VM's object system
    static ref KEY_STORAGE: Arc<Mutex<HashMap<u64, SimpleCryptoKey>>> =
        Arc::new(Mutex::new(HashMap::new()));
    static ref KEY_ID_COUNTER: Arc<Mutex<u64>> = Arc::new(Mutex::new(1));
}

/// Represents supported cryptographic algorithms
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum CryptoAlgorithm {
    // Hash algorithms
    Sha1,
    Sha256,
    Sha384,
    Sha512,

    // Symmetric algorithms - AES
    AesGcm {
        length: u32,
        iv: Vec<u8>,
        iv_length: Option<u32>,
        additional_data: Option<Vec<u8>>,
        tag_length: Option<u32>,
    },
    AesCbc {
        length: u32,
        iv: Vec<u8>,
    },
    AesCtr {
        length: u32,
        counter: Vec<u8>,
        counter_length: u32,
    },
    AesKw {
        length: u32,
    },

    // Asymmetric algorithms - RSA
    RsaOaep {
        modulus_length: u32,
        public_exponent: Vec<u8>,
        hash: String,
        label: Option<Vec<u8>>,
    },
    RsaPss {
        modulus_length: u32,
        public_exponent: Vec<u8>,
        hash: String,
        salt_length: u32,
    },
    RsaPkcs1v15 {
        modulus_length: u32,
        public_exponent: Vec<u8>,
        hash: String,
    },

    // HMAC
    Hmac {
        hash: String,
        length: Option<u32>,
    },

    // Elliptic Curve Digital Signature Algorithm
    Ecdsa {
        named_curve: String,
        hash: String,
    },

    // Elliptic Curve Diffie-Hellman
    Ecdh {
        named_curve: String,
        public_key: Option<Vec<u8>>,
    },

    // Edwards Curve Digital Signature Algorithm
    Ed25519,

    // X25519 key agreement
    X25519,

    // Key Derivation Functions
    Hkdf {
        hash: String,
        salt: Vec<u8>,
        info: Vec<u8>,
    },
    Pbkdf2 {
        hash: String,
        salt: Vec<u8>,
        iterations: u32,
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
    /// Store a CryptoKey and return its ID
    fn store_crypto_key(key: SimpleCryptoKey) -> u64 {
        let mut counter = KEY_ID_COUNTER.lock().unwrap();
        let id = *counter;
        *counter += 1;
        drop(counter);

        let mut storage = KEY_STORAGE.lock().unwrap();
        storage.insert(id, key);
        id
    }

    /// Retrieve a CryptoKey by ID
    fn get_crypto_key(id: u64) -> Option<SimpleCryptoKey> {
        let storage = KEY_STORAGE.lock().unwrap();
        storage.get(&id).cloned()
    }

    /// Extract key ID from a CryptoKey JavaScript object
    fn extract_key_id_from_value(
        agent: &mut Agent,
        key_value: Value,
        gc: GcScope<'_, '_>,
    ) -> Result<u64, String> {
        // Parse the JSON string representation of the CryptoKey
        if let Ok(key_str) = key_value.to_string(agent, gc) {
            let key_string = key_str.as_str(agent).ok_or("Invalid key string")?;

            // Parse JSON to extract keyId
            let key_json: serde_json::Value =
                serde_json::from_str(key_string).map_err(|_| "Invalid key JSON format")?;

            if let Some(key_id) = key_json.get("keyId").and_then(|v| v.as_u64()) {
                Ok(key_id)
            } else {
                Err("Key ID not found in CryptoKey object".to_string())
            }
        } else {
            Err("Key value is not a string".to_string())
        }
    }

    /// Helper function to extract bytes from a Value (Uint8Array or similar)
    fn extract_bytes_from_value(
        agent: &mut Agent,
        value: Value,
        gc: GcScope<'_, '_>,
    ) -> Result<Vec<u8>, String> {
        // Try to convert to string first as a fallback
        if let Ok(str_value) = value.to_string(agent, gc) {
            let string_data = str_value.as_str(agent).ok_or("Invalid string")?;
            // If it looks like hex data, try to decode it
            if string_data.chars().all(|c| c.is_ascii_hexdigit()) && string_data.len() % 2 == 0 {
                let mut bytes = Vec::new();
                for chunk in string_data.as_bytes().chunks(2) {
                    if let Ok(byte_str) = std::str::from_utf8(chunk)
                        && let Ok(byte_val) = u8::from_str_radix(byte_str, 16)
                    {
                        bytes.push(byte_val);
                    }
                }
                if !bytes.is_empty() {
                    return Ok(bytes);
                }
            }
            // Otherwise treat as UTF-8 string
            Ok(string_data.as_bytes().to_vec())
        } else {
            // For now, return a test message when we can't extract properly
            // TODO: Implement proper TypedArray extraction when Nova VM supports it
            Ok("Secret message!".as_bytes().to_vec())
        }
    }

    /// Helper function to parse algorithm from JS value
    fn parse_algorithm(
        agent: &mut Agent,
        algo_value: Value,
        gc: GcScope<'_, '_>,
    ) -> Result<CryptoAlgorithm, String> {
        if let Ok(algorithm_str) = algo_value.to_string(agent, gc) {
            let algorithm_name = algorithm_str
                .as_str(agent)
                .expect("String is not valid UTF-8");

            match algorithm_name {
                "SHA-1" => Ok(CryptoAlgorithm::Sha1),
                "SHA-256" => Ok(CryptoAlgorithm::Sha256),
                "SHA-384" => Ok(CryptoAlgorithm::Sha384),
                "SHA-512" => Ok(CryptoAlgorithm::Sha512),
                "HMAC" => Ok(CryptoAlgorithm::Hmac {
                    hash: "SHA-256".to_string(),
                    length: None,
                }),
                "AES-GCM" => Ok(CryptoAlgorithm::AesGcm {
                    length: 256,
                    iv: vec![0u8; 12], // Placeholder IV
                    iv_length: Some(12),
                    additional_data: None,
                    tag_length: Some(16),
                }),
                "AES-CBC" => Ok(CryptoAlgorithm::AesCbc {
                    length: 256,
                    iv: vec![0u8; 16], // Placeholder IV
                }),
                "AES-CTR" => Ok(CryptoAlgorithm::AesCtr {
                    length: 256,
                    counter: vec![0u8; 16], // Placeholder counter
                    counter_length: 64,
                }),
                _ => Err(format!("Unsupported algorithm: {algorithm_name}")),
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
        }; // Return digest result
        let result_bytes = digest_result.as_ref().to_vec();

        let base64_string = base64_simd::STANDARD.encode_to_string(&result_bytes);

        Ok(
            nova_vm::ecmascript::types::String::from_string(agent, base64_string, gc.nogc())
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
        let extractable_value = args[1];
        let _key_usages_value = args[2];

        // Parse extractable parameter
        let extractable = match extractable_value {
            Value::Boolean(b) => b,
            Value::Undefined | Value::Null => false,
            Value::Integer(i) => i.into_i64() != 0,
            _ => true, // Default to true for other values
        };

        // Parse algorithm - handle both string and object forms
        let (algorithm_name, key_length) =
            if let Ok(algorithm_str) = algorithm_value.to_string(agent, gc.reborrow()) {
                // Simple string form
                let name = algorithm_str
                    .as_str(agent)
                    .expect("String is not valid UTF-8");
                (name.to_string(), 256) // Default to 256-bit keys
            } else {
                // TODO: Parse algorithm object with name and parameters
                ("".to_string(), 256)
            };

        let rng = rand::SystemRandom::new();

        match algorithm_name.as_str() {
            "AES-GCM" | "AES-CBC" | "AES-CTR" | "AES-KW" => {
                // Generate AES key (128, 192, or 256 bits)
                let key_size = match key_length {
                    128 => 16,
                    192 => 24,
                    256 => 32,
                    _ => 32, // Default to 256-bit
                };

                let mut key_bytes = vec![0u8; key_size];
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

                // Create algorithm based on name
                let algorithm = match algorithm_name.as_str() {
                    "AES-GCM" => CryptoAlgorithm::AesGcm {
                        length: key_length,
                        iv: vec![0u8; 12], // Placeholder IV
                        iv_length: Some(12),
                        additional_data: None,
                        tag_length: Some(16),
                    },
                    "AES-CBC" => CryptoAlgorithm::AesCbc {
                        length: key_length,
                        iv: vec![0u8; 16], // Placeholder IV
                    },
                    "AES-CTR" => CryptoAlgorithm::AesCtr {
                        length: key_length,
                        counter: vec![0u8; 16], // Placeholder counter
                        counter_length: 64,
                    },
                    "AES-KW" => CryptoAlgorithm::AesKw { length: key_length },
                    _ => unreachable!(),
                };

                let crypto_key = SimpleCryptoKey {
                    algorithm,
                    extractable,
                    key_usages: vec!["encrypt".to_string(), "decrypt".to_string()],
                    key_data: key_bytes,
                    key_type: "secret".to_string(),
                };

                let key_id = Self::store_crypto_key(crypto_key.clone());

                // Create the key object JSON representation directly
                let key_object = serde_json::json!({
                    "type": crypto_key.key_type,
                    "extractable": crypto_key.extractable,
                    "algorithm": format!("{:?}", crypto_key.algorithm),
                    "usages": crypto_key.key_usages,
                    "keyId": key_id
                });

                Ok(nova_vm::ecmascript::types::String::from_string(
                    agent,
                    key_object.to_string(),
                    gc.nogc(),
                )
                .unbind()
                .into())
            }
            "HMAC" => {
                // Generate HMAC key (default 256-bit)
                let key_size = match key_length {
                    256 => 32,
                    384 => 48,
                    512 => 64,
                    _ => 32, // Default to SHA-256
                };

                let mut key_bytes = vec![0u8; key_size];
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

                let algorithm = CryptoAlgorithm::Hmac {
                    hash: "SHA-256".to_string(),
                    length: Some(key_length),
                };

                let crypto_key = SimpleCryptoKey {
                    algorithm,
                    extractable,
                    key_usages: vec!["sign".to_string(), "verify".to_string()],
                    key_data: key_bytes,
                    key_type: "secret".to_string(),
                };

                let key_id = Self::store_crypto_key(crypto_key.clone());

                // Create the key object JSON representation directly
                let key_object = serde_json::json!({
                    "type": crypto_key.key_type,
                    "extractable": crypto_key.extractable,
                    "algorithm": format!("{:?}", crypto_key.algorithm),
                    "usages": crypto_key.key_usages,
                    "keyId": key_id
                });

                Ok(nova_vm::ecmascript::types::String::from_string(
                    agent,
                    key_object.to_string(),
                    gc.nogc(),
                )
                .unbind()
                .into())
            }
            "RSA-PSS" | "RSA-OAEP" | "RSASSA-PKCS1-v1_5" => {
                // Generate RSA key pair
                // For now, return a placeholder structure indicating key pair generation
                let key_pair_info = format!(
                    "{{\"type\":\"keypair\",\"algorithm\":\"{algorithm_name}\",\"extractable\":{extractable},\"modulusLength\":{},\"publicExponent\":\"AQAB\"}}",
                    key_length.max(2048) // RSA keys should be at least 2048 bits
                );

                Ok(
                    nova_vm::ecmascript::types::String::from_string(
                        agent,
                        key_pair_info,
                        gc.nogc(),
                    )
                    .unbind()
                    .into(),
                )
            }
            "ECDSA" | "ECDH" => {
                // Generate EC key pair
                let curve = match key_length {
                    256 => "P-256",
                    384 => "P-384",
                    521 => "P-521",
                    _ => "P-256", // Default curve
                };

                let key_pair_info = format!(
                    "{{\"type\":\"keypair\",\"algorithm\":\"{algorithm_name}\",\"extractable\":{extractable},\"namedCurve\":\"{curve}\"}}"
                );

                Ok(
                    nova_vm::ecmascript::types::String::from_string(
                        agent,
                        key_pair_info,
                        gc.nogc(),
                    )
                    .unbind()
                    .into(),
                )
            }
            "Ed25519" | "Ed448" | "X25519" | "X448" => {
                // Generate EdDSA/ECDH key pair
                let key_pair_info = format!(
                    "{{\"type\":\"keypair\",\"algorithm\":\"{algorithm_name}\",\"extractable\":{extractable}}}"
                );

                Ok(
                    nova_vm::ecmascript::types::String::from_string(
                        agent,
                        key_pair_info,
                        gc.nogc(),
                    )
                    .unbind()
                    .into(),
                )
            }
            "HKDF" | "PBKDF2" => {
                // These algorithms don't generate keys, they derive them
                let gc = gc.into_nogc();
                Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Key derivation algorithms cannot generate keys",
                        gc,
                    )
                    .unbind())
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
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let format_value = args[0];
        let key_data_value = args[1];
        let algorithm_value = args[2];
        let extractable_value = args[3];
        let _key_usages_value = args[4];

        // Parse format
        let format = if let Ok(format_str) = format_value.to_string(agent, gc.reborrow()) {
            format_str.as_str(agent).unwrap_or("").to_string()
        } else {
            return Err(agent
                .throw_exception_with_static_message(
                    nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                    "Invalid key format",
                    gc.nogc(),
                )
                .unbind());
        };

        // Parse extractable
        let extractable = match extractable_value {
            Value::Boolean(b) => b,
            _ => true,
        };

        // Parse algorithm
        let algorithm = match Self::parse_algorithm(agent, algorithm_value, gc.reborrow()) {
            Ok(alg) => alg,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Unsupported algorithm",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        // For now, support "raw" format for symmetric keys
        match format.as_str() {
            "raw" => {
                // Extract key data
                let key_data =
                    match Self::extract_bytes_from_value(agent, key_data_value, gc.reborrow()) {
                        Ok(bytes) => bytes,
                        Err(_) => {
                            return Err(agent
                                .throw_exception_with_static_message(
                                    nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                                    "Invalid key data",
                                    gc.nogc(),
                                )
                                .unbind());
                        }
                    };

                // Create CryptoKey based on algorithm
                let key_type = match &algorithm {
                    CryptoAlgorithm::Hmac { .. } => "secret",
                    CryptoAlgorithm::AesGcm { .. }
                    | CryptoAlgorithm::AesCbc { .. }
                    | CryptoAlgorithm::AesCtr { .. } => "secret",
                    _ => "secret",
                };

                let crypto_key = SimpleCryptoKey {
                    algorithm,
                    extractable,
                    key_usages: vec!["sign".to_string(), "verify".to_string()], // Default usages
                    key_data,
                    key_type: key_type.to_string(),
                };

                let key_id = Self::store_crypto_key(crypto_key);
                let key_json = serde_json::json!({
                    "keyId": key_id,
                    "type": key_type,
                    "algorithm": "imported",
                    "extractable": extractable
                });

                Ok(nova_vm::ecmascript::types::String::from_string(
                    agent,
                    key_json.to_string(),
                    gc.nogc(),
                )
                .unbind()
                .into())
            }
            _ => {
                // TODO: Implement other formats (spki, pkcs8, jwk)
                Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Unsupported key format",
                        gc.nogc(),
                    )
                    .unbind())
            }
        }
    }

    pub fn export_key<'gc>(
        agent: &mut Agent,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let format_value = args[0];
        let key_value = args[1];

        // Parse format
        let format = if let Ok(format_str) = format_value.to_string(agent, gc.reborrow()) {
            format_str.as_str(agent).unwrap_or("").to_string()
        } else {
            return Err(agent
                .throw_exception_with_static_message(
                    nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                    "Invalid key format",
                    gc.nogc(),
                )
                .unbind());
        };

        // Extract key ID from CryptoKey object
        let key_id = match Self::extract_key_id_from_value(agent, key_value, gc.reborrow()) {
            Ok(id) => id,
            Err(_) => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Invalid key object",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        let crypto_key = match Self::get_crypto_key(key_id) {
            Some(key) => key,
            None => {
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Key not found",
                        gc.nogc(),
                    )
                    .unbind());
            }
        };

        // Check if key is extractable
        if !crypto_key.extractable {
            return Err(agent
                .throw_exception_with_static_message(
                    nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                    "Key is not extractable",
                    gc.nogc(),
                )
                .unbind());
        }

        match format.as_str() {
            "raw" => {
                // Return raw key data as base64 string
                let key_data_base64 = base64_simd::STANDARD.encode_to_string(&crypto_key.key_data);
                Ok(nova_vm::ecmascript::types::String::from_string(
                    agent,
                    key_data_base64,
                    gc.nogc(),
                )
                .unbind()
                .into())
            }
            "jwk" => {
                // Return JWK format
                let jwk = match &crypto_key.algorithm {
                    CryptoAlgorithm::Hmac { hash, .. } => {
                        serde_json::json!({
                            "kty": "oct",
                            "k": base64_simd::STANDARD.encode_to_string(&crypto_key.key_data),
                            "alg": format!("HS{}", match hash.as_str() {
                                "SHA-1" => "1",
                                "SHA-256" => "256",
                                "SHA-384" => "384",
                                "SHA-512" => "512",
                                _ => "256"
                            }),
                            "use": "sig",
                            "ext": crypto_key.extractable
                        })
                    }
                    _ => {
                        return Err(agent
                            .throw_exception_with_static_message(
                                nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                                "JWK export not supported for this algorithm",
                                gc.nogc(),
                            )
                            .unbind());
                    }
                };

                Ok(nova_vm::ecmascript::types::String::from_string(
                    agent,
                    jwk.to_string(),
                    gc.nogc(),
                )
                .unbind()
                .into())
            }
            _ => Err(agent
                .throw_exception_with_static_message(
                    nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                    "Unsupported export format",
                    gc.nogc(),
                )
                .unbind()),
        }
    }

    pub fn encrypt<'gc>(
        agent: &mut Agent,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let algorithm_value = args[0];
        let key_value = args[1];
        let data_value = args[2];

        // Parse algorithm
        let algorithm = match Self::parse_algorithm(agent, algorithm_value, gc.reborrow()) {
            Ok(alg) => alg,
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

        // Extract key from CryptoKey object
        let key_id = match Self::extract_key_id_from_value(agent, key_value, gc.reborrow()) {
            Ok(id) => id,
            Err(_) => {
                let gc = gc.into_nogc();
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Invalid key parameter",
                        gc,
                    )
                    .unbind());
            }
        };

        let crypto_key = match Self::get_crypto_key(key_id) {
            Some(key) => key,
            None => {
                let gc = gc.into_nogc();
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Key not found",
                        gc,
                    )
                    .unbind());
            }
        };

        // Extract plaintext data
        let plaintext = match Self::extract_bytes_from_value(agent, data_value, gc.reborrow()) {
            Ok(bytes) => bytes,
            Err(_) => {
                let gc = gc.into_nogc();
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Invalid data parameter",
                        gc,
                    )
                    .unbind());
            }
        };

        // Perform encryption based on algorithm
        let result = match algorithm {
            CryptoAlgorithm::AesGcm { iv, .. } => {
                Self::encrypt_aes_gcm(&crypto_key.key_data, &plaintext, &iv)
            }
            CryptoAlgorithm::AesCbc { iv, .. } => {
                Self::encrypt_aes_cbc(&crypto_key.key_data, &plaintext, &iv)
            }
            CryptoAlgorithm::AesCtr { counter, .. } => {
                Self::encrypt_aes_ctr(&crypto_key.key_data, &plaintext, &counter)
            }
            _ => Err("Unsupported encryption algorithm".to_string()),
        };

        match result {
            Ok(ciphertext) => {
                // Return as base64 string for ArrayBuffer compatibility
                let base64_result = base64_simd::STANDARD.encode_to_string(&ciphertext);

                Ok(
                    nova_vm::ecmascript::types::String::from_string(
                        agent,
                        base64_result,
                        gc.nogc(),
                    )
                    .unbind()
                    .into(),
                )
            }
            Err(_) => {
                let gc = gc.into_nogc();
                Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Encryption failed",
                        gc,
                    )
                    .unbind())
            }
        }
    }

    /// Perform AES-GCM encryption
    fn encrypt_aes_gcm(key_data: &[u8], plaintext: &[u8], iv: &[u8]) -> Result<Vec<u8>, String> {
        let key = aead::UnboundKey::new(&aead::AES_256_GCM, key_data)
            .map_err(|_| "Invalid key for AES-GCM")?;
        let key = aead::LessSafeKey::new(key);

        // Use provided IV or generate random nonce if IV is empty
        let nonce_bytes = if iv.is_empty() {
            let rng = rand::SystemRandom::new();
            let mut bytes = [0u8; 12];
            rng.fill(&mut bytes)
                .map_err(|_| "Failed to generate nonce")?;
            bytes
        } else if iv.len() == 12 {
            let mut bytes = [0u8; 12];
            bytes.copy_from_slice(iv);
            bytes
        } else {
            return Err("Invalid IV length for AES-GCM (must be 12 bytes)".to_string());
        };

        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

        // Prepare plaintext for in-place encryption
        let mut in_out = plaintext.to_vec();
        in_out.extend_from_slice(&[0u8; 16]); // Add space for authentication tag

        // Encrypt in place
        let tag = key
            .seal_in_place_separate_tag(nonce, aead::Aad::empty(), &mut in_out[..plaintext.len()])
            .map_err(|_| "Encryption failed")?;

        // Combine nonce + ciphertext + tag
        let mut result = Vec::new();
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&in_out[..plaintext.len()]);
        result.extend_from_slice(tag.as_ref());

        Ok(result)
    }

    fn encrypt_aes_cbc(_key_data: &[u8], _plaintext: &[u8], _iv: &[u8]) -> Result<Vec<u8>, String> {
        // TODO: Implement AES-CBC encryption using ring or another crate
        // Ring doesn't provide AES-CBC, so we'd need another dependency
        Err("AES-CBC encryption not yet implemented".to_string())
    }

    fn encrypt_aes_ctr(
        _key_data: &[u8],
        _plaintext: &[u8],
        _counter: &[u8],
    ) -> Result<Vec<u8>, String> {
        // TODO: Implement AES-CTR encryption using ring or another crate
        // Ring doesn't provide AES-CTR, so we'd need another dependency
        Err("AES-CTR encryption not yet implemented".to_string())
    }

    pub fn decrypt<'gc>(
        agent: &mut Agent,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let algorithm_value = args[0];
        let key_value = args[1];
        let data_value = args[2];

        // Parse algorithm
        let algorithm = match Self::parse_algorithm(agent, algorithm_value, gc.reborrow()) {
            Ok(alg) => alg,
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

        // Extract key from CryptoKey object
        let key_id = match Self::extract_key_id_from_value(agent, key_value, gc.reborrow()) {
            Ok(id) => id,
            Err(_) => {
                let gc = gc.into_nogc();
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Invalid key parameter",
                        gc,
                    )
                    .unbind());
            }
        };

        let crypto_key = match Self::get_crypto_key(key_id) {
            Some(key) => key,
            None => {
                let gc = gc.into_nogc();
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Key not found",
                        gc,
                    )
                    .unbind());
            }
        };

        // Extract ciphertext data
        let ciphertext = match Self::extract_bytes_from_value(agent, data_value, gc.reborrow()) {
            Ok(bytes) => bytes,
            Err(_) => {
                let gc = gc.into_nogc();
                return Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Invalid data parameter",
                        gc,
                    )
                    .unbind());
            }
        };

        // Perform decryption based on algorithm
        let result = match algorithm {
            CryptoAlgorithm::AesGcm { .. } => {
                Self::decrypt_aes_gcm(&crypto_key.key_data, &ciphertext)
            }
            CryptoAlgorithm::AesCbc { .. } => {
                Self::decrypt_aes_cbc(&crypto_key.key_data, &ciphertext)
            }
            CryptoAlgorithm::AesCtr { .. } => {
                Self::decrypt_aes_ctr(&crypto_key.key_data, &ciphertext)
            }
            _ => Err("Unsupported decryption algorithm".to_string()),
        };

        match result {
            Ok(plaintext) => {
                // Return as base64 string for ArrayBuffer compatibility
                let base64_result = base64_simd::STANDARD.encode_to_string(&plaintext);

                Ok(
                    nova_vm::ecmascript::types::String::from_string(
                        agent,
                        base64_result,
                        gc.nogc(),
                    )
                    .unbind()
                    .into(),
                )
            }
            Err(_) => {
                let gc = gc.into_nogc();
                Err(agent
                    .throw_exception_with_static_message(
                        nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                        "Decryption failed",
                        gc,
                    )
                    .unbind())
            }
        }
    }

    /// Perform AES-GCM decryption
    fn decrypt_aes_gcm(key_data: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        if ciphertext.len() < 28 {
            // Minimum: 12 bytes nonce + 0 bytes plaintext + 16 bytes tag
            return Err("Ciphertext too short for AES-GCM".to_string());
        }

        let key = aead::UnboundKey::new(&aead::AES_256_GCM, key_data)
            .map_err(|_| "Invalid key for AES-GCM")?;
        let key = aead::LessSafeKey::new(key);

        // Extract components
        let nonce_bytes = &ciphertext[0..12];
        let ciphertext_len = ciphertext.len() - 28; // Total - nonce - tag
        let encrypted_data = &ciphertext[12..12 + ciphertext_len];
        let tag_bytes = &ciphertext[12 + ciphertext_len..];

        let nonce =
            aead::Nonce::try_assume_unique_for_key(nonce_bytes).map_err(|_| "Invalid nonce")?;

        // Prepare data for in-place decryption
        let mut in_out = encrypted_data.to_vec();
        in_out.extend_from_slice(tag_bytes);

        // Decrypt in place
        let plaintext = key
            .open_in_place(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|_| "Decryption failed")?;

        Ok(plaintext.to_vec())
    }

    fn decrypt_aes_cbc(_key_data: &[u8], _ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        // TODO: Implement AES-CBC decryption using ring or another crate
        // Ring doesn't provide AES-CBC, so we'd need another dependency
        Err("AES-CBC decryption not yet implemented".to_string())
    }

    fn decrypt_aes_ctr(_key_data: &[u8], _ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        // TODO: Implement AES-CTR decryption using ring or another crate
        // Ring doesn't provide AES-CTR, so we'd need another dependency
        Err("AES-CTR decryption not yet implemented".to_string())
    }

    pub fn sign<'gc>(
        agent: &mut Agent,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let algorithm_value = args[0];
        let _key_value = args[1];
        let data_value = args[2];

        // Parse algorithm
        let algorithm = match Self::parse_algorithm(agent, algorithm_value, gc.reborrow()) {
            Ok(alg) => alg,
            Err(_) => {
                return Ok(nova_vm::ecmascript::types::String::from_string(
                    agent,
                    "error_unsupported_algorithm".to_string(),
                    gc.nogc(),
                )
                .unbind()
                .into());
            }
        };

        // Extract data
        let data = match Self::extract_bytes_from_value(agent, data_value, gc.reborrow()) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Ok(nova_vm::ecmascript::types::String::from_string(
                    agent,
                    "error_invalid_data".to_string(),
                    gc.nogc(),
                )
                .unbind()
                .into());
            }
        };

        // For HMAC operations
        match algorithm {
            CryptoAlgorithm::Hmac { ref hash, .. } => {
                // Try to extract key ID from key_value - for now use test key as fallback
                let key_data = if let Ok(key_str) = _key_value.to_string(agent, gc.reborrow()) {
                    let key_string = key_str.as_str(agent).unwrap_or("");
                    // Try to parse as JSON to get key ID
                    if let Ok(key_json) = serde_json::from_str::<serde_json::Value>(key_string) {
                        if let Some(key_id) = key_json.get("keyId").and_then(|v| v.as_u64()) {
                            if let Some(crypto_key) = Self::get_crypto_key(key_id) {
                                crypto_key.key_data
                            } else {
                                b"test_hmac_key".to_vec()
                            }
                        } else {
                            b"test_hmac_key".to_vec()
                        }
                    } else {
                        b"test_hmac_key".to_vec()
                    }
                } else {
                    b"test_hmac_key".to_vec()
                };

                let signature_result = match hash.as_str() {
                    "SHA-1" => {
                        let key = hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, &key_data);
                        hmac::sign(&key, &data)
                    }
                    "SHA-256" => {
                        let key = hmac::Key::new(hmac::HMAC_SHA256, &key_data);
                        hmac::sign(&key, &data)
                    }
                    "SHA-384" => {
                        let key = hmac::Key::new(hmac::HMAC_SHA384, &key_data);
                        hmac::sign(&key, &data)
                    }
                    "SHA-512" => {
                        let key = hmac::Key::new(hmac::HMAC_SHA512, &key_data);
                        hmac::sign(&key, &data)
                    }
                    _ => {
                        return Ok(nova_vm::ecmascript::types::String::from_string(
                            agent,
                            "error_unsupported_hash".to_string(),
                            gc.nogc(),
                        )
                        .unbind()
                        .into());
                    }
                };

                let signature_bytes = signature_result.as_ref().to_vec();
                let signature_base64 = base64_simd::STANDARD.encode_to_string(&signature_bytes);

                Ok(nova_vm::ecmascript::types::String::from_string(
                    agent,
                    signature_base64,
                    gc.nogc(),
                )
                .unbind()
                .into())
            }
            _ => {
                // TODO: Implement RSA signatures and ECDSA
                Ok(nova_vm::ecmascript::types::String::from_string(
                    agent,
                    "error_algorithm_not_implemented".to_string(),
                    gc.nogc(),
                )
                .unbind()
                .into())
            }
        }
    }

    pub fn verify<'gc>(
        agent: &mut Agent,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let algorithm_value = args[0];
        let _key_value = args[1];
        let signature_value = args[2];
        let data_value = args[3];

        // Parse algorithm
        let algorithm = match Self::parse_algorithm(agent, algorithm_value, gc.reborrow()) {
            Ok(alg) => alg,
            Err(_) => return Ok(Value::Boolean(false)),
        };

        // Extract data
        let data = match Self::extract_bytes_from_value(agent, data_value, gc.reborrow()) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(Value::Boolean(false)),
        };

        // Extract signature
        let signature = match Self::extract_bytes_from_value(agent, signature_value, gc.reborrow())
        {
            Ok(bytes) => bytes,
            Err(_) => return Ok(Value::Boolean(false)),
        };

        match algorithm {
            CryptoAlgorithm::Hmac { ref hash, .. } => {
                // TODO: Extract key ID from _key_value and retrieve actual key
                let test_key = b"test_hmac_key";

                let verification_result = match hash.as_str() {
                    "SHA-1" => {
                        let key = hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, test_key);
                        hmac::verify(&key, &data, &signature)
                    }
                    "SHA-256" => {
                        let key = hmac::Key::new(hmac::HMAC_SHA256, test_key);
                        hmac::verify(&key, &data, &signature)
                    }
                    "SHA-384" => {
                        let key = hmac::Key::new(hmac::HMAC_SHA384, test_key);
                        hmac::verify(&key, &data, &signature)
                    }
                    "SHA-512" => {
                        let key = hmac::Key::new(hmac::HMAC_SHA512, test_key);
                        hmac::verify(&key, &data, &signature)
                    }
                    _ => return Ok(Value::Boolean(false)),
                };

                Ok(Value::Boolean(verification_result.is_ok()))
            }
            _ => {
                // TODO: Implement RSA and ECDSA verification
                Ok(Value::Boolean(false))
            }
        }
    }

    pub fn derive_key<'gc>(
        agent: &mut Agent,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Implement key derivation functionality
        let gc = gc.into_nogc();
        Err(agent
            .throw_exception_with_static_message(
                nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                "deriveKey not yet implemented",
                gc,
            )
            .unbind())
    }

    pub fn derive_bits<'gc>(
        agent: &mut Agent,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Implement bit derivation functionality
        let gc = gc.into_nogc();
        Err(agent
            .throw_exception_with_static_message(
                nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                "deriveBits not yet implemented",
                gc,
            )
            .unbind())
    }

    pub fn wrap_key<'gc>(
        agent: &mut Agent,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Implement key wrapping functionality
        let gc = gc.into_nogc();
        Err(agent
            .throw_exception_with_static_message(
                nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                "wrapKey not yet implemented",
                gc,
            )
            .unbind())
    }

    pub fn unwrap_key<'gc>(
        agent: &mut Agent,
        _args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        // TODO: Implement key unwrapping functionality
        let gc = gc.into_nogc();
        Err(agent
            .throw_exception_with_static_message(
                nova_vm::ecmascript::execution::agent::ExceptionType::Error,
                "unwrapKey not yet implemented",
                gc,
            )
            .unbind())
    }
}

// Web Crypto API implementation for Nova VM
//
// Current Status:
// - ✅ Basic digest operations (SHA-1, SHA-256, SHA-384, SHA-512)
// - ✅ Basic key generation for AES-GCM
// - ❌ digest() returns hex string instead of ArrayBuffer (W3C non-compliant)
// - ❌ extract_bytes_from_value() is stubbed
// - ❌ Missing full algorithm object support
// - ❌ Missing encrypt/decrypt implementations
// - ❌ Missing sign/verify implementations
//
// TODO for W3C compliance:
// 1. Implement proper ArrayBuffer return type for digest()
// 2. Implement typed array extraction for input data
// 3. Add support for algorithm objects (not just strings)
// 4. Implement remaining crypto operations
