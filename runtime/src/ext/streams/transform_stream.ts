// deno-lint-ignore-file no-unused-vars
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

interface QueuingStrategy<T = unknown> {
  highWaterMark?: number;
  size?(chunk: T): number;
}

/**
 * Implementation of the Streams API TransformStream interface
 * Based on: https://streams.spec.whatwg.org/#transformstream
 * WinterTC Compliance: https://min-common-api.proposal.wintertc.org/
 */

interface Transformer<I = unknown, O = unknown> {
  start?(
    controller: TransformStreamDefaultController<O>,
  ): void | Promise<void>;
  transform?(
    chunk: I,
    controller: TransformStreamDefaultController<O>,
  ): void | Promise<void>;
  flush?(
    controller: TransformStreamDefaultController<O>,
  ): void | Promise<void>;
  readableType?: undefined;
  writableType?: undefined;
}

/**
 * TransformStreamDefaultController implementation
 */
class TransformStreamDefaultController<O = unknown> {
  #readableStreamId: string;

  constructor(readableStreamId: string) {
    this.#readableStreamId = readableStreamId;
  }

  get desiredSize(): number | null {
    try {
      const desiredSizeResult = __andromeda__.internal_stream_get_desired_size(
        this.#readableStreamId,
      );
      const desiredSize = parseInt(desiredSizeResult, 10);

      if (isNaN(desiredSize)) {
        const state = __andromeda__.internal_stream_get_state(
          this.#readableStreamId,
        );
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

  enqueue(chunk?: O): void {
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
        this.#readableStreamId,
        bytesString,
      );
    }
  }

  error(e?: unknown): void {
    const errorMessage = e instanceof Error ?
      e.message :
      String(e || "Transform stream error");
    __andromeda__.internal_readable_stream_error(
      this.#readableStreamId,
      errorMessage,
    );
  }

  terminate(): void {
    __andromeda__.internal_readable_stream_close(this.#readableStreamId);
  }
}

/**
 * TransformStream implementation
 */
class TransformStream<I = unknown, O = unknown> {
  #readable: ReadableStream<O>;
  #writable: WritableStream<I>;
  #controller: TransformStreamDefaultController<O>;

  constructor(
    transformer?: Transformer<I, O>,
    writableStrategy?: QueuingStrategy<I>,
    readableStrategy?: QueuingStrategy<O>,
  ) {
    this.#readable = new ReadableStream<O>(undefined, readableStrategy);

    // Get the readable stream ID for the controller
    // deno-lint-ignore no-explicit-any
    const readableStreamId = (this.#readable as any)._streamId;
    this.#controller = new TransformStreamDefaultController<O>(
      readableStreamId,
    );

    this.#writable = new WritableStream<I>({
      start: (controller) => {
        if (transformer?.start) {
          try {
            const result = transformer.start(this.#controller);
            if (result instanceof Promise) {
              result.catch((error) => controller.error(error));
            }
          } catch (error) {
            controller.error(error);
          }
        }
      },
      write: (chunk, controller) => {
        if (transformer?.transform) {
          try {
            const result = transformer.transform(chunk, this.#controller);
            if (result instanceof Promise) {
              result.catch((error) => controller.error(error));
            }
          } catch (error) {
            controller.error(error);
          }
        } else {
          this.#controller.enqueue(chunk as unknown as O);
        }
      },
      close: () => {
        if (transformer?.flush) {
          try {
            const result = transformer.flush(this.#controller);
            if (result instanceof Promise) {
              result.finally(() => this.#controller.terminate());
            } else {
              this.#controller.terminate();
            }
          } catch (error) {
            this.#controller.error(error);
          }
        } else {
          this.#controller.terminate();
        }
      },
      abort: () => {
        this.#controller.error(new Error("Transform stream aborted"));
      },
    }, writableStrategy);
  }

  get readable(): ReadableStream<O> {
    return this.#readable;
  }

  get writable(): WritableStream<I> {
    return this.#writable;
  }
}
