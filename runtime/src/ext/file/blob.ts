// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * Implementation of the File API Blob interface
 * Based on: https://w3c.github.io/FileAPI/#blob-section
 * WinterTC Compliance: https://min-common-api.proposal.wintertc.org/
 */

type BlobPart = BufferSource | string | Blob;

interface BlobPropertyBag {
  type?: string;
  endings?: "transparent" | "native";
}

/**
 * Utility function to convert various input types to byte array
 */
function convertBlobPartsToBytes(blobParts: BlobPart[]): number[] {
  const bytes: number[] = [];

  for (const part of blobParts) {
    if (typeof part === "string") {
      // Convert string to UTF-8 bytes
      const encoder = new TextEncoder();
      const stringBytes = encoder.encode(part);
      for (let i = 0; i < stringBytes.length; i++) {
        bytes.push(stringBytes[i]);
      }
    } else if (part instanceof Blob) {
      // Get bytes from another blob
      const partBlob = part as Blob & { _blobId: string; };
      const blobBytes = internal_blob_get_data(partBlob._blobId);
      if (blobBytes) {
        const blobByteArray = blobBytes.split(",").map((b) => parseInt(b, 10)).filter((b) =>
          !isNaN(b)
        );
        bytes.push(...blobByteArray);
      }
    } else if (part instanceof ArrayBuffer) {
      // Convert ArrayBuffer to bytes
      const view = new Uint8Array(part);
      for (let i = 0; i < view.length; i++) {
        bytes.push(view[i]);
      }
    } else if (ArrayBuffer.isView(part)) {
      // Handle TypedArray views
      const view = new Uint8Array(
        part.buffer,
        part.byteOffset,
        part.byteLength,
      );
      for (let i = 0; i < view.length; i++) {
        bytes.push(view[i]);
      }
    }
  }

  return bytes;
}

/**
 * Blob represents a file-like object of immutable, raw data
 */
class Blob {
  #blobId: string;

  constructor(
    blobParts: BlobPart[] = [],
    options: BlobPropertyBag = {},
    existingBlobId?: string,
  ) {
    if (existingBlobId) {
      // Use existing blob ID (for internal operations like slice)
      this.#blobId = existingBlobId;
    } else {
      // Normal blob creation
      const type = options.type || "";

      // Validate and normalize type
      let normalizedType = "";
      if (type) {
        // Basic MIME type validation - should be lowercase and ASCII printable
        if (
          /^[a-zA-Z0-9][a-zA-Z0-9!#$&\-\^_]*\/[a-zA-Z0-9][a-zA-Z0-9!#$&\-\^_.]*$/
            .test(type)
        ) {
          normalizedType = type.toLowerCase();
        }
      }

      // Convert blob parts to bytes
      const bytes = convertBlobPartsToBytes(blobParts);
      const bytesString = bytes.join(",");

      // Create blob through native implementation
      this.#blobId = internal_blob_create(bytesString, normalizedType);
    }
  }

  /**
   * The size of the blob in bytes
   */
  get size(): number {
    return internal_blob_get_size(this.#blobId);
  }

  /**
   * The MIME type of the blob
   */
  get type(): string {
    return internal_blob_get_type(this.#blobId);
  }

  /**
   * Returns a new Blob containing the data in the specified range
   */
  slice(start?: number, end?: number, contentType?: string): Blob {
    const actualStart = start ?? 0;
    const actualEnd = end ?? this.size;
    const actualContentType = contentType ?? "";

    const newBlobId = internal_blob_slice(
      this.#blobId,
      actualStart,
      actualEnd,
      actualContentType,
    );

    // Create a new Blob instance with the sliced blob ID
    return new Blob([], {}, newBlobId);
  }

  /**
   * Returns a ReadableStream of the blob data
   */
  stream(): ReadableStream<Uint8Array> {
    // TODO: return a proper ReadableStream
    const data = internal_blob_stream(this.#blobId);
    const bytes = data ?
      data.split(",").map((b) => parseInt(b, 10)).filter((b) => !isNaN(b)) :
      [];
    const uint8Array = new Uint8Array(bytes);

    return new ReadableStream({
      start(controller) {
        controller.enqueue(uint8Array);
        controller.close();
      },
    });
  }

  /**
   * Returns a Promise that resolves with the blob data as an ArrayBuffer
   */
  arrayBuffer(): Promise<ArrayBuffer> {
    return new Promise((resolve) => {
      const data = internal_blob_array_buffer(this.#blobId);
      const bytes = data ?
        data.split(",").map((b) => parseInt(b, 10)).filter((b) => !isNaN(b)) :
        [];

      const buffer = new ArrayBuffer(bytes.length);
      const view = new Uint8Array(buffer);
      for (let i = 0; i < bytes.length; i++) {
        view[i] = bytes[i];
      }

      resolve(buffer);
    });
  }

  /**
   * Returns a Promise that resolves with the blob data as a string
   */
  text(): Promise<string> {
    return new Promise((resolve) => {
      resolve(internal_blob_text(this.#blobId));
    });
  }

  /**
   * Returns the blob ID (internal method for File implementation)
   */
  get [Symbol.toStringTag]() {
    return "Blob";
  }

  // Internal accessor for other implementations
  get _blobId(): string {
    return this.#blobId;
  }
}
