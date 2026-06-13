// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// deno-lint-ignore-file no-explicit-any

interface WorkerOptions {
  type?: "classic" | "module";
  name?: string;
  credentials?: "omit" | "same-origin" | "include";
}

// Map of worker_id -> Worker instance, used by the dispatcher.
const workerRegistry: Map<number, Worker> = new Map();

class Worker {
  #id: number;
  #name: string;
  #terminated: boolean;
  #eventTarget: EventTarget;
  #onmessage: ((event: Event) => void) | null = null;
  #onmessageerror: ((event: Event) => void) | null = null;
  #onerror: ((event: Event) => void) | null = null;

  constructor(scriptURL: string | URL, options: WorkerOptions = {}) {
    if (arguments.length < 1) {
      throw new TypeError(
        "Failed to construct 'Worker': 1 argument required, but only 0 present.",
      );
    }

    const type = options.type ?? "classic";
    if (type !== "module") {
      throw new TypeError(
        "Andromeda only supports module workers. Pass { type: \"module\" }.",
      );
    }

    const name = options.name ?? "";
    this.#name = name;
    this.#terminated = false;
    this.#eventTarget = new EventTarget();

    let url: string = scriptURL instanceof URL
      ? scriptURL.href
      : String(scriptURL);
    if (url.startsWith("file://")) {
      url = url.slice("file://".length);
    }

    this.#id = __andromeda__.op_worker_spawn(
      url,
      name,
      type,
    ) as unknown as number;
    workerRegistry.set(this.#id, this);
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
    if (value) this.#eventTarget.addEventListener("message", value);
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

  get onerror(): ((event: Event) => void) | null {
    return this.#onerror;
  }
  set onerror(value: ((event: Event) => void) | null) {
    if (this.#onerror) {
      this.#eventTarget.removeEventListener("error", this.#onerror);
    }
    this.#onerror = value;
    if (value) this.#eventTarget.addEventListener("error", value);
  }

  postMessage(message: unknown, transferOrOptions?: any): void {
    if (this.#terminated) return; // silent drop per spec

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

    const { json, sharedValues } = (globalThis as any)
      .__andromeda_structured_serialize(message, transfer);
    __andromeda__.op_worker_post_to_worker(this.#id, json, ...sharedValues);
  }

  terminate(): void {
    if (this.#terminated) return;
    this.#terminated = true;
    workerRegistry.delete(this.#id);
    __andromeda__.op_worker_terminate(this.#id);
  }
}

(globalThis as any).__andromeda_make_message_event = function(
  type: string,
  data: unknown,
): Event {
  const ev = new Event(type);
  Object.defineProperty(ev, "data", {
    value: data,
    writable: false,
    enumerable: true,
    configurable: false,
  });
  Object.defineProperty(ev, "origin", {
    value: "",
    writable: false,
    enumerable: true,
    configurable: false,
  });
  Object.defineProperty(ev, "lastEventId", {
    value: "",
    writable: false,
    enumerable: true,
    configurable: false,
  });
  Object.defineProperty(ev, "ports", {
    value: Object.freeze([]),
    writable: false,
    enumerable: true,
    configurable: false,
  });
  return ev;
};

(globalThis as any).__andromeda_make_error_event = function(
  message: string,
  filename: string,
  lineno: number,
  colno: number,
): Event {
  const ev = new Event("error");
  Object.defineProperty(ev, "message", {
    value: message,
    writable: false,
    enumerable: true,
    configurable: false,
  });
  Object.defineProperty(ev, "filename", {
    value: filename,
    writable: false,
    enumerable: true,
    configurable: false,
  });
  Object.defineProperty(ev, "lineno", {
    value: lineno,
    writable: false,
    enumerable: true,
    configurable: false,
  });
  Object.defineProperty(ev, "colno", {
    value: colno,
    writable: false,
    enumerable: true,
    configurable: false,
  });
  Object.defineProperty(ev, "error", {
    value: null,
    writable: false,
    enumerable: true,
    configurable: false,
  });
  return ev;
};


(globalThis as any).__andromeda_dispatch_worker_event = function(
  worker_id: number,
  kind: string,
  arg1?: string,
  arg2?: unknown,
  arg3?: string,
  arg4?: string,
): void {
  const worker = workerRegistry.get(Number(worker_id));
  if (kind === "__cleanup__") {
    workerRegistry.delete(Number(worker_id));
    return;
  }
  if (!worker) return;

  if (kind === "message") {
    let data: unknown;
    try {
      data = (globalThis as any).__andromeda_structured_deserialize(
        arg1 ?? "",
        [],
        (arg2 as unknown[]) ?? [],
      );
    } catch (_e) {
      worker.dispatchEvent((globalThis as any).__andromeda_make_message_event("messageerror", null));
      return;
    }
    worker.dispatchEvent((globalThis as any).__andromeda_make_message_event("message", data));
  } else if (kind === "messageerror") {
    worker.dispatchEvent((globalThis as any).__andromeda_make_message_event("messageerror", null));
  } else if (kind === "error") {
    worker.dispatchEvent(
      (globalThis as any).__andromeda_make_error_event(
        arg1 ?? "",
        (arg2 as string) ?? "",
        Number(arg3 ?? 0) | 0,
        Number(arg4 ?? 0) | 0,
      ),
    );
  }
};

// @ts-ignore globalThis is not readonly
globalThis.Worker = Worker;
