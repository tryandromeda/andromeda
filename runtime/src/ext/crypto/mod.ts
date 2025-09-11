// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Web Crypto API implementation following W3C specification

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
    const result = __andromeda__.internal_subtle_digest(algorithm, data);

    // Convert base64 result to ArrayBuffer
    if (typeof result === "string") {
      // Decode base64 to binary string
      const binaryString = atob(result);
      // Convert binary string to ArrayBuffer
      const bytes = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }
      return Promise.resolve(bytes.buffer);
    }

    return Promise.resolve(result);
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
      __andromeda__.internal_subtle_generateKey(
        algorithm,
        extractable,
        keyUsages,
      ),
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
      __andromeda__.internal_subtle_importKey(
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
    return Promise.resolve(
      __andromeda__.internal_subtle_exportKey(format, key),
    );
  },

  /**
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
    return Promise.resolve(
      __andromeda__.internal_subtle_encrypt(algorithm, key, data),
    );
  },

  /**
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
    return Promise.resolve(
      __andromeda__.internal_subtle_decrypt(algorithm, key, data),
    );
  },

  /**
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
    return Promise.resolve(
      __andromeda__.internal_subtle_sign(algorithm, key, data),
    );
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
      __andromeda__.internal_subtle_verify(algorithm, key, signature, data),
    );
  },

  /**
   * Derives a new key from an existing key using the specified algorithm.
   *
   * @example
   * ```ts
   * const derivedKey = await crypto.subtle.deriveKey(
   *   { name: "ECDH", public: publicKey },
   *   privateKey,
   *   { name: "AES-GCM", length: 256 },
   *   false,
   *   ["encrypt", "decrypt"]
   * );
   * ```
   */
  deriveKey(
    algorithm: AlgorithmIdentifier,
    baseKey: CryptoKey,
    derivedKeyType: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey> {
    return Promise.resolve(
      __andromeda__.internal_subtle_deriveKey(
        algorithm,
        baseKey,
        derivedKeyType,
        extractable,
        keyUsages,
      ),
    );
  },

  /**
   * Derives bits from an existing key using the specified algorithm.
   *
   * @example
   * ```ts
   * const derivedBits = await crypto.subtle.deriveBits(
   *   { name: "ECDH", public: publicKey },
   *   privateKey,
   *   256
   * );
   * ```
   */
  deriveBits(
    algorithm: AlgorithmIdentifier,
    baseKey: CryptoKey,
    length?: number,
  ): Promise<ArrayBuffer> {
    return Promise.resolve(
      __andromeda__.internal_subtle_deriveBits(algorithm, baseKey, length),
    );
  },

  /**
   * Wraps a key using another key for secure transport or storage.
   *
   * @example
   * ```ts
   * const wrappedKey = await crypto.subtle.wrapKey(
   *   "raw",
   *   keyToWrap,
   *   wrappingKey,
   *   "AES-KW"
   * );
   * ```
   */
  wrapKey(
    format: KeyFormat,
    key: CryptoKey,
    wrappingKey: CryptoKey,
    wrapAlgorithm: AlgorithmIdentifier,
  ): Promise<ArrayBuffer> {
    return Promise.resolve(
      __andromeda__.internal_subtle_wrapKey(
        format,
        key,
        wrappingKey,
        wrapAlgorithm,
      ),
    );
  },

  /**
   * Unwraps a key that was previously wrapped for secure transport or storage.
   *
   * @example
   * ```ts
   * const unwrappedKey = await crypto.subtle.unwrapKey(
   *   "raw",
   *   wrappedKeyData,
   *   unwrappingKey,
   *   "AES-KW",
   *   { name: "AES-GCM" },
   *   true,
   *   ["encrypt", "decrypt"]
   * );
   * ```
   */
  unwrapKey(
    format: KeyFormat,
    wrappedKey: ArrayBuffer | Uint8Array,
    unwrappingKey: CryptoKey,
    unwrapAlgorithm: AlgorithmIdentifier,
    unwrappedKeyAlgorithm: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey> {
    return Promise.resolve(
      __andromeda__.internal_subtle_unwrapKey(
        format,
        wrappedKey,
        unwrappingKey,
        unwrapAlgorithm,
        unwrappedKeyAlgorithm,
        extractable,
        keyUsages,
      ),
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
    // Get cryptographically secure random data from Rust
    const randomDataBase64 = __andromeda__.internal_crypto_getRandomValues(
      array,
    ) as string;

    // Decode base64 to get the random bytes
    const randomBytes = atob(randomDataBase64);

    // Fill the array with the secure random data
    for (let i = 0; i < array.length; i++) {
      const byteIndex = i % randomBytes.length;

      if (array instanceof Uint8Array) {
        array[i] = randomBytes.charCodeAt(byteIndex);
      } else if (array instanceof Uint16Array) {
        const byte1 = randomBytes.charCodeAt(
          (byteIndex * 2) % randomBytes.length,
        );
        const byte2 = randomBytes.charCodeAt(
          (byteIndex * 2 + 1) % randomBytes.length,
        );
        array[i] = (byte1 << 8) | byte2;
      } else if (array instanceof Uint32Array) {
        const byte1 = randomBytes.charCodeAt(
          (byteIndex * 4) % randomBytes.length,
        );
        const byte2 = randomBytes.charCodeAt(
          (byteIndex * 4 + 1) % randomBytes.length,
        );
        const byte3 = randomBytes.charCodeAt(
          (byteIndex * 4 + 2) % randomBytes.length,
        );
        const byte4 = randomBytes.charCodeAt(
          (byteIndex * 4 + 3) % randomBytes.length,
        );
        array[i] = (byte1 << 24) | (byte2 << 16) | (byte3 << 8) | byte4;
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
    return __andromeda__.internal_crypto_randomUUID();
  },
};

// @ts-ignore globalThis is not readonly
globalThis.crypto = crypto;
