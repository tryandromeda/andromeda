// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// deno-lint-ignore-file no-explicit-any

const selfEventTarget = new EventTarget();

(globalThis as any).__andromeda_dispatch_self_event = function(
  kind: string,
  arg1?: string,
): void {
  if (kind === "message") {
    let data: unknown;
    try {
      data = (globalThis as any).__andromeda_structured_deserialize(arg1 ?? "");
    } catch (_e) {
      selfEventTarget.dispatchEvent((globalThis as any).__andromeda_make_message_event("messageerror", null));
      return;
    }
    selfEventTarget.dispatchEvent((globalThis as any).__andromeda_make_message_event("message", data));
  } else if (kind === "messageerror") {
    selfEventTarget.dispatchEvent(
      (globalThis as any).__andromeda_make_message_event("messageerror", arg1 ?? ""),
    );
  }
};

(globalThis as any).__andromeda_init_worker_globals = function(
  workerName: string,
): void {
  let onmessage_handler: ((event: Event) => void) | null = null;
  let onmessageerror_handler: ((event: Event) => void) | null = null;

  function workerPostMessage(message: unknown, transferOrOptions?: any): void {
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
    const serialized = (globalThis as any).__andromeda_structured_serialize(
      message,
      transfer,
    );
    __andromeda__.op_worker_post_to_parent(serialized);
  }

  function workerClose(): void {
    __andromeda__.op_worker_close_self();
  }

  // @ts-ignore add EventTarget surface to globalThis
  globalThis.addEventListener = (type: string, listener: any, options?: any) =>
    selfEventTarget.addEventListener(type, listener, options);
  // @ts-ignore add EventTarget surface to globalThis
  globalThis.removeEventListener =
    (type: string, listener: any, options?: any) =>
      selfEventTarget.removeEventListener(type, listener, options);
  // @ts-ignore add EventTarget surface to globalThis
  globalThis.dispatchEvent = (event: Event) =>
    selfEventTarget.dispatchEvent(event);

  // @ts-ignore add worker-specific globals
  globalThis.postMessage = workerPostMessage;
  // @ts-ignore add worker-specific globals
  globalThis.close = workerClose;
  // @ts-ignore add worker-specific globals
  globalThis.name = workerName;

  Object.defineProperty(globalThis, "onmessage", {
    get() {
      return onmessage_handler;
    },
    set(value: ((event: Event) => void) | null) {
      if (onmessage_handler) {
        selfEventTarget.removeEventListener("message", onmessage_handler);
      }
      onmessage_handler = value;
      if (value) selfEventTarget.addEventListener("message", value);
    },
    configurable: true,
    enumerable: true,
  });

  Object.defineProperty(globalThis, "onmessageerror", {
    get() {
      return onmessageerror_handler;
    },
    set(value: ((event: Event) => void) | null) {
      if (onmessageerror_handler) {
        selfEventTarget.removeEventListener(
          "messageerror",
          onmessageerror_handler,
        );
      }
      onmessageerror_handler = value;
      if (value) selfEventTarget.addEventListener("messageerror", value);
    },
    configurable: true,
    enumerable: true,
  });

  Object.defineProperty(globalThis, "self", {
    get() {
      return globalThis;
    },
    configurable: true,
    enumerable: true,
  });
};
