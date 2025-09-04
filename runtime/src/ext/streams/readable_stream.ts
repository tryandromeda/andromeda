// deno-lint-ignore-file no-async-promise-executor no-unused-vars require-await
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

interface QueuingStrategy<R = unknown> {
  highWaterMark?: number;
  size?(chunk: R): number;
}

interface PipeOptions {
  preventClose?: boolean;
  preventAbort?: boolean;
  preventCancel?: boolean;
  signal?: AbortSignal;
}

interface ReadableStreamUnderlyingSource<R = unknown> {
  start?(
    controller: ReadableStreamDefaultController<R>,
  ): void | Promise<void>;
  pull?(controller: ReadableStreamDefaultController<R>): void | Promise<void>;
  cancel?(reason?: unknown): void | Promise<void>;
  type?: undefined;
}

interface ReadableStreamUnderlyingByteSource {
  start?(controller: ReadableByteStreamController): void | Promise<void>;
  pull?(controller: ReadableByteStreamController): void | Promise<void>;
  cancel?(reason?: unknown): void | Promise<void>;
  type: "bytes";
  autoAllocateChunkSize?: number;
}

interface ReadableStreamReadResult<T> {
  done: boolean;
  value: T;
}

const symbolForSetReader = Symbol("[[setReader]]");

/**
 * ReadableStreamDefaultController
 */
class ReadableStreamDefaultController<R = unknown> {
  #streamId: string;
  #stream: ReadableStream;

  constructor(streamId: string, stream: ReadableStream) {
    this.#streamId = streamId;
    this.#stream = stream;
  }

