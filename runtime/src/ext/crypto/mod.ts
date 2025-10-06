// deno-lint-ignore-file no-unused-vars no-explicit-any
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

const webidl = (globalThis as any).webidl;

// Enum converters for Crypto API
const KeyFormat = webidl.createEnumConverter("KeyFormat", [
  "raw",
  "pkcs8",
  "spki",
  "jwk",
]);

const KeyType = webidl.createEnumConverter("KeyType", [
  "public",
  "private",
  "secret",
]);

const KeyUsage = webidl.createEnumConverter("KeyUsage", [
  "encrypt",
  "decrypt",
  "sign",
  "verify",
  "deriveKey",
  "deriveBits",
  "wrapKey",
  "unwrapKey",
]);

// AlgorithmIdentifier union converter - can be string or object
const AlgorithmIdentifierConverter = webidl.createUnionConverter([
  {
    test: (V: any) => typeof V === "object" && V !== null,
    convert: (V: any) => V,
  },
  { test: (V: any) => typeof V === "string", convert: (V: any) => V as string },
]);

// AesGcmParams dictionary
const AesGcmParams = webidl.createDictionaryConverter("AesGcmParams", [], [
  {
    key: "name",
    converter: webidl.converters.DOMString,
    required: true,
  },
  {
    key: "iv",
    converter: webidl.converters.BufferSource,
    required: true,
  },
  {
    key: "additionalData",
    converter: webidl.converters.BufferSource,
  },
  {
    key: "tagLength",
    converter: webidl.converters.octet,
  },
]);

// AesKeyGenParams dictionary
const AesKeyGenParams = webidl.createDictionaryConverter(
  "AesKeyGenParams",
  [],
  [
    {
      key: "name",
      converter: webidl.converters.DOMString,
      required: true,
    },
    {
      key: "length",
      converter: webidl.converters["unsigned short"],
      required: true,
    },
  ],
);

// RsaHashedKeyGenParams dictionary
const RsaHashedKeyGenParams = webidl.createDictionaryConverter(
  "RsaHashedKeyGenParams",
  [],
  [
    {
      key: "name",
      converter: webidl.converters.DOMString,
      required: true,
    },
    {
      key: "modulusLength",
      converter: webidl.converters["unsigned long"],
      required: true,
    },
    {
      key: "publicExponent",
      converter: webidl.converters.BufferSource,
      required: true,
    },
    {
      key: "hash",
      converter: webidl.converters.AlgorithmIdentifier,
      required: true,
    },
  ],
);

// HmacKeyGenParams dictionary
const HmacKeyGenParams = webidl.createDictionaryConverter(
  "HmacKeyGenParams",
  [],
  [
    {
      key: "name",
      converter: webidl.converters.DOMString,
      required: true,
    },
    {
      key: "hash",
      converter: webidl.converters.AlgorithmIdentifier,
      required: true,
    },
    {
      key: "length",
      converter: webidl.converters["unsigned long"],
    },
  ],
);

// EcKeyGenParams dictionary
const EcKeyGenParams = webidl.createDictionaryConverter(
  "EcKeyGenParams",
  [],
  [
    {
      key: "name",
      converter: webidl.converters.DOMString,
      required: true,
    },
    {
      key: "namedCurve",
      converter: webidl.converters.DOMString,
      required: true,
    },
  ],
);
// Internal slots for CryptoKey (using Symbols for privacy)
const _type = Symbol("[[type]]");
const _extractable = Symbol("[[extractable]]");
const _algorithm = Symbol("[[algorithm]]");
const _usages = Symbol("[[usages]]");
const _handle = Symbol("[[handle]]");

/**
 * CryptoKey interface representing a cryptographic key.
 * Following the Web Crypto API specification with internal slots.
 */
class CryptoKey {
  [_type]: any;
  [_extractable]: any;
  [_algorithm]: any;
  [_usages]: any;
  [_handle]: any;

  constructor() {
    webidl.illegalConstructor();
  }

  /** The type of the key (public, private, or secret) */
  get type(): string {
    webidl.assertBranded(this, CryptoKeyPrototype);
    return this[_type];
  }

  /** Whether the key can be extracted */
  get extractable(): boolean {
    webidl.assertBranded(this, CryptoKeyPrototype);
    return this[_extractable];
  }

  /** The algorithm used with this key */
  get algorithm(): object {
    webidl.assertBranded(this, CryptoKeyPrototype);
    return this[_algorithm];
  }

  /** The allowed usages for this key */
  get usages(): string[] {
    webidl.assertBranded(this, CryptoKeyPrototype);
    return this[_usages];
  }
}

const CryptoKeyPrototype = CryptoKey.prototype;
webidl.configureInterface(CryptoKey);

