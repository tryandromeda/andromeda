// deno-lint-ignore-file no-unused-vars
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

interface QueuingStrategy<W = unknown> {
  highWaterMark?: number;
  size?(chunk: W): number;
}

interface WritableStreamUnderlyingSink<W = unknown> {
  start?(controller: WritableStreamDefaultController): void | Promise<void>;
  write?(
    chunk: W,
    controller: WritableStreamDefaultController,
  ): void | Promise<void>;
  close?(): void | Promise<void>;
  abort?(reason?: unknown): void | Promise<void>;
  type?: undefined;
}

const symbolForSetWriter = Symbol("[[setWriter]]");

/**
 * WritableStreamDefaultController
 */
class WritableStreamDefaultController {
  #streamId: string;
  #stream: WritableStream;

  constructor(streamId: string, stream: WritableStream) {
    this.#streamId = streamId;
    this.#stream = stream;
  }

  error(e?: unknown): void {
    // According to WHATWG spec: "error a WritableStream"
    const errorMessage = e instanceof Error ?
      e.message :
      String(e || "Stream error");
    __andromeda__.internal_writable_stream_error(this.#streamId, errorMessage);
  }
}

/**
 * WritableStreamDefaultWriter
 */
class WritableStreamDefaultWriter<W = unknown> {
  #streamId: string;
  #stream: WritableStream;
  #closed: Promise<undefined>;

  constructor(streamId: string, stream: WritableStream) {
    this.#streamId = streamId;
    this.#stream = stream;
    this.#closed = new Promise((resolve) => {
      // TODO: Properly handle closed promise resolution
      setTimeout(() => resolve(undefined), 0);
    });
  }

  get closed(): Promise<undefined> {
    return this.#closed;
  }

  get desiredSize(): number | null {
    try {
      const desiredSizeResult = __andromeda__.internal_stream_get_desired_size(
        this.#streamId,
      );
      const desiredSize = parseInt(desiredSizeResult, 10);

      if (isNaN(desiredSize)) {
        const state = __andromeda__.internal_stream_get_state(this.#streamId);
        const [, writableState, ,] = state.split(":");

        if (writableState === "closed" || writableState === "errored") {
          return null;
        }

        return 1; // Default positive desired size
      }

      return desiredSize;
    } catch {
      return 1;
    }
  }

  get ready(): Promise<undefined> {
    return new Promise((resolve) => {
      try {
        const desiredSizeResult = __andromeda__
          .internal_stream_get_desired_size(this.#streamId);
        const desiredSize = parseInt(desiredSizeResult, 10);

        if (isNaN(desiredSize) || desiredSize > 0) {
          resolve(undefined);
        } else {
          // TODO: Implement backpressure handling
          setTimeout(() => resolve(undefined), 0);
        }
      } catch {
        resolve(undefined);
      }
    });
  }

  abort(_reason?: unknown): Promise<void> {
    return new Promise((resolve) => {
      try {
        __andromeda__.internal_writable_stream_abort(this.#streamId);
      } catch {
        // Ignore errors during abort
      }
      resolve();
    });
  }

  close(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        // deno-lint-ignore no-explicit-any
        const stream = this.#stream as any;
        if (stream.close) {
          stream.close().then(() => {
            resolve();
          }).catch((error: unknown) => {
            reject(error);
          });
        } else {
          __andromeda__.internal_writable_stream_close(this.#streamId);
          resolve();
        }
      } catch (error) {
        reject(error);
      }
    });
  }

  releaseLock(): void {
    try {
      __andromeda__.internal_writable_stream_unlock(this.#streamId);
      this.#stream[symbolForSetWriter](null);
    } catch {
      // Ignore errors when releasing lock
    }
  }

  write(chunk: W): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        // deno-lint-ignore no-explicit-any
        const stream = this.#stream as any;
        if (stream._callUnderlyingWrite) {
          stream._callUnderlyingWrite(chunk).then(() => {
            this.#writeToInternalStorage(chunk).then(resolve).catch(
              reject,
            );
          }).catch(reject);
        } else {
          this.#writeToInternalStorage(chunk).then(resolve).catch(
            reject,
          );
        }
      } catch (error) {
        reject(error);
      }
    });
  }

  #writeToInternalStorage(chunk: W): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        // Convert chunk to bytes string representation
        let bytesString = "";
        if (chunk instanceof Uint8Array) {
          bytesString = Array.from(chunk).join(",");
        } else if (typeof chunk === "string") {
          const encoder = new TextEncoder();
          const bytes = encoder.encode(chunk);
          bytesString = Array.from(bytes).join(",");
        } else {
          // Try to convert to string then to bytes
          const str = String(chunk);
          const encoder = new TextEncoder();
          const bytes = encoder.encode(str);
          bytesString = Array.from(bytes).join(",");
        }

        const result = __andromeda__.internal_writable_stream_write(
          this.#streamId,
          bytesString,
        );
        if (result === "written") {
          resolve();
        } else {
          reject(new Error("Failed to write to stream"));
        }
      } catch (error) {
        reject(error);
      }
    });
  }
}

