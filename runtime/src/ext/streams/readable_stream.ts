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
    // TODO: Implement desiredSize - check the stream state
    // For simplicity, return 1 if the stream is not closed/errored
    const state = __andromeda__.internal_stream_get_state(this.#streamId);
    const [readable, , closed, errored] = state.split(":");

    if (closed === "true" || errored === "true") {
      return 0;
    }

    return readable === "true" ? 1 : null;
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
    // TODO: Implement proper error handling
    __andromeda__.internal_readable_stream_close(this.#streamId);
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
    // TODO: Implement proper lock release
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
    _strategy?: QueuingStrategy<R>,
  ) {
    // TODO: Implement proper stream creation
    this.#streamId = __andromeda__.internal_readable_stream_create();

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

  get locked(): boolean {
    return this.#reader !== null;
  }

  cancel(_reason?: unknown): Promise<void> {
    return new Promise((resolve) => {
      __andromeda__.internal_readable_stream_cancel(this.#streamId);
      resolve();
    });
  }

  getReader(): ReadableStreamDefaultReader<R> {
    if (this.locked) {
      throw new TypeError("ReadableStream is already locked");
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
    // TODO: Implement proper tee logic
    const stream1 = new ReadableStream<R>();
    const stream2 = new ReadableStream<R>();

    return [stream1, stream2];
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
