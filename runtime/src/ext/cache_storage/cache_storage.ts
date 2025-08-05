// deno-lint-ignore-file no-explicit-any
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * Implementation of the Cache and CacheStorage APIs for Andromeda
 * Based on: https://developer.mozilla.org/en-US/docs/Web/API/CacheStorage
 * Spec: https://w3c.github.io/ServiceWorker/#cache-interface
 *
 * Note: This is a simplified implementation that wraps the native sync functions
 * in promises to maintain compatibility with the Web API specification.
 */

type RequestInfo = Request | string | URL;

interface CacheQueryOptions {
  ignoreSearch?: boolean;
  ignoreMethod?: boolean;
  ignoreVary?: boolean;
}

class Cache {
  #cacheName: string;

  constructor(cacheName: string) {
    this.#cacheName = cacheName;
  }

  /**
   * Returns a Promise that resolves to the response associated with the first matching request in the Cache object.
   */
  match(
    request: RequestInfo,
    options?: CacheQueryOptions,
  ): Promise<Response | undefined> {
    return Promise.resolve(
      cache_match(this.#cacheName, request as any, options),
    );
  }

  /**
   * Returns a Promise that resolves to an array of all matching responses in the Cache object.
   */
  matchAll(
    request?: RequestInfo,
    options?: CacheQueryOptions,
  ): Promise<Response[]> {
    return Promise.resolve(
      cache_matchAll(this.#cacheName, request as any, options),
    );
  }

  /**
   * Takes a URL, retrieves it and adds the resulting response object to the given cache.
   */
  add(request: RequestInfo): Promise<void> {
    return Promise.resolve(cache_add(this.#cacheName, request as any));
  }

  /**
   * Takes an array of URLs, retrieves them, and adds the resulting response objects to the given cache.
   */
  addAll(requests: RequestInfo[]): Promise<void> {
    return Promise.resolve(cache_addAll(this.#cacheName, requests as any));
  }

  /**
   * Takes both a request and its response and adds it to the given cache.
   */
  put(request: RequestInfo, response: Response): Promise<void> {
    // Clone the response to ensure it can be consumed
    const responseClone = response.clone();
    return Promise.resolve(
      cache_put(this.#cacheName, request as any, responseClone),
    );
  }

  /**
   * Finds the Cache entry whose key is the request, and if found, deletes the Cache entry and returns a Promise that resolves to true.
   */
  delete(
    request: RequestInfo,
    options?: CacheQueryOptions,
  ): Promise<boolean> {
    return Promise.resolve(
      cache_delete(this.#cacheName, request as any, options),
    );
  }

  /**
   * Returns a Promise that resolves to an array of Cache keys.
   */
  keys(
    request?: RequestInfo,
    options?: CacheQueryOptions,
  ): Promise<Request[]> {
    return Promise.resolve(
      cache_keys(this.#cacheName, request as any, options),
    );
  }
}

class CacheStorage {
  /**
   * Returns a Promise that resolves to the Cache object matching the cacheName.
   */
  open(cacheName: string): Promise<Cache> {
    // Call the sync function
    cacheStorage_open(cacheName);
    return Promise.resolve(new Cache(cacheName));
  }

  /**
   * Returns a Promise that resolves to true if a Cache object matching the cacheName exists.
   */
  has(cacheName: string): Promise<boolean> {
    return Promise.resolve(cacheStorage_has(cacheName));
  }

  /**
   * Finds the Cache object matching the cacheName, and if found, deletes the Cache object and returns a Promise that resolves to true.
   */
  delete(cacheName: string): Promise<boolean> {
    return Promise.resolve(cacheStorage_delete(cacheName));
  }

  /**
   * Returns a Promise that will resolve with an array containing strings corresponding to all of the named Cache objects.
   */
  keys(): Promise<string[]> {
    return Promise.resolve(cacheStorage_keys());
  }

  /**
   * Checks if a given Request is a key in any of the Cache objects that the CacheStorage object tracks.
   */
  match(
    request: RequestInfo,
    options?: CacheQueryOptions,
  ): Promise<Response | undefined> {
    return Promise.resolve(cacheStorage_match(request as any, options));
  }
}

// Create global CacheStorage instance
let cacheStorageInstance: CacheStorage | undefined;

function getCacheStorage(): CacheStorage {
  if (!cacheStorageInstance) {
    cacheStorageInstance = new CacheStorage();
  }
  return cacheStorageInstance;
}

// Define the global 'caches' property
Object.defineProperty(globalThis, "caches", {
  get: () => getCacheStorage(),
  configurable: true,
  enumerable: true,
});
