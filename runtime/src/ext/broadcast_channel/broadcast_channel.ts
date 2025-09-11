// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// deno-lint-ignore-file no-explicit-any no-unused-vars

const channels: BroadcastChannel[] = [];
let rid: number | null = null;

/**
 * Create a MessageEvent-like object with all required properties per MDN spec
 */
function createMessageEvent<T>(type: string, data: T): Event {
  const event = new Event(type);

  Object.defineProperty(event, "data", {
    value: data,
    writable: false,
    enumerable: true,
    configurable: false,
  });

  Object.defineProperty(event, "origin", {
    value: "",
    writable: false,
    enumerable: true,
    configurable: false,
  });

  Object.defineProperty(event, "lastEventId", {
    value: "",
    writable: false,
    enumerable: true,
    configurable: false,
  });

  Object.defineProperty(event, "source", {
    value: null,
    writable: false,
    enumerable: true,
    configurable: false,
  });

  Object.defineProperty(event, "ports", {
    value: Object.freeze([]),
    writable: false,
    enumerable: true,
    configurable: false,
  });

  return event;
}

/**
 * Dispatch a message to all relevant channels
 */
function broadcastMessage(
  source: BroadcastChannel | null,
  name: string,
  data: unknown,
) {
  for (let i = 0; i < channels.length; ++i) {
    const channel = channels[i];

    if (channel === source) continue;
    if (channel.name !== name) continue;
    if (channel._closed) continue;

    const go = () => {
      if (channel._closed) return;

      const messageEvent = createMessageEvent("message", data);

      channel.dispatchEvent(messageEvent);
    };
    queueMicrotask(go);
  }
}

/**
 * Async function to receive messages from other processes/workers
 */
async function recv() {
  while (channels.length > 0) {
    const message = await new Promise((resolve) => {
      setTimeout(() => resolve(null), 100);
    });

    if (message === null) {
      break;
    }
  }

  if (rid !== null) {
    rid = null;
  }
}

/**
 * BroadcastChannel allows simple communication between browsing contexts
 */
class BroadcastChannel {
  #name: string;
  _closed: boolean;
  #eventTarget: EventTarget;
  #onmessage: ((event: Event) => void) | null = null;
  #onmessageerror: ((event: Event) => void) | null = null;

  /**
   * Returns the channel name (read-only per spec)
   */
  get name(): string {
    return this.#name;
  }

  /**
   * Gets the onmessage event handler
   */
  get onmessage(): ((event: Event) => void) | null {
    return this.#onmessage;
  }

  /**
   * Sets the onmessage event handler
   */
  set onmessage(value: ((event: Event) => void) | null) {
    if (this.#onmessage) {
      this.removeEventListener("message", this.#onmessage);
    }
    this.#onmessage = value;
    if (value) {
      this.addEventListener("message", value);
    }
  }

  /**
   * Gets the onmessageerror event handler
   */
  get onmessageerror(): ((event: Event) => void) | null {
    return this.#onmessageerror;
  }

  /**
   * Sets the onmessageerror event handler
   */
  set onmessageerror(value: ((event: Event) => void) | null) {
    if (this.#onmessageerror) {
      this.removeEventListener("messageerror", this.#onmessageerror);
    }
    this.#onmessageerror = value;
    if (value) {
      this.addEventListener("messageerror", value);
    }
  }

  /**
   * Creates a new BroadcastChannel with the given name
   */
  constructor(name: string) {
    if (arguments.length < 1) {
      throw new TypeError(
        "Failed to construct 'BroadcastChannel': 1 argument required, but only 0 present.",
      );
    }
    this.#eventTarget = new EventTarget();

    this.#name = String(name);
    this._closed = false;

    channels.push(this);
    if (rid === null) {
      rid = __andromeda__.op_broadcast_subscribe();
      recv();
    }
  }
  /**
   * EventTarget interface methods - delegate to internal EventTarget
   */
  addEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject | null,
    options?: boolean | AddEventListenerOptions,
  ): void {
    this.#eventTarget.addEventListener(type, listener, options);
  }

  removeEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject | null,
    options?: boolean | EventListenerOptions,
  ): void {
    this.#eventTarget.removeEventListener(type, listener, options);
  }
  dispatchEvent(event: Event): boolean {
    return this.#eventTarget.dispatchEvent(event);
  }

  /**
   * Posts a message to all other BroadcastChannel objects with the same name
   * Uses the global structuredClone function per HTML spec
   */
  postMessage(message: unknown): void {
    if (this._closed) {
      // @ts-ignore createDOMException is defined in dom_exception.ts
      throw createDOMException(
        "BroadcastChannel is closed",
        "InvalidStateError",
      );
    }

    let serializedData: unknown;
    try {
      serializedData = (globalThis as unknown as {
        structuredClone: (value: unknown) => unknown;
      })
        .structuredClone(message);
    } catch (error) {
      if (error instanceof Error && error.name === "DataCloneError") {
        throw error;
      }
      // @ts-ignore createDOMException is defined in dom_exception.ts
      throw createDOMException(
        "Failed to clone message",
        "DataCloneError",
      );
    }

    broadcastMessage(this, this.name, serializedData);

    queueMicrotask(() => {
      if (!this._closed && rid !== null) {
        try {
          __andromeda__.op_broadcast_send(
            rid,
            this.name,
            serializedData,
          );
        } catch (_e) {
          this.dispatchEvent(
            new ErrorEvent("messageerror", {
              // @ts-ignore createDOMException is defined in dom_exception.ts
              error: createDOMException(
                "Failed to send message",
                "DataCloneError",
              ),
            }),
          );
        }
      }
    });
  }

  /**
   * Closes the BroadcastChannel
   */
  close(): void {
    this._closed = true;

    const index = channels.indexOf(this);
    if (index === -1) return;

    channels.splice(index, 1);

    if (channels.length === 0 && rid !== null) {
      __andromeda__.op_broadcast_unsubscribe(rid);
      rid = null;
    }
  }

  /**
   * Custom inspect implementation for debugging
   */
  [Symbol.for("Deno.privateCustomInspect")](
    inspect: unknown,
    inspectOptions: unknown,
  ) {
    return (inspect as any)({
      name: this.name,
      onmessage: this.onmessage,
      onmessageerror: this.onmessageerror,
    }, inspectOptions);
  }
}

globalThis.BroadcastChannel = BroadcastChannel;
