// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

interface Storage {
  readonly length: number;
  key(index: number): string | null;
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
  clear(): void;
}

class StorageImpl implements Storage {
  storageType: boolean;

  constructor(persistent: boolean) {
    storage_new(persistent);
    this.storageType = persistent;
  }

  get length(): number {
    return storage_length(this.storageType);
  }

  key(index: number): string | null {
    return storage_key(this.storageType, index);
  }

  getItem(key: string): string | null {
    if (typeof key !== "string") {
      key = String(key);
    }
    return storage_getItem(this.storageType, key);
  }
  setItem(key: string, value: string): void {
    if (typeof key !== "string") {
      key = String(key);
    }
    if (typeof value !== "string") {
      value = String(value);
    }
    storage_setItem(this.storageType, key, value);
  }

  removeItem(key: string): void {
    if (typeof key !== "string") {
      key = String(key);
    }
    storage_removeItem(this.storageType, key);
  }

  clear(): void {
    storage_clear(this.storageType);
  }
}

function createStorage(persistent: boolean): Storage {
  const storage = new StorageImpl(persistent);

  return new Proxy(storage as unknown as Storage, {
    deleteProperty(target, key) {
      if (typeof key === "symbol") {
        return Reflect.deleteProperty(target, key);
      }
      target.removeItem(key as string);
      return true;
    },

    defineProperty(target, key, descriptor) {
      if (typeof key === "symbol") {
        return Reflect.defineProperty(target, key, descriptor);
      }
      target.setItem(key as string, String(descriptor.value));
      return true;
    },
    get(target, key) {
      if (typeof key === "symbol") {
        return (target as unknown as Record<symbol, unknown>)[key];
      }
      if (Reflect.has(target, key)) {
        const value = (target as unknown as Record<string, unknown>)[key];
        if (typeof value === "function") {
          return value.bind(target);
        }
        return value;
      }
      return target.getItem(key as string) ?? undefined;
    },

    set(target, key, value) {
      if (typeof key === "symbol") {
        return Reflect.defineProperty(target, key, {
          value,
          configurable: true,
        });
      }
      target.setItem(key as string, String(value));
      return true;
    },

    has(target, key) {
      if (Reflect.has(target, key)) {
        return true;
      }
      return typeof key === "string" &&
        typeof target.getItem(key) === "string";
    },
    ownKeys(target) {
      const keys = storage_iterate_keys(
        (target as StorageImpl).storageType,
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

      const value = target.getItem(key as string);
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
