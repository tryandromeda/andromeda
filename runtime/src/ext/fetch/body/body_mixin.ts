// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// deno-lint-ignore-file no-explicit-any

const consumeArrayBuffer = (globalThis as any).consumeArrayBuffer;
const consumeBlob = (globalThis as any).consumeBlob;
const consumeBytes = (globalThis as any).consumeBytes;
const consumeFormData = (globalThis as any).consumeFormData;
const consumeJson = (globalThis as any).consumeJson;
const consumeText = (globalThis as any).consumeText;
const _InnerBody = (globalThis as any).InnerBody;

const BODY_SYMBOL = Symbol("body");
const CONTENT_TYPE_SYMBOL = Symbol("contentType");

/**
 * The Body mixin provides methods for reading body content.
 * This is implemented by both Request and Response.
 * 
 * @see https://fetch.spec.whatwg.org/#body-mixin
 */
interface Body {
  /**
   * A boolean indicating whether the body has been read.
   * @see https://fetch.spec.whatwg.org/#dom-body-bodyused
   */
  readonly bodyUsed: boolean;

  /**
   * Returns a promise that resolves with an ArrayBuffer.
   * @see https://fetch.spec.whatwg.org/#dom-body-arraybuffer
   */
  arrayBuffer(): Promise<ArrayBuffer>;

  /**
   * Returns a promise that resolves with a Blob.
   * @see https://fetch.spec.whatwg.org/#dom-body-blob
   */
  blob(): Promise<Blob>;

  /**
   * Returns a promise that resolves with a Uint8Array.
   * This is a non-standard extension for easier access to raw bytes.
   */
  bytes(): Promise<Uint8Array>;

  /**
   * Returns a promise that resolves with FormData.
   * @see https://fetch.spec.whatwg.org/#dom-body-formdata
   */
  formData(): Promise<FormData>;

  /**
   * Returns a promise that resolves with the result of parsing JSON.
   * @see https://fetch.spec.whatwg.org/#dom-body-json
   */
  json<T = unknown>(): Promise<T>;

  /**
   * Returns a promise that resolves with text.
   * @see https://fetch.spec.whatwg.org/#dom-body-text
   */
  text(): Promise<string>;
}

/**
 * Helper class that implements the Body mixin methods.
 * This can be used by Request and Response to provide body functionality.
 */
class BodyMixin implements Body {
  constructor(body: any, contentType?: string | null) {
    (this as any)[BODY_SYMBOL] = body;
    (this as any)[CONTENT_TYPE_SYMBOL] = contentType || null;
  }

  /**
   * Checks if the body has been used (consumed or locked).
   * @see https://fetch.spec.whatwg.org/#dom-body-bodyused
   */
  get bodyUsed(): boolean {
    if (!(this as any)[BODY_SYMBOL]) {
      return false;
    }
    return (this as any)[BODY_SYMBOL].unusable();
  }

  /**
   * Gets the inner body (for internal use).
   */
  protected getBody(): any {
    if (!(this as any)[BODY_SYMBOL]) {
      throw new TypeError("Body is null");
    }
    if ((this as any)[BODY_SYMBOL].unusable()) {
      throw new TypeError("Body is unusable");
    }
    return (this as any)[BODY_SYMBOL];
  }

  /**
   * Returns a promise that resolves with an ArrayBuffer.
   */
  async arrayBuffer(): Promise<ArrayBuffer> {
    return await consumeArrayBuffer(this.getBody());
  }

  /**
   * Returns a promise that resolves with a Blob.
   */
  async blob(): Promise<Blob> {
    return await consumeBlob(this.getBody(), (this as any)[CONTENT_TYPE_SYMBOL]);
  }

  /**
   * Returns a promise that resolves with a Uint8Array.
   * @see https://fetch.spec.whatwg.org/#dom-body-bytes
   */
  async bytes(): Promise<Uint8Array> {
    return await consumeBytes(this.getBody());
  }

  /**
   * Returns a promise that resolves with FormData.
   */
  async formData(): Promise<FormData> {
    return await consumeFormData(this.getBody(), (this as any)[CONTENT_TYPE_SYMBOL]);
  }

  /**
   * Returns a promise that resolves with the result of parsing JSON.
   */
  async json<T = unknown>(): Promise<T> {
    return await consumeJson(this.getBody()) as Promise<T>;
  }

  /**
   * Returns a promise that resolves with text.
   */
  async text(): Promise<string> {
    return await consumeText(this.getBody());
  }
}

// Export to globalThis for use in other files
(globalThis as any).BodyMixin = BodyMixin;
(globalThis as any).BODY_SYMBOL = BODY_SYMBOL;
(globalThis as any).CONTENT_TYPE_SYMBOL = CONTENT_TYPE_SYMBOL;