/**
 * WritableStream
 */
class WritableStream<W = unknown> {
  #streamId: string;
  #controller: WritableStreamDefaultController | null = null;
  #writer: WritableStreamDefaultWriter<W> | null = null;
  #underlyingSink: WritableStreamUnderlyingSink<W> | null = null;

  constructor(
    underlyingSink?: WritableStreamUnderlyingSink<W>,
    strategy?: QueuingStrategy<W>,
  ) {
    this.#streamId = __andromeda__.internal_writable_stream_create();
    this.#underlyingSink = underlyingSink || null;

    if (strategy?.highWaterMark !== undefined) {
      try {
        __andromeda__.internal_stream_set_desired_size(
          this.#streamId,
          strategy.highWaterMark,
        );
      } catch {
        // Ignore errors in setting desired size
      }
    }

    // Create controller
    this.#controller = new WritableStreamDefaultController(
      this.#streamId,
      this,
    );

    // Call start if provided
    if (underlyingSink?.start) {
      try {
        const result = underlyingSink.start(this.#controller);
        if (result instanceof Promise) {
          result.catch((error) => this.#controller?.error(error));
        }
      } catch (error) {
        this.#controller.error(error);
      }
    }
  }

  [symbolForSetWriter](writer: WritableStreamDefaultWriter<W> | null): void {
    this.#writer = writer;
  }

  get locked(): boolean {
    try {
      const state = __andromeda__.internal_stream_get_state(this.#streamId);
      const [, , locked] = state.split(":");
      return locked === "true";
    } catch {
      return this.#writer !== null;
    }
  }

  abort(_reason?: unknown): Promise<void> {
    return new Promise((resolve) => {
      try {
        // Call underlying sink abort if available
        if (this.#underlyingSink?.abort) {
          const result = this.#underlyingSink.abort(_reason);
          if (result instanceof Promise) {
            result.finally(() => {
              __andromeda__.internal_writable_stream_abort(this.#streamId);
              resolve();
            });
          } else {
            __andromeda__.internal_writable_stream_abort(this.#streamId);
            resolve();
          }
        } else {
          __andromeda__.internal_writable_stream_abort(this.#streamId);
          resolve();
        }
      } catch {
        __andromeda__.internal_writable_stream_abort(this.#streamId);
        resolve();
      }
    });
  }

  close(): Promise<void> {
    return new Promise((resolve) => {
      // Call underlying sink close if available
      if (this.#underlyingSink?.close) {
        try {
          const result = this.#underlyingSink.close();
          if (result instanceof Promise) {
            result.finally(() => {
              __andromeda__.internal_writable_stream_close(this.#streamId);
              resolve();
            });
          } else {
            __andromeda__.internal_writable_stream_close(this.#streamId);
            resolve();
          }
        } catch (error) {
          __andromeda__.internal_writable_stream_close(this.#streamId);
          resolve();
        }
      } else {
        __andromeda__.internal_writable_stream_close(this.#streamId);
        resolve();
      }
    });
  }

  getWriter(): WritableStreamDefaultWriter<W> {
    if (this.locked) {
      throw new TypeError("WritableStream is already locked");
    }

    try {
      __andromeda__.internal_writable_stream_lock(this.#streamId);
    } catch (error) {
      throw new TypeError("Failed to lock WritableStream");
    }

    this.#writer = new WritableStreamDefaultWriter<W>(this.#streamId, this);
    return this.#writer;
  }

  _callUnderlyingWrite(chunk: W): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.#underlyingSink?.write) {
        try {
          const result = this.#underlyingSink.write(
            chunk,
            this.#controller!,
          );
          if (result instanceof Promise) {
            result.then(resolve).catch(reject);
          } else {
            resolve();
          }
        } catch (error) {
          reject(error);
        }
      } else {
        resolve();
      }
    });
  }

  get _streamId(): string {
    return this.#streamId;
  }

  /**
   * Internal method to get the number of chunks queued in the stream
   */
  _getChunkCount(): number {
    try {
      const chunkCountResult = __andromeda__.internal_stream_get_chunk_count(
        this.#streamId,
      );
      return parseInt(chunkCountResult, 10) || 0;
    } catch {
      return 0;
    }
  }

  /**
   * Internal method to set the desired size for backpressure handling
   */
  _setDesiredSize(desiredSize: number): void {
    try {
      __andromeda__.internal_stream_set_desired_size(
        this.#streamId,
        desiredSize,
      );
    } catch {
      // Ignore errors
    }
  }
}
