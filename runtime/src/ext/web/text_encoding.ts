// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * TextEncoder encodes strings into UTF-8 bytes.
 * Always uses UTF-8 encoding as per WHATWG spec.
 */
class TextEncoder {
  /**
   * The encoding name, always "utf-8"
   */
  readonly encoding: string = "utf-8";

  /**
   * Encodes a string into a Uint8Array of UTF-8 bytes.
   */
  encode(input: string = ""): Uint8Array {
    const bytesStr = __andromeda__.internal_text_encode(input);

    if (bytesStr === "") {
      return new Uint8Array(0);
    }

    const bytes = bytesStr.split(",").map((s) => parseInt(s.trim(), 10));
    return new Uint8Array(bytes);
  }

  /**
   * Encodes a string into an existing Uint8Array
   */
  encodeInto(
    source: string,
    destination: Uint8Array,
  ): TextEncoderEncodeIntoResult {
    if (!(destination instanceof Uint8Array)) {
      throw new TypeError("Destination must be a Uint8Array");
    }

    const destStr = Array.from(destination).join(",");

    const result = __andromeda__.internal_text_encode_into(
      source,
      destStr,
      destination.length,
    );

    const parts = result.split(":");
    const newBytesStr = parts[0];
    const read = parseInt(parts[1], 10);
    const written = parseInt(parts[2], 10);

    if (newBytesStr) {
      const newBytes = newBytesStr.split(",").map((s) =>
        parseInt(s.trim(), 10)
      );
      for (
        let i = 0;
        i < newBytes.length && i < destination.length;
        i++
      ) {
        destination[i] = newBytes[i];
      }
    }

    return { read, written };
  }
}

/**
 * TextDecoder decodes byte sequences into strings.
 * Supports multiple encodings including UTF-8, UTF-16, and ISO-8859-1.
 */
class TextDecoder {
  #encoding: string;
  #fatal: boolean;
  #ignoreBOM: boolean;

  /**
   * The encoding name
   */
  get encoding(): string {
    return this.#encoding;
  }

  /**
   * Whether to throw on decoding errors
   */
  get fatal(): boolean {
    return this.#fatal;
  }

  /**
   * Whether to ignore the byte order mark
   */
  get ignoreBOM(): boolean {
    return this.#ignoreBOM;
  }

  constructor(encoding: string = "utf-8", options: TextDecoderOptions = {}) {
    this.#encoding = encoding.toLowerCase().split("_").join("-");

    const supportedEncodings = [
      "utf-8",
      "utf8",
      "utf-16le",
      "utf-16",
      "utf-16be",
      "iso-8859-1",
      "latin1",
      "windows-1252",
    ];

    if (!supportedEncodings.includes(this.#encoding)) {
      throw new RangeError(`The encoding '${encoding}' is not supported`);
    }

    this.#fatal = options.fatal === true;
    this.#ignoreBOM = options.ignoreBOM === true;
  }

  /**
   * Decodes a byte sequence into a string
   */
  decode(
    input?: BufferSource | null,
    _options: TextDecodeOptions = {},
  ): string {
    // Convert input to bytes array
    let bytes: number[] = [];

    if (input != null) {
      if (input instanceof Uint8Array) {
        bytes = Array.from(input);
      } else if (input instanceof ArrayBuffer) {
        bytes = Array.from(new Uint8Array(input));
      } else if (ArrayBuffer.isView(input)) {
        // Handle other TypedArray views
        const uint8 = new Uint8Array(
          input.buffer,
          input.byteOffset,
          input.byteLength,
        );
        bytes = Array.from(uint8);
      } else if (typeof input === "object" && "length" in input) {
        // Handle array-like objects
        // deno-lint-ignore no-explicit-any
        const arrayLike: any = input;
        const length = Number(arrayLike.length) || 0;
        bytes = [];
        for (let i = 0; i < length; i++) {
          const value = arrayLike[i];
          bytes.push(Number(value) & 0xFF);
        }
      }
    }

    // Handle BOM (Byte Order Mark) if not ignoring
    if (!this.#ignoreBOM && bytes.length >= 3) {
      // UTF-8 BOM: EF BB BF
      if (
        this.#encoding === "utf-8" &&
        bytes[0] === 0xEF && bytes[1] === 0xBB && bytes[2] === 0xBF
      ) {
        bytes = bytes.slice(3);
      } // UTF-16LE BOM: FF FE
      else if (
        (this.#encoding === "utf-16le" ||
          this.#encoding === "utf-16") &&
        bytes.length >= 2 && bytes[0] === 0xFF && bytes[1] === 0xFE
      ) {
        bytes = bytes.slice(2);
      } // UTF-16BE BOM: FE FF
      else if (
        this.#encoding === "utf-16be" &&
        bytes.length >= 2 && bytes[0] === 0xFE && bytes[1] === 0xFF
      ) {
        bytes = bytes.slice(2);
      }
    }

    // Convert bytes to comma-separated string for native call
    const bytesStr = bytes.join(",");

    // Call native decoder
    return __andromeda__.internal_text_decode(
      bytesStr,
      this.#encoding,
      this.#fatal,
    );
  }
}

// @ts-ignore globalThis is not readonly
globalThis.TextEncoder = TextEncoder;
// @ts-ignore globalThis is not readonly
globalThis.TextDecoder = TextDecoder;
