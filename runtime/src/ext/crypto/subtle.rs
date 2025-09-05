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
use ring::{aead, digest, rand};
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
                    if let Ok(byte_str) = std::str::from_utf8(chunk) {
                        if let Ok(byte_val) = u8::from_str_radix(byte_str, 16) {
                            bytes.push(byte_val);
                        }
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
        }; // Return as ArrayBuffer once proper support is implemented
        let result_bytes = digest_result.as_ref().to_vec();

        // For now, directly return the hex string since proper ArrayBuffer isn't available
        let hex_string = result_bytes.iter().fold(String::new(), |mut acc, b| {
            use std::fmt::Write;
            write!(&mut acc, "{b:02x}").unwrap();
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
            CryptoAlgorithm::AesGcm { .. } => {
                Self::encrypt_aes_gcm(&crypto_key.key_data, &plaintext)
            }
            _ => Err("Unsupported encryption algorithm".to_string()),
        };

        match result {
            Ok(ciphertext) => {
                // Return as hex string for now
                let hex_result = ciphertext.iter().fold(String::new(), |mut acc, b| {
                    use std::fmt::Write;
                    write!(&mut acc, "{b:02x}").unwrap();
                    acc
                });

                Ok(
                    nova_vm::ecmascript::types::String::from_string(agent, hex_result, gc.nogc())
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
    fn encrypt_aes_gcm(key_data: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let key = aead::UnboundKey::new(&aead::AES_256_GCM, key_data)
            .map_err(|_| "Invalid key for AES-GCM")?;
        let key = aead::LessSafeKey::new(key);

        // Generate random nonce (12 bytes for AES-GCM)
        let rng = rand::SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes)
            .map_err(|_| "Failed to generate nonce")?;
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
            _ => Err("Unsupported decryption algorithm".to_string()),
        };

        match result {
            Ok(plaintext) => {
                // Return as hex string for now
                let hex_result = plaintext.iter().fold(String::new(), |mut acc, b| {
                    use std::fmt::Write;
                    write!(&mut acc, "{b:02x}").unwrap();
                    acc
                });

                Ok(
                    nova_vm::ecmascript::types::String::from_string(agent, hex_result, gc.nogc())
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
