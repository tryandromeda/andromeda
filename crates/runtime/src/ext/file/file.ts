// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * Implementation of the File API File interface
 * Based on: https://w3c.github.io/FileAPI/#file-section
 * WinterTC Compliance: https://min-common-api.proposal.wintertc.org/
 */

interface FilePropertyBag extends BlobPropertyBag {
  lastModified?: number;
}

/**
 * File represents a file-like object of immutable, raw data that implements the Blob interface
 */
class File {
  #blob: Blob;
  #name: string;
  #lastModified: number;

  constructor(
    fileBits: BlobPart[],
    fileName: string,
    options: FilePropertyBag = {},
  ) {
    // Create the underlying blob
    this.#blob = new Blob(fileBits, { type: options.type || "" });

    // Set file-specific properties
    this.#name = fileName;
    this.#lastModified = options.lastModified ?? Date.now();
  }

  /**
   * The size of the file in bytes
   */
  get size(): number {
    return this.#blob.size;
  }

  /**
   * The MIME type of the file
   */
  get type(): string {
    return this.#blob.type;
  }

  /**
   * The name of the file
   */
  get name(): string {
    return this.#name;
  }

  /**
   * The last modified timestamp of the file in milliseconds since Unix epoch
   */
  get lastModified(): number {
    return this.#lastModified;
  }

  /**
   * The last modified date as a Date object
   */
  get lastModifiedDate(): Date {
    return new Date(this.#lastModified);
  }

  /**
   * Returns a new Blob containing the data in the specified range
   */
  slice(start?: number, end?: number, contentType?: string): Blob {
    // Delegate to the underlying blob's slice method
    return this.#blob.slice(start, end, contentType);
  }

  /**
   * Returns a Promise that resolves with the contents of the blob as text
   */
  async text(): Promise<string> {
    return await this.#blob.text();
  }

  /**
   * Returns a Promise that resolves with the contents of the blob as an ArrayBuffer
   */
  async arrayBuffer(): Promise<ArrayBuffer> {
    return await this.#blob.arrayBuffer();
  }

  /**
   * Returns a ReadableStream of the blob's data
   */
  stream(): ReadableStream<Uint8Array> {
    return this.#blob.stream();
  }

  /**
   * Returns the string tag for Object.prototype.toString
   */
  get [Symbol.toStringTag]() {
    return "File";
  }
}

// @ts-ignore globalThis is not readonly
globalThis.File = File;
