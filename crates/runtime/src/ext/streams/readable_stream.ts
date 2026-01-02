// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// deno-lint-ignore-file no-async-promise-executor require-await

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
    // Check if this is a byte stream
    const isByteStream = underlyingSource?.type === "bytes";

    if (isByteStream) {
      // Create BYOB stream
      const autoAllocateChunkSize =
        (underlyingSource as ReadableStreamUnderlyingByteSource)
          .autoAllocateChunkSize || 1024;
      this.#streamId = __andromeda__.internal_readable_stream_create_byob(
        autoAllocateChunkSize.toString(),
      );
    } else {
      // Create regular stream
      this.#streamId = __andromeda__.internal_readable_stream_create();
    }

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

    if (isByteStream) {
      // For byte streams, we don't create a default controller here
      // The controller will be created when needed
      this.#controller = null;
    } else {
      this.#controller = new ReadableStreamDefaultController<R>(
        this.#streamId,
        this,
      );
    }

    if (underlyingSource?.start) {
      try {
        const controller = isByteStream ?
          new ReadableByteStreamController(this.#streamId, this) :
          this.#controller!;
        // deno-lint-ignore no-explicit-any
        const result = underlyingSource.start(controller as any);
        if (result instanceof Promise) {
          result.catch((error) => {
            if (isByteStream) {
              new ReadableByteStreamController(this.#streamId, this).error(
                error,
              );
            } else {
              this.#controller?.error(error);
            }
          });
        }
      } catch (error) {
        if (isByteStream) {
          new ReadableByteStreamController(this.#streamId, this).error(error);
        } else {
          this.#controller?.error(error);
        }
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

  getReader(
    options?: { mode?: "byob"; },
  ): ReadableStreamDefaultReader<R> | ReadableStreamBYOBReader {
    // According to WHATWG spec: "acquire a readable stream reader"
    if (this.locked) {
      throw new TypeError("ReadableStream is already locked");
    }

    // Lock the stream in the backend
    try {
      __andromeda__.internal_readable_stream_lock(this.#streamId);
    } catch (_) {
      throw new TypeError("Failed to lock ReadableStream");
    }

    if (options?.mode === "byob") {
      const byobReader = new ReadableStreamBYOBReader(this.#streamId, this);
      // deno-lint-ignore no-explicit-any
      this.#reader = byobReader as any;
      return byobReader;
    } else {
      this.#reader = new ReadableStreamDefaultReader<R>(this.#streamId, this);
      return this.#reader;
    }
  }

  pipeThrough<T>(
    transform: { readable: ReadableStream<T>; writable: WritableStream<R>; },
    options?: PipeOptions,
  ): ReadableStream<T> {
    const {
      preventClose = false,
      preventAbort = false,
      preventCancel = false,
      signal,
    } = options || {};

    if (signal?.aborted) {
      throw new DOMException("Aborted", "AbortError");
    }

    // Instead of using backend operations, implement pipe through manually
    // This ensures compatibility and proper functionality
    (async () => {
      try {
        const reader = this.getReader();
        const writer = transform.writable.getWriter();

        // Handle abort signal
        const abortHandler = () => {
          if (!preventCancel) {
            this.cancel(signal?.reason);
          }
          if (!preventAbort) {
            transform.writable.abort(signal?.reason);
          }
        };

        if (signal) {
          signal.addEventListener("abort", abortHandler);
        }

        // Pipe data from this stream to the transform writable
        let result;
        // deno-lint-ignore no-explicit-any
        while (!(result = await (reader as any).read()).done) {
          await writer.write(result.value);
        }

        // Close the writer unless prevented
        if (!preventClose) {
          await writer.close();
        }

        reader.releaseLock();
        writer.releaseLock();

        // Clean up abort handler
        if (signal) {
          signal.removeEventListener("abort", abortHandler);
        }
      } catch (error) {
        // Handle pipe errors
        if (!preventAbort) {
          transform.writable.abort(error);
        }
        if (!preventCancel) {
          this.cancel(error);
        }
      }
    })();

    return transform.readable;
  }

  pipeTo(
    destination: WritableStream<R>,
    options?: PipeOptions,
  ): Promise<void> {
    const {
      preventClose = false,
      preventAbort = false,
      preventCancel = false,
      signal,
    } = options || {};

    return new Promise(async (resolve, reject) => {
      if (signal?.aborted) {
        reject(new DOMException("Aborted", "AbortError"));
        return;
      }

      try {
        const reader = this.getReader();
        const writer = destination.getWriter();

        // Handle abort signal
        const abortHandler = () => {
          if (!preventCancel) {
            this.cancel(signal?.reason);
          }
          if (!preventAbort) {
            destination.abort(signal?.reason);
          }
          reject(new DOMException("Aborted", "AbortError"));
        };

        if (signal) {
          signal.addEventListener("abort", abortHandler);
        }

        let result;
        // deno-lint-ignore no-explicit-any
        while (!(result = await (reader as any).read()).done) {
          await writer.write(result.value);
        }

        if (!preventClose) {
          await writer.close();
        }

        reader.releaseLock();
        writer.releaseLock();

        if (signal) {
          signal.removeEventListener("abort", abortHandler);
        }

        resolve();
      } catch (error) {
        try {
          if (!preventAbort) {
            await destination.abort(error);
          }
          if (!preventCancel) {
            await this.cancel(error);
          }
        } catch {
          // Ignore cleanup errors
        }
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
 * ReadableByteStreamController implementation
 */
class ReadableByteStreamController {
  #streamId: string;
  #stream: ReadableStream;

  constructor(streamId: string, stream: ReadableStream) {
    this.#streamId = streamId;
    this.#stream = stream;
  }

  get byobRequest(): ReadableStreamBYOBRequest | null {
    try {
      const state = __andromeda__.internal_stream_get_state(this.#streamId);
      const [readableState] = state.split(":");

      if (readableState === "readable") {
        return new ReadableStreamBYOBRequest(this.#streamId);
      }
      return null;
    } catch {
      return null;
    }
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
        return Math.max(0, 1 - chunks);
      }

      return desiredSize;
    } catch {
      return 1;
    }
  }

  close(): void {
    __andromeda__.internal_readable_stream_close(this.#streamId);
  }

  enqueue(chunk: ArrayBufferView): void {
    if (!chunk || !chunk.buffer) {
      throw new TypeError("chunk must be an ArrayBufferView");
    }

    const bytes = new Uint8Array(
      chunk.buffer,
      chunk.byteOffset,
      chunk.byteLength,
    );
    const bytesString = Array.from(bytes).join(",");

    __andromeda__.internal_readable_stream_enqueue(this.#streamId, bytesString);
  }

  error(e?: unknown): void {
    const errorMessage = e instanceof Error ?
      e.message :
      String(e || "Byte stream error");
    __andromeda__.internal_readable_stream_error(this.#streamId, errorMessage);
  }
}

class ReadableStreamBYOBRequest {
  #streamId: string;
  #view: ArrayBufferView | null;

  constructor(streamId: string, view?: ArrayBufferView) {
    this.#streamId = streamId;
    this.#view = view || null;
  }

  get view(): ArrayBufferView | null {
    return this.#view;
  }

  respond(bytesWritten: number): void {
    if (bytesWritten < 0) {
      throw new RangeError("bytesWritten must be non-negative");
    }

    if (!this.#view) {
      throw new TypeError("Cannot respond with null view");
    }

    try {
      __andromeda__.internal_readable_stream_pull_into(
        this.#streamId,
        JSON.stringify(
          Array.from(
            new Uint8Array(
              this.#view.buffer,
              this.#view.byteOffset,
              bytesWritten,
            ),
          ),
        ),
        this.#view.byteOffset.toString(),
        bytesWritten.toString(),
      );
    } catch (error) {
      throw new Error(`Failed to respond: ${error}`);
    }
  }

  respondWithNewView(view: ArrayBufferView): void {
    if (!view || !view.buffer) {
      throw new TypeError("view must be an ArrayBufferView");
    }

    try {
      __andromeda__.internal_readable_stream_pull_into(
        this.#streamId,
        JSON.stringify(
          Array.from(
            new Uint8Array(view.buffer, view.byteOffset, view.byteLength),
          ),
        ),
        view.byteOffset.toString(),
        view.byteLength.toString(),
      );
      this.#view = view;
    } catch (error) {
      throw new Error(`Failed to respond with new view: ${error}`);
    }
  }
}

class ReadableStreamBYOBReader {
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

  async cancel(reason?: unknown): Promise<void> {
    __andromeda__.internal_readable_stream_cancel(this.#streamId);
  }

  async read<T extends ArrayBufferView>(
    view: T,
  ): Promise<ReadableStreamReadResult<T>> {
    if (!view || !view.buffer) {
      throw new TypeError("read() requires a view argument");
    }

    const getElementSize = (v: ArrayBufferView): number => {
      if ("BYTES_PER_ELEMENT" in v && typeof v.BYTES_PER_ELEMENT === "number") {
        return v.BYTES_PER_ELEMENT;
      }
      return 1;
    };

    const bufferInfo = JSON.stringify({
      byteLength: view.byteLength,
      byteOffset: view.byteOffset,
      elementSize: getElementSize(view),
    });

    const result = __andromeda__.internal_readable_stream_byob_reader_read(
      this.#streamId,
      bufferInfo,
    );

    if (result === "done") {
      return { done: true, value: view };
    }

    if (result === "error") {
      throw new Error("Stream is in error state");
    }

    try {
      const response = JSON.parse(result);
      const bytesRead = response.bytesRead;
      const data = response.data;

      if (bytesRead === 0) {
        return { done: true, value: view };
      }

      const sourceArray = new Uint8Array(data);
      const targetArray = new Uint8Array(
        view.buffer,
        view.byteOffset,
        Math.min(bytesRead, view.byteLength),
      );
      targetArray.set(sourceArray.slice(0, targetArray.length));

      const ViewConstructor = view.constructor as new(
        buffer: ArrayBufferLike,
        byteOffset?: number,
        length?: number,
      ) => T;
      const resultView = new ViewConstructor(
        view.buffer,
        view.byteOffset,
        Math.floor(bytesRead / getElementSize(view)),
      );

      return { done: false, value: resultView };
    } catch (error) {
      throw new Error(`Failed to read from BYOB reader: ${error}`);
    }
  }

  releaseLock(): void {
    try {
      __andromeda__.internal_readable_stream_unlock(this.#streamId);
      // deno-lint-ignore no-explicit-any
      (this.#stream as any)[symbolForSetReader](null);
    } catch {
      // Ignore errors when releasing lock
    }
  }
}

function createReadableStreamFrom<T>(
  asyncIterable: AsyncIterable<T> | Iterable<T>,
): ReadableStream<T> {
  if (
    asyncIterable &&
    typeof (asyncIterable as AsyncIterable<T>)[Symbol.asyncIterator] ===
      "function"
  ) {
    return new ReadableStream<T>({
      async start(controller: ReadableStreamDefaultController<T>) {
        try {
          for await (const chunk of asyncIterable as AsyncIterable<T>) {
            controller.enqueue(chunk);
          }
          controller.close();
        } catch (error) {
          controller.error(error);
        }
      },
    });
  }

  if (
    asyncIterable &&
    typeof (asyncIterable as Iterable<T>)[Symbol.iterator] === "function"
  ) {
    return new ReadableStream<T>({
      start(controller: ReadableStreamDefaultController<T>) {
        try {
          for (const chunk of asyncIterable as Iterable<T>) {
            controller.enqueue(chunk);
          }
          controller.close();
        } catch (error) {
          controller.error(error);
        }
      },
    });
  }

  throw new TypeError(
    "createReadableStreamFrom() requires an iterable or async iterable",
  );
}

// @ts-ignore globalThis is not readonly
globalThis.ReadableStream = ReadableStream;
// @ts-ignore globalThis is not readonly
globalThis.ReadableByteStreamController = ReadableByteStreamController;
// @ts-ignore globalThis is not readonly
globalThis.ReadableStreamDefaultController = ReadableStreamDefaultController;
// @ts-ignore globalThis is not readonly
globalThis.ReadableStreamDefaultReader = ReadableStreamDefaultReader;
// @ts-ignore globalThis is not readonly
globalThis.ReadableStreamBYOBReader = ReadableStreamBYOBReader;
// @ts-ignore globalThis is not readonly
globalThis.ReadableStreamBYOBRequest = ReadableStreamBYOBRequest;