// CryptoKey converter for WebIDL
const CryptoKeyConverter = webidl.createInterfaceConverter(
  "CryptoKey",
  CryptoKeyPrototype,
);

// Helper: Normalize algorithm identifier to algorithm object
function normalizeAlgorithm(
  algorithm: AlgorithmIdentifier,
  operation: string,
): object {
  // If it's a string, convert to object with name property
  if (typeof algorithm === "string") {
    return { name: algorithm };
  }

  // If it's already an object, ensure it has a name property
  if (typeof algorithm === "object" && algorithm !== null) {
    const alg = algorithm as unknown as Record<string, unknown>;
    if (!alg.name || typeof alg.name !== "string") {
      throw new TypeError(
        `Algorithm: name: Missing or not a string`,
      );
    }
    // Return a copy to avoid mutation
    return { ...alg };
  }

  throw new TypeError("Algorithm: AlgorithmIdentifier: not a string or object");
}

// Helper: Validate key usages
function validateKeyUsages(
  usages: KeyUsage[],
  validUsages: KeyUsage[],
): void {
  for (const usage of usages) {
    if (!validUsages.includes(usage)) {
      throw new DOMException(
        `Unsupported key usage: ${usage}`,
        "SyntaxError",
      );
    }
  }
}

// Helper: Get usage intersection
function usageIntersection(
  a: KeyUsage[],
  b: KeyUsage[],
): KeyUsage[] {
  return a.filter((usage) => b.includes(usage));
}

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
  async digest(
    algorithm: AlgorithmIdentifier,
    data: Uint8Array | ArrayBuffer,
  ): Promise<ArrayBuffer> {
    const prefix = "Failed to execute 'digest' on 'SubtleCrypto'";
    webidl.requiredArguments(arguments.length, 2, prefix);

    algorithm = AlgorithmIdentifierConverter(
      algorithm,
      prefix,
      "Argument 1",
    );
    data = webidl.converters.BufferSource(
      data,
      prefix,
      "Argument 2",
    );

    // Normalize algorithm
    const normalizedAlgorithm = normalizeAlgorithm(algorithm, "digest");

    // Extract algorithm name for Rust FFI (Rust expects a simple string)
    const algorithmName =
      typeof normalizedAlgorithm === "object" && normalizedAlgorithm !== null ?
        (normalizedAlgorithm as any).name :
        String(normalizedAlgorithm);

    const result = __andromeda__.internal_subtle_digest(algorithmName, data);

    // Convert base64 result to ArrayBuffer
    if (typeof result === "string") {
      // Decode base64 to binary string
      const binaryString = atob(result);
      // Convert binary string to ArrayBuffer
      const bytes = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }
      return bytes.buffer;
    }

    return result;
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
  async generateKey(
    algorithm: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey | CryptoKeyPair> {
    const prefix = "Failed to execute 'generateKey' on 'SubtleCrypto'";

    // Test converters one by one
    webidl.requiredArguments(3, 3, prefix);

    algorithm = AlgorithmIdentifierConverter(
      algorithm,
      prefix,
      "Argument 1",
    );

    extractable = webidl.converters.boolean(extractable);
    keyUsages = webidl.createSequenceConverter(KeyUsage)(
      keyUsages,
      prefix,
      "Argument 3",
    );

    // Normalize algorithm
    const normalizedAlgorithm = normalizeAlgorithm(algorithm, "generateKey");

    // Extract algorithm name for Rust FFI (Rust expects a simple string)
    const algorithmName =
      typeof normalizedAlgorithm === "object" && normalizedAlgorithm !== null ?
        (normalizedAlgorithm as any).name :
        String(normalizedAlgorithm);

    const result = __andromeda__.internal_subtle_generateKey(
      algorithmName,
      extractable,
      keyUsages,
    );

    // Parse the result - Rust returns a JSON string
    if (typeof result === "string") {
      const keyData = JSON.parse(result);
      return keyData; // Return the plain object for now
    }

    return result;
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
  async importKey(
    format: KeyFormat,
    keyData: ArrayBuffer | Uint8Array | object,
    algorithm: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey> {
    const prefix = "Failed to execute 'importKey' on 'SubtleCrypto'";
    webidl.requiredArguments(arguments.length, 5, prefix);

    format = KeyFormat(format, prefix, "Argument 1");

    // For jwk format, keyData should be object; otherwise BufferSource
    if (format === "jwk") {
      keyData = webidl.converters.object(keyData, prefix, "Argument 2");
    } else {
      keyData = webidl.converters.BufferSource(keyData, prefix, "Argument 2");
    }

    algorithm = AlgorithmIdentifierConverter(
      algorithm,
      prefix,
      "Argument 3",
    );
    extractable = webidl.converters.boolean(extractable);
    keyUsages = webidl.createSequenceConverter(KeyUsage)(
      keyUsages,
      prefix,
      "Argument 5",
    );

    // Normalize algorithm
    const normalizedAlgorithm = normalizeAlgorithm(algorithm, "importKey");

    const result = __andromeda__.internal_subtle_importKey(
      format,
      keyData,
      normalizedAlgorithm,
      extractable,
      keyUsages,
    );

    return result;
  },

  /**
   * Exports a key to external format.
   *
   * @example
   * ```ts
   * const exportedKey = await crypto.subtle.exportKey("spki", publicKey);
   * ```
   */
  async exportKey(
    format: KeyFormat,
    key: CryptoKey,
  ): Promise<ArrayBuffer | object> {
    const prefix = "Failed to execute 'exportKey' on 'SubtleCrypto'";
    webidl.requiredArguments(arguments.length, 2, prefix);

    format = KeyFormat(format, prefix, "Argument 1");
    key = CryptoKeyConverter(key, prefix, "Argument 2");

    const result = __andromeda__.internal_subtle_exportKey(format, key);

    return result;
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
  async encrypt(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    data: Uint8Array | ArrayBuffer,
  ): Promise<ArrayBuffer> {
    const prefix = "Failed to execute 'encrypt' on 'SubtleCrypto'";
    webidl.requiredArguments(arguments.length, 3, prefix);

    algorithm = AlgorithmIdentifierConverter(
      algorithm,
      prefix,
      "Argument 1",
    );
    key = CryptoKeyConverter(key, prefix, "Argument 2");
    data = webidl.converters.BufferSource(
      data,
      prefix,
      "Argument 3",
    );

    // Normalize algorithm
    const normalizedAlgorithm = normalizeAlgorithm(algorithm, "encrypt");

    const result = __andromeda__.internal_subtle_encrypt(
      normalizedAlgorithm,
      key,
      data,
    );

    return result;
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
  async decrypt(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    data: Uint8Array | ArrayBuffer,
  ): Promise<ArrayBuffer> {
    const prefix = "Failed to execute 'decrypt' on 'SubtleCrypto'";
    webidl.requiredArguments(arguments.length, 3, prefix);

    algorithm = AlgorithmIdentifierConverter(
      algorithm,
      prefix,
      "Argument 1",
    );
    key = CryptoKeyConverter(key, prefix, "Argument 2");
    data = webidl.converters.BufferSource(
      data,
      prefix,
      "Argument 3",
    );

    // Normalize algorithm
    const normalizedAlgorithm = normalizeAlgorithm(algorithm, "decrypt");

    const result = __andromeda__.internal_subtle_decrypt(
      normalizedAlgorithm,
      key,
      data,
    );

    return result;
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
  async sign(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    data: Uint8Array | ArrayBuffer,
  ): Promise<ArrayBuffer> {
    const prefix = "Failed to execute 'sign' on 'SubtleCrypto'";
    webidl.requiredArguments(arguments.length, 3, prefix);

    algorithm = AlgorithmIdentifierConverter(
      algorithm,
      prefix,
      "Argument 1",
    );
    key = CryptoKeyConverter(key, prefix, "Argument 2");
    data = webidl.converters.BufferSource(
      data,
      prefix,
      "Argument 3",
    );

    // Normalize algorithm
    const normalizedAlgorithm = normalizeAlgorithm(algorithm, "sign");

    const result = __andromeda__.internal_subtle_sign(
      normalizedAlgorithm,
      key,
      data,
    );

    return result;
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
  async verify(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    signature: Uint8Array | ArrayBuffer,
    data: Uint8Array | ArrayBuffer,
  ): Promise<boolean> {
    const prefix = "Failed to execute 'verify' on 'SubtleCrypto'";
    webidl.requiredArguments(arguments.length, 4, prefix);

    algorithm = AlgorithmIdentifierConverter(
      algorithm,
      prefix,
      "Argument 1",
    );
    key = CryptoKeyConverter(key, prefix, "Argument 2");
    signature = webidl.converters.BufferSource(
      signature,
      prefix,
      "Argument 3",
    );
    data = webidl.converters.BufferSource(
      data,
      prefix,
      "Argument 4",
    );

    // Normalize algorithm
    const normalizedAlgorithm = normalizeAlgorithm(algorithm, "verify");

    const result = __andromeda__.internal_subtle_verify(
      normalizedAlgorithm,
      key,
      signature,
      data,
    );

    return result;
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
  async deriveKey(
    algorithm: AlgorithmIdentifier,
    baseKey: CryptoKey,
    derivedKeyType: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey> {
    const prefix = "Failed to execute 'deriveKey' on 'SubtleCrypto'";
    webidl.requiredArguments(arguments.length, 5, prefix);

    algorithm = AlgorithmIdentifierConverter(
      algorithm,
      prefix,
      "Argument 1",
    );
    baseKey = CryptoKeyConverter(baseKey, prefix, "Argument 2");
    derivedKeyType = AlgorithmIdentifierConverter(
      derivedKeyType,
      prefix,
      "Argument 3",
    );
    extractable = webidl.converters.boolean(extractable);
    keyUsages = webidl.createSequenceConverter(KeyUsage)(
      keyUsages,
      prefix,
      "Argument 5",
    );

    // Normalize algorithms
    const normalizedAlgorithm = normalizeAlgorithm(algorithm, "deriveKey");
    const normalizedDerivedKeyType = normalizeAlgorithm(
      derivedKeyType,
      "get key length",
    );

    const result = __andromeda__.internal_subtle_deriveKey(
      normalizedAlgorithm,
      baseKey,
      normalizedDerivedKeyType,
      extractable,
      keyUsages,
    );

    return result;
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
  async deriveBits(
    algorithm: AlgorithmIdentifier,
    baseKey: CryptoKey,
    length?: number,
  ): Promise<ArrayBuffer> {
    const prefix = "Failed to execute 'deriveBits' on 'SubtleCrypto'";
    webidl.requiredArguments(arguments.length, 2, prefix);

    algorithm = AlgorithmIdentifierConverter(
      algorithm,
      prefix,
      "Argument 1",
    );
    baseKey = CryptoKeyConverter(baseKey, prefix, "Argument 2");

    if (length !== undefined) {
      length = webidl.converters["unsigned long"](
        length,
        prefix,
        "Argument 3",
      );
    }

    // Normalize algorithm
    const normalizedAlgorithm = normalizeAlgorithm(algorithm, "deriveBits");

    const result = __andromeda__.internal_subtle_deriveBits(
      normalizedAlgorithm,
      baseKey,
      length,
    );

    return result;
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
  async wrapKey(
    format: KeyFormat,
    key: CryptoKey,
    wrappingKey: CryptoKey,
    wrapAlgorithm: AlgorithmIdentifier,
  ): Promise<ArrayBuffer> {
    const prefix = "Failed to execute 'wrapKey' on 'SubtleCrypto'";
    webidl.requiredArguments(arguments.length, 4, prefix);

    format = KeyFormat(format, prefix, "Argument 1");
    key = CryptoKeyConverter(key, prefix, "Argument 2");
    wrappingKey = CryptoKeyConverter(wrappingKey, prefix, "Argument 3");
    wrapAlgorithm = AlgorithmIdentifierConverter(
      wrapAlgorithm,
      prefix,
      "Argument 4",
    );

    // Normalize algorithm
    const normalizedAlgorithm = normalizeAlgorithm(wrapAlgorithm, "wrapKey");

    const result = __andromeda__.internal_subtle_wrapKey(
      format,
      key,
      wrappingKey,
      normalizedAlgorithm,
    );

    return result;
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
  async unwrapKey(
    format: KeyFormat,
    wrappedKey: ArrayBuffer | Uint8Array,
    unwrappingKey: CryptoKey,
    unwrapAlgorithm: AlgorithmIdentifier,
    unwrappedKeyAlgorithm: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey> {
    const prefix = "Failed to execute 'unwrapKey' on 'SubtleCrypto'";
    webidl.requiredArguments(arguments.length, 7, prefix);

    format = KeyFormat(format, prefix, "Argument 1");
    wrappedKey = webidl.converters.BufferSource(
      wrappedKey,
      prefix,
      "Argument 2",
    );
    unwrappingKey = CryptoKeyConverter(unwrappingKey, prefix, "Argument 3");
    unwrapAlgorithm = AlgorithmIdentifierConverter(
      unwrapAlgorithm,
      prefix,
      "Argument 4",
    );
    unwrappedKeyAlgorithm = AlgorithmIdentifierConverter(
      unwrappedKeyAlgorithm,
      prefix,
      "Argument 5",
    );
    extractable = webidl.converters.boolean(extractable);
    keyUsages = webidl.createSequenceConverter(KeyUsage)(
      keyUsages,
      prefix,
      "Argument 7",
    );

    // Normalize algorithms
    const normalizedUnwrapAlgorithm = normalizeAlgorithm(
      unwrapAlgorithm,
      "unwrapKey",
    );
    const normalizedUnwrappedKeyAlgorithm = normalizeAlgorithm(
      unwrappedKeyAlgorithm,
      "get key length",
    );

    const result = __andromeda__.internal_subtle_unwrapKey(
      format,
      wrappedKey,
      unwrappingKey,
      normalizedUnwrapAlgorithm,
      normalizedUnwrappedKeyAlgorithm,
      extractable,
      keyUsages,
    );

    return result;
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
// @ts-ignore globalThis is not readonly
globalThis.CryptoKey = CryptoKey;
