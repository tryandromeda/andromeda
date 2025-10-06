// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Web Storage API implementation following:
// https://html.spec.whatwg.org/multipage/webstorage.html
// https://webidl.spec.whatwg.org/

// deno-lint-ignore-file no-explicit-any

const webidl = (globalThis as any).webidl;

interface Storage {
  readonly length: number;
  key(index: number): string | null;
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
  clear(): void;
  [name: string]: any;
}

class StorageImpl implements Storage {
  #persistent: boolean;

  constructor(persistent: boolean) {
    __andromeda__.storage_new(persistent);
    this.#persistent = persistent;
    // Set brand after construction
    (this as any)[webidl.brand] = webidl.brand;
  }

  get length(): number {
    // TODO: re-enable branding checks
    // webidl.assertBranded(this, StoragePrototype);
    return __andromeda__.storage_length(this.#persistent);
  }

  key(index: number): string | null {
    // TODO: re-enable branding checks
    // webidl.assertBranded(this, StoragePrototype);
    const prefix = "Failed to execute 'key' on 'Storage'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    index = webidl.converters["unsigned long"](index, prefix, "Argument 1");
    return __andromeda__.storage_key(this.#persistent, index);
  }

  getItem(key: string): string | null {
    // TODO: re-enable branding checks
    // webidl.assertBranded(this, StoragePrototype);
    const prefix = "Failed to execute 'getItem' on 'Storage'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    key = webidl.converters.DOMString(key, prefix, "Argument 1");
    return __andromeda__.storage_getItem(this.#persistent, key);
  }

  setItem(key: string, value: string): void {
    // TODO: re-enable branding checks
    // webidl.assertBranded(this, StoragePrototype);
    const prefix = "Failed to execute 'setItem' on 'Storage'";
    webidl.requiredArguments(arguments.length, 2, prefix);
    key = webidl.converters.DOMString(key, prefix, "Argument 1");
    value = webidl.converters.DOMString(value, prefix, "Argument 2");
    __andromeda__.storage_setItem(this.#persistent, key, value);
  }

  removeItem(key: string): void {
    // TODO: re-enable branding checks
    // webidl.assertBranded(this, StoragePrototype);
    const prefix = "Failed to execute 'removeItem' on 'Storage'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    key = webidl.converters.DOMString(key, prefix, "Argument 1");
    __andromeda__.storage_removeItem(this.#persistent, key);
  }

  clear(): void {
    // TODO: re-enable branding checks
    // webidl.assertBranded(this, StoragePrototype);
    __andromeda__.storage_clear(this.#persistent);
  }

  _getPersistent(): boolean {
    return this.#persistent;
  }
}

webidl.configureInterface(StorageImpl);
// deno-lint-ignore no-unused-vars
const StoragePrototype = StorageImpl.prototype;

function createStorage(persistent: boolean): Storage {
  const storage = new StorageImpl(persistent);

  const proxy = new Proxy(storage, {
    deleteProperty(target, key) {
      if (typeof key === "symbol") {
        return Reflect.deleteProperty(target, key);
      }
      target.removeItem(key);
      return true;
    },

    defineProperty(target, key, descriptor) {
      if (typeof key === "symbol") {
        return Reflect.defineProperty(target, key, descriptor);
      }
      target.setItem(key, String(descriptor.value));
      return true;
    },

    get(target, key) {
      if (typeof key === "symbol") {
        return (target as any)[key];
      }
      if (Reflect.has(target, key)) {
        const value = (target as any)[key];
        if (typeof value === "function") {
          return value.bind(target);
        }
        return value;
      }
      return target.getItem(key) ?? undefined;
    },

    set(target, key, value) {
      if (typeof key === "symbol") {
        return Reflect.defineProperty(target, key, {
          value,
          configurable: true,
        });
      }
      target.setItem(key, String(value));
      return true;
    },

    has(target, key) {
      if (Reflect.has(target, key)) {
        return true;
      }
      return typeof key === "string" && typeof target.getItem(key) === "string";
    },

    ownKeys(target) {
      const keys = __andromeda__.storage_iterate_keys(
        (target as StorageImpl)._getPersistent(),
      ) as string[];
      return keys;
    },

    getOwnPropertyDescriptor(target, key) {
      if (Reflect.has(target, key)) {
        return undefined;
      }
      if (typeof key === "symbol") {
        return undefined;
      }

      const value = target.getItem(key);
      if (value === null) {
        return undefined;
      }
      return {
        value,
        enumerable: true,
        configurable: true,
        writable: true,
      };
    },
  });

  return proxy;
}

let localStorageInstance: Storage | undefined;
let sessionStorageInstance: Storage | undefined;

function getLocalStorage(): Storage {
  if (!localStorageInstance) {
    localStorageInstance = createStorage(true);
  }
  return localStorageInstance;
}

function getSessionStorage(): Storage {
  if (!sessionStorageInstance) {
    sessionStorageInstance = createStorage(false);
  }
  return sessionStorageInstance;
}

Object.defineProperty(globalThis, "localStorage", {
  get: function() {
    return getLocalStorage();
  },
  configurable: true,
  enumerable: true,
});

Object.defineProperty(globalThis, "sessionStorage", {
  get: function() {
    return getSessionStorage();
  },
  configurable: true,
  enumerable: true,
});

Object.defineProperty(globalThis, "Storage", {
  value: StorageImpl,
  writable: true,
  enumerable: false,
  configurable: true,
});
