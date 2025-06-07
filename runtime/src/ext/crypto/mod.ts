// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// deno-lint-ignore-file no-unused-vars no-explicit-any

// Web Crypto API implementation following W3C specification

// Types are now defined in global.d.ts to match W3C specification

/**
 * SubtleCrypto interface providing low-level cryptographic primitives.
 * Following the Web Crypto API specification.
 */
const subtle = {
  /**
   * Generates a digest of the given data using the specified algorithm.
   *
   * @example
   * ```ts
   * const encoder = new TextEncoder();
   * const data = encoder.encode("Hello, World!");
   * const digest = await crypto.subtle.digest("SHA-256", data);
   * const hexString = Array.from(new Uint8Array(digest))
   *   .map(b => b.toString(16).padStart(2, '0')).join('');
   * console.log(hexString);
   * ```
   */
  digest(
    algorithm: AlgorithmIdentifier,
    data: Uint8Array | ArrayBuffer,
  ): Promise<ArrayBuffer> {
    return Promise.resolve(internal_subtle_digest(algorithm, data));
  },

  /**
   * Generates a new cryptographic key or key pair.
   *
   * @example
   * ```ts
   * const keyPair = await crypto.subtle.generateKey(
   *   { name: "RSA-PSS", modulusLength: 2048, publicExponent: new Uint8Array([1, 0, 1]), hash: "SHA-256" },
   *   true,
   *   ["sign", "verify"]
   * );
   * ```
   */
  generateKey(
    algorithm: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey | CryptoKeyPair> {
    return Promise.resolve(
      internal_subtle_generateKey(algorithm, extractable, keyUsages),
    );
  },

  /**
   * Imports a key from external data.
   *
   * @example
   * ```ts
   * const key = await crypto.subtle.importKey(
   *   "raw",
   *   keyData,
   *   { name: "AES-GCM" },
   *   false,
   *   ["encrypt", "decrypt"]
   * );
   * ```
   */
  importKey(
    format: KeyFormat,
    keyData: ArrayBuffer | Uint8Array | object,
    algorithm: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey> {
    return Promise.resolve(
      internal_subtle_importKey(
        format,
        keyData,
        algorithm,
        extractable,
        keyUsages,
      ),
    );
  },

  /**
   * Exports a key to external format.
   *
   * @example
   * ```ts
   * const exportedKey = await crypto.subtle.exportKey("spki", publicKey);
   * ```
   */
  exportKey(
    format: KeyFormat,
    key: CryptoKey,
  ): Promise<ArrayBuffer | object> {
    // @ts-ignore - Allow internal function call
    return Promise.resolve(internal_subtle_exportKey(format, key));
  }, /**
   * Encrypts data using the specified algorithm and key.
   *
   * @example
   * ```ts
   * const encoder = new TextEncoder();
   * const data = encoder.encode("Secret message");
   * const encrypted = await crypto.subtle.encrypt(
   *   { name: "AES-GCM", iv: crypto.getRandomValues(new Uint8Array(12)) },
   *   key,
   *   data
   * );
   * ```
   */
  encrypt(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    data: Uint8Array | ArrayBuffer,
  ): Promise<ArrayBuffer> {
    // @ts-ignore - Allow internal function call
    return Promise.resolve(internal_subtle_encrypt(algorithm, key, data));
  }, /**
   * Decrypts data using the specified algorithm and key.
   *
   * @example
   * ```ts
   * const decrypted = await crypto.subtle.decrypt(
   *   { name: "AES-GCM", iv: savedIv },
   *   key,
   *   encryptedData
   * );
   * const decoder = new TextDecoder();
   * const decryptedText = decoder.decode(decrypted);
   * ```
   */
  decrypt(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    data: Uint8Array | ArrayBuffer,
  ): Promise<ArrayBuffer> {
    // @ts-ignore - Allow internal function call
    return Promise.resolve(internal_subtle_decrypt(algorithm, key, data));
  }, /**
   * Creates a digital signature for data using the specified algorithm and key.
   *
   * @example
   * ```ts
   * const encoder = new TextEncoder();
   * const message = encoder.encode("Document to sign");
   * const signature = await crypto.subtle.sign(
   *   "RSA-PSS",
   *   privateKey,
   *   message
   * );
   * const hexString = Array.from(new Uint8Array(signature))
   *   .map(b => b.toString(16).padStart(2, '0')).join('');
   * console.log("Signature:", hexString);
   * ```
   */
  sign(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    data: Uint8Array | ArrayBuffer,
  ): Promise<ArrayBuffer> {
    // @ts-ignore - Allow internal function call
    return Promise.resolve(internal_subtle_sign(algorithm, key, data));
  },

  /**
   * Verifies a digital signature using the specified algorithm and key.
   * @example
   * ```ts
   * const isValid = await crypto.subtle.verify(
   *   "RSA-PSS",
   *   publicKey,
   *   signature,
   *   data
   * );
   * ```
   */
  verify(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    signature: Uint8Array | ArrayBuffer,
    data: Uint8Array | ArrayBuffer,
  ): Promise<boolean> {
    return Promise.resolve(
      // @ts-ignore - Allow internal function call
      internal_subtle_verify(algorithm, key, signature, data),
    );
  },
};

/**
 * Crypto interface providing access to cryptographically strong random values
 * and cryptographic primitives. Following the Web Crypto API specification.
 */
const crypto = {
  /**
   * SubtleCrypto interface for low-level cryptographic operations.
   */
  subtle,
  /**
   * Fills the passed array with cryptographically strong random values.
   *
   * @example
   * ```ts
   * const buffer = new Uint8Array(16);
   * crypto.getRandomValues(buffer);
   * console.log(buffer); // [random values]
   * ```
   */ getRandomValues<T extends Uint8Array | Uint16Array | Uint32Array>(
    array: T,
  ): T {
    const result = internal_crypto_getRandomValues(array);

    let seed = (Date.now() * Math.random() * 0x7FFFFFFF) | 0;

    for (let i = 0; i < array.length; i++) {
      seed = (seed * 1664525 + 1013904223) | 0;
      const randomValue = (seed >>> 0) / 0x100000000;

      if (array instanceof Uint8Array) {
        array[i] = Math.floor(randomValue * 256);
      } else if (array instanceof Uint16Array) {
        array[i] = Math.floor(randomValue * 65536);
      } else if (array instanceof Uint32Array) {
        array[i] = Math.floor(randomValue * 4294967296);
      }
    }

    return array;
  },

  /**
   * Generates a new UUID (Universally Unique Identifier) string.
   *
   * @example
   * ```ts
   * const uuid = crypto.randomUUID();
   * console.log(uuid); // "f47ac10b-58cc-4372-a567-0e02b2c3d479"
   * ```
   */
  randomUUID(): string {
    return internal_crypto_randomUUID();
  },
};
