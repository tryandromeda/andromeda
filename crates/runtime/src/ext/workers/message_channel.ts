// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// deno-lint-ignore-file no-explicit-any

class MessagePort {
  _other: MessagePort | null = null;
  _started: boolean = false;
  _closed: boolean = false;
  _pending: unknown[] = [];
  #eventTarget: EventTarget;
  #onmessage: ((event: Event) => void) | null = null;
  #onmessageerror: ((event: Event) => void) | null = null;

  constructor() {
    this.#eventTarget = new EventTarget();
  }

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

  get onmessage(): ((event: Event) => void) | null {
    return this.#onmessage;
  }
  set onmessage(value: ((event: Event) => void) | null) {
    if (this.#onmessage) {
      this.#eventTarget.removeEventListener("message", this.#onmessage);
    }
    this.#onmessage = value;
    if (value) {
      this.#eventTarget.addEventListener("message", value);
      // Spec: assigning a value handler implicitly starts the port.
      this.start();
    }
  }

  get onmessageerror(): ((event: Event) => void) | null {
    return this.#onmessageerror;
  }
  set onmessageerror(value: ((event: Event) => void) | null) {
    if (this.#onmessageerror) {
      this.#eventTarget.removeEventListener(
        "messageerror",
        this.#onmessageerror,
      );
    }
    this.#onmessageerror = value;
    if (value) this.#eventTarget.addEventListener("messageerror", value);
  }

  postMessage(message: unknown, transferOrOptions?: any): void {
    if (this._closed) return;
    if (!this._other) return;

    let transfer: unknown[] = [];
    if (Array.isArray(transferOrOptions)) {
      transfer = transferOrOptions;
    } else if (
      transferOrOptions &&
      typeof transferOrOptions === "object" &&
      Array.isArray(transferOrOptions.transfer)
    ) {
      transfer = transferOrOptions.transfer;
    }

    const cloned = (globalThis as any).structuredClone(message, { transfer });
    const target = this._other;
    if (target._started) {
      queueMicrotask(() => {
        if (target._closed) return;
        target.dispatchEvent((globalThis as any).__andromeda_make_message_event("message", cloned));
      });
    } else {
      target._pending.push(cloned);
    }
  }

  start(): void {
    if (this._closed || this._started) return;
    this._started = true;
    const queued = this._pending;
    this._pending = [];
    // deno-lint-ignore no-this-alias
    const port = this;
    for (const data of queued) {
      queueMicrotask(() => {
        if (port._closed) return;
        port.dispatchEvent((globalThis as any).__andromeda_make_message_event("message", data));
      });
    }
  }

  close(): void {
    if (this._closed) return;
    this._closed = true;
    if (this._other) {
      this._other._other = null;
      this._other = null;
    }
  }
}

class MessageChannel {
  readonly port1: MessagePort;
  readonly port2: MessagePort;

  constructor() {
    const p1 = new MessagePort();
    const p2 = new MessagePort();
    p1._other = p2;
    p2._other = p1;
    this.port1 = p1;
    this.port2 = p2;
  }
}

// @ts-ignore globalThis is not readonly
globalThis.MessagePort = MessagePort;
// @ts-ignore globalThis is not readonly
globalThis.MessageChannel = MessageChannel;
