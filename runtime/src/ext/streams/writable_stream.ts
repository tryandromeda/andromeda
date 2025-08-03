// deno-lint-ignore-file no-unused-vars
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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

    error(_e?: unknown): void {
        // TODO: Implement proper error handling
        internal_writable_stream_abort(this.#streamId);
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
        // TODO: Implement desiredSize - check the stream state
        return 1;
    }

    get ready(): Promise<undefined> {
        // TODO: Implement ready - check backpressure
        return Promise.resolve(undefined);
    }

    abort(_reason?: unknown): Promise<void> {
        return new Promise((resolve) => {
            internal_writable_stream_abort(this.#streamId);
            resolve();
        });
    }

    close(): Promise<void> {
        return new Promise((resolve) => {
            internal_writable_stream_close(this.#streamId);
            resolve();
        });
    }

    releaseLock(): void {
        // TODO: Implement proper lock release
    }

    write(chunk: W): Promise<void> {
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

                const result = internal_writable_stream_write(
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

    constructor(
        underlyingSink?: WritableStreamUnderlyingSink<W>,
        _strategy?: QueuingStrategy<W>,
    ) {
        this.#streamId = internal_writable_stream_create();

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

    get locked(): boolean {
        return this.#writer !== null;
    }

    abort(_reason?: unknown): Promise<void> {
        return new Promise((resolve) => {
            internal_writable_stream_abort(this.#streamId);
            resolve();
        });
    }

    close(): Promise<void> {
        return new Promise((resolve) => {
            internal_writable_stream_close(this.#streamId);
            resolve();
        });
    }

    getWriter(): WritableStreamDefaultWriter<W> {
        if (this.locked) {
            throw new TypeError("WritableStream is already locked");
        }

        this.#writer = new WritableStreamDefaultWriter<W>(this.#streamId, this);
        return this.#writer;
    }

    get _streamId(): string {
        return this.#streamId;
    }
}
