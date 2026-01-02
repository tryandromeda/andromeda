// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// deno-lint-ignore-file no-explicit-any

/**
 * Type representing a static body source (string or Uint8Array)
 */
type BodySource = string | Uint8Array | null;

const STREAM = Symbol("stream");
const SOURCE = Symbol("source");
const LENGTH = Symbol("length");

/**
 * InnerBody manages the body content for Request and Response objects.
 * It handles streaming, cloning, and consumption tracking according to the Fetch spec.
 *
 * @see https://fetch.spec.whatwg.org/#concept-body
 */
class InnerBody {
  /**
   * Creates an InnerBody from a stream or static data.
   * @param streamOrStatic - Either a ReadableStream or an object with {body, consumed}
   */
  constructor(
    streamOrStatic: ReadableStream<Uint8Array> | {
      body: Uint8Array | string;
      consumed: boolean;
    },
  ) {
    if (streamOrStatic instanceof ReadableStream) {
      (this as any)[STREAM] = streamOrStatic;
      (this as any)[SOURCE] = null;
      (this as any)[LENGTH] = null;
    } else {
      // Static body - create a ReadableStream from it
      const { body, consumed } = streamOrStatic;

      if (consumed) {
        // Already consumed - create an empty stream
        (this as any)[STREAM] = new ReadableStream({
          start(controller) {
            controller.close();
          },
        });
        (this as any)[SOURCE] = null;
      } else {
        // Create stream from static data
        const chunk = typeof body === "string" ?
          new TextEncoder().encode(body) :
          body;

        (this as any)[STREAM] = new ReadableStream({
          start(controller) {
            controller.enqueue(chunk);
            controller.close();
          },
        });

        (this as any)[SOURCE] = body;
        (this as any)[LENGTH] = chunk.byteLength;
      }
    }
  }

  /**
   * Gets the ReadableStream for this body.
   */
  get stream(): ReadableStream<Uint8Array> {
    return (this as any)[STREAM];
  }

  /**
   * Gets the original source (for cloning).
   */
  get source(): BodySource {
    return (this as any)[SOURCE];
  }

  /**
   * Gets the body length if known.
   */
  get length(): number | null {
    return (this as any)[LENGTH];
  }

  /**
   * Checks if the body is unusable (stream is locked or disturbed).
   * @see https://fetch.spec.whatwg.org/#body-unusable
   */
  unusable(): boolean {
    // A body is unusable if its stream is disturbed or locked
    return (this as any)[STREAM].locked || this.isDisturbed();
  }

  /**
   * Checks if the body has been consumed (stream is disturbed).
   */
  consumed(): boolean {
    return this.isDisturbed();
  }

  /**
   * Checks if the stream has been disturbed (read from).
   */
  private isDisturbed(): boolean {
    // A stream is disturbed if it has been read from
    // We check this by attempting to get a reader and immediately releasing it
    try {
      if ((this as any)[STREAM].locked) {
        return true;
      }
      // Note: There's no standard way to check if a stream is disturbed without reading
      // In practice, we rely on the locked state and user tracking
      return false;
    } catch {
      return true;
    }
  }

  /**
   * Consumes the entire body and returns it as a Uint8Array.
   * @see https://fetch.spec.whatwg.org/#concept-body-consume-body
   */
  async consume(): Promise<Uint8Array> {
    if (this.unusable()) {
      throw new TypeError("Body is unusable");
    }

    const reader = (this as any)[STREAM].getReader();
    const chunks: Uint8Array[] = [];
    let totalLength = 0;

    try {
      while (true) {
        const { done, value } = await reader.read();

        if (done) {
          break;
        }

        // Handle Andromeda's ReadableStream returning strings instead of Uint8Array
        let chunk: Uint8Array;
        if (typeof value === "string") {
          // Convert string back to Uint8Array
          chunk = new TextEncoder().encode(value);
        } else if (value instanceof Uint8Array) {
          chunk = value;
        } else {
          // Fallback: try to convert to string then to bytes
          chunk = new TextEncoder().encode(String(value));
        }

        chunks.push(chunk);
        totalLength += chunk.byteLength;
      }

      // Combine all chunks into a single Uint8Array
      const result = new Uint8Array(totalLength);
      let offset = 0;
      for (const chunk of chunks) {
        result.set(chunk, offset);
        offset += chunk.byteLength;
      }

      return result;
    } finally {
      reader.releaseLock();
    }
  }

  /**
   * Cancels the stream with an optional reason.
   */
  async cancel(error?: unknown): Promise<void> {
    try {
      await (this as any)[STREAM].cancel(error);
    } catch {
      // Ignore errors during cancellation
    }
  }

  /**
   * Errors the stream.
   */
  error(error: unknown): void {
    // To error a stream, we need to get the controller
    // Since we can't access it directly, we cancel the stream
    this.cancel(error).catch(() => {
      // Ignore cancellation errors
    });
  }

  /**
   * Clones the body by teeing the stream.
   * @see https://fetch.spec.whatwg.org/#concept-body-clone
   */
  clone(): InnerBody {
    if (this.unusable()) {
      throw new TypeError("Cannot clone an unusable body");
    }

    // If we have a source, create a new body from it
    if ((this as any)[SOURCE] !== null) {
      const clonedBody = new InnerBody({
        body: (this as any)[SOURCE],
        consumed: false,
      });
      return clonedBody;
    }

    // Tee the stream to create two independent streams
    const [stream1, stream2] = (this as any)[STREAM].tee();

    // Update this body's stream to stream1
    (this as any)[STREAM] = stream1;

    // Create a new body with stream2
    const clonedBody = Object.create(InnerBody.prototype);
    (clonedBody as any)[STREAM] = stream2;
    (clonedBody as any)[SOURCE] = null;
    (clonedBody as any)[LENGTH] = (this as any)[LENGTH];

    return clonedBody;
  }

  /**
   * Creates a proxy (clone) of this body.
   * This is used when a Request is passed to another Request constructor.
   */
  createProxy(): InnerBody {
    return this.clone();
  }

  /**
   * Creates an InnerBody from a source value.
   */
  static from(
    source: BodySource | ReadableStream<Uint8Array> | InnerBody,
  ): InnerBody {
    if (source instanceof InnerBody) {
      return source;
    }

    if (source instanceof ReadableStream) {
      const body = Object.create(InnerBody.prototype);
      (body as any)[STREAM] = source;
      (body as any)[SOURCE] = null;
      (body as any)[LENGTH] = null;
      return body;
    }

    if (source === null) {
      // Empty body
      return new InnerBody({
        body: new Uint8Array(0),
        consumed: false,
      });
    }

    // For other types, create static body
    return new InnerBody({
      body: source,
      consumed: false,
    });
  }
}

(globalThis as any).InnerBody = InnerBody;