  get desiredSize(): number | null {
    try {
      const desiredSizeResult = __andromeda__.internal_stream_get_desired_size(
        this.#streamId,
      );
      const desiredSize = parseInt(desiredSizeResult, 10);

      if (isNaN(desiredSize)) {
        const state = __andromeda__.internal_stream_get_state(this.#streamId);
        const [readableState, , , chunkCount] = state.split(":");

        if (readableState === "closed" || readableState === "errored") {
          return 0;
        }

        const chunks = parseInt(chunkCount, 10) || 0;
        const highWaterMark = 1; // Default from spec
        return Math.max(0, highWaterMark - chunks);
      }

      return desiredSize;
    } catch {
      return 1;
    }
  }

  close(): void {
    __andromeda__.internal_readable_stream_close(this.#streamId);
  }

  enqueue(chunk?: R): void {
    if (chunk !== undefined) {
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

      __andromeda__.internal_readable_stream_enqueue(
        this.#streamId,
        bytesString,
      );
    }
  }

  error(e?: unknown): void {
    // According to WHATWG spec: "error a ReadableStream"
    const errorMessage = e instanceof Error ?
      e.message :
      String(e || "Stream error");
    __andromeda__.internal_readable_stream_error(this.#streamId, errorMessage);
  }
}

/**
 * ReadableStreamDefaultReader implementation
 */
class ReadableStreamDefaultReader<R = unknown> {
  #streamId: string;
  #stream: ReadableStream;
  #closed: Promise<undefined>;

  constructor(streamId: string, stream: ReadableStream) {
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

  async cancel(_reason?: unknown): Promise<void> {
    __andromeda__.internal_readable_stream_cancel(this.#streamId);
  }

  async read(): Promise<ReadableStreamReadResult<R>> {
    const result = __andromeda__.internal_readable_stream_read(this.#streamId);

    if (result === "done") {
      return { done: true, value: undefined as unknown as R };
    }

    if (result === "") {
      // No data available yet - for now return done
      // TODO: Implement waiting for data
      return { done: true, value: undefined as unknown as R };
    }

    // Convert bytes string back to appropriate type
    const bytes = result.split(",").map((b: string) => parseInt(b, 10))
      .filter((b: number) => !isNaN(b));
    const uint8Array = new Uint8Array(bytes);

    // Try to decode as text first
    try {
      const decoder = new TextDecoder();
      const text = decoder.decode(uint8Array);
      return { done: false, value: text as unknown as R };
    } catch {
      // If decoding fails, return the raw bytes
      return { done: false, value: uint8Array as unknown as R };
    }
  }

  // Synchronous read method for testing
  readSync(): ReadableStreamReadResult<R> {
    const result = __andromeda__.internal_readable_stream_read(this.#streamId);

    if (result === "done") {
      return { done: true, value: undefined as unknown as R };
    }

    if (result === "") {
      // No data available yet
      return { done: true, value: undefined as unknown as R };
    }

    // Convert bytes string back to appropriate type
    const bytes = result.split(",").map((b: string) => parseInt(b, 10))
      .filter((b: number) => !isNaN(b));
    const uint8Array = new Uint8Array(bytes);

    // Try to decode as text first
    try {
      const decoder = new TextDecoder();
      const text = decoder.decode(uint8Array);
      return { done: false, value: text as unknown as R };
    } catch {
      // If decoding fails, return the raw bytes
      return { done: false, value: uint8Array as unknown as R };
    }
  }

  releaseLock(): void {
    // According to WHATWG spec: "release a readable stream reader"
    try {
      __andromeda__.internal_readable_stream_unlock(this.#streamId);
      // Clear the reader reference in the stream
      this.#stream[symbolForSetReader](null);
    } catch {
      // Ignore errors when releasing lock
    }
  }
}

/**
 * ReadableStream
 */
class ReadableStream<R = unknown> {
  #streamId: string;
  #controller: ReadableStreamDefaultController<R> | null = null;
  #reader: ReadableStreamDefaultReader<R> | null = null;

  constructor(
    underlyingSource?:
      | ReadableStreamUnderlyingSource<R>
      | ReadableStreamUnderlyingByteSource,
    strategy?: QueuingStrategy<R>,
  ) {
    // Create stream with proper initialization
    this.#streamId = __andromeda__.internal_readable_stream_create();

    // Set up desired size based on strategy
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

    this.#controller = new ReadableStreamDefaultController<R>(
      this.#streamId,
      this,
    );

    if (underlyingSource?.start) {
      try {
        // deno-lint-ignore no-explicit-any
        const result = underlyingSource.start(this.#controller as any);
        if (result instanceof Promise) {
          result.catch((error) => this.#controller?.error(error));
        }
      } catch (error) {
        this.#controller.error(error);
      }
    }
  }

  [symbolForSetReader](reader: ReadableStreamDefaultReader<R> | null): void {
    this.#reader = reader;
  }

  get locked(): boolean {
    // According to WHATWG spec: A ReadableStream is locked if it has a reader
    try {
      const state = __andromeda__.internal_stream_get_state(this.#streamId);
      const [, , locked] = state.split(":");
      return locked === "true";
    } catch {
      return false;
    }
  }

  cancel(_reason?: unknown): Promise<void> {
    return new Promise((resolve) => {
      __andromeda__.internal_readable_stream_cancel(this.#streamId);
      resolve();
    });
  }

  getReader(): ReadableStreamDefaultReader<R> {
    // According to WHATWG spec: "acquire a readable stream reader"
    if (this.locked) {
      throw new TypeError("ReadableStream is already locked");
    }

    // Lock the stream in the backend
    try {
      __andromeda__.internal_readable_stream_lock(this.#streamId);
    } catch (error) {
      throw new TypeError("Failed to lock ReadableStream");
    }

    this.#reader = new ReadableStreamDefaultReader<R>(this.#streamId, this);
    return this.#reader;
  }

  pipeThrough<T>(
    transform: { readable: ReadableStream<T>; writable: WritableStream<R>; },
    _options?: PipeOptions,
  ): ReadableStream<T> {
    // TODO: Implement proper piping logic
    return transform.readable;
  }

  pipeTo(
    destination: WritableStream<R>,
    _options?: PipeOptions,
  ): Promise<void> {
    // TODO: Implement proper pipe-to logic
    return new Promise(async (resolve, reject) => {
      try {
        const reader = this.getReader();
        const writer = destination.getWriter();

        while (true) {
          const result = await reader.read();
          if (result.done) break;

          await writer.write(result.value);
        }

        await writer.close();
        resolve();
      } catch (error) {
        reject(error);
      }
    });
  }

  tee(): [ReadableStream<R>, ReadableStream<R>] {
    // According to WHATWG spec: "tee a ReadableStream"
    if (this.locked) {
      throw new TypeError("ReadableStream is already locked");
    }

    try {
      // Use the backend tee operation
      const result = __andromeda__.internal_readable_stream_tee(this.#streamId);
      const [stream1Id, stream2Id] = result.split(",");

      // Create two new ReadableStream instances using the existing stream IDs
      const stream1 = Object.create(ReadableStream.prototype);
      const stream2 = Object.create(ReadableStream.prototype);

      // Set up the internal state for both streams
      // deno-lint-ignore no-explicit-any
      (stream1 as any).#streamId = stream1Id;
      // deno-lint-ignore no-explicit-any
      (stream1 as any).#controller = null;
      // deno-lint-ignore no-explicit-any
      (stream1 as any).#reader = null;

      // deno-lint-ignore no-explicit-any
      (stream2 as any).#streamId = stream2Id;
      // deno-lint-ignore no-explicit-any
      (stream2 as any).#controller = null;
      // deno-lint-ignore no-explicit-any
      (stream2 as any).#reader = null;

      return [stream1 as ReadableStream<R>, stream2 as ReadableStream<R>];
    } catch {
      // Fallback: create empty streams
      const stream1 = new ReadableStream<R>();
      const stream2 = new ReadableStream<R>();
      return [stream1, stream2];
    }
  }

  [Symbol.asyncIterator](): AsyncIterator<R> {
    const reader = this.getReader();

    return {
      async next(): Promise<IteratorResult<R>> {
        const result = await reader.read();
        if (result.done) {
          return { done: true, value: undefined };
        }
        return { done: false, value: result.value };
      },

      async return(): Promise<IteratorResult<R>> {
        await reader.cancel();
        return { done: true, value: undefined };
      },
    };
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

/**
 * Stub implementations for byte stream classes
 */
class ReadableByteStreamController {
  constructor() {
    // TODO: Implement
  }

  get byobRequest(): ReadableStreamBYOBRequest | null {
    return null;
  }

  get desiredSize(): number | null {
    return 1;
  }

  close(): void {
    // TODO: Implement
  }

  enqueue(chunk: ArrayBufferView): void {
    // TODO: Implement
  }

  error(e?: unknown): void {
    // TODO: Implement
  }
}

class ReadableStreamBYOBRequest {
  constructor() {
    // TODO: Implement
  }

  get view(): ArrayBufferView | null {
    return null;
  }

  respond(bytesWritten: number): void {
    // TODO: Implement
  }

  respondWithNewView(view: ArrayBufferView): void {
    // TODO: Implement
  }
}

class ReadableStreamBYOBReader {
  constructor() {
    // TODO: Implement
  }

  get closed(): Promise<undefined> {
    return Promise.resolve(undefined);
  }

  async cancel(reason?: unknown): Promise<void> {
    // TODO: Implement
  }

  async read<T extends ArrayBufferView>(
    view?: T,
  ): Promise<ReadableStreamReadResult<T | Uint8Array>> {
    // TODO: Implement
    return { done: true, value: new Uint8Array(0) as unknown as T };
  }

  releaseLock(): void {
    // TODO: Implement
  }
}
