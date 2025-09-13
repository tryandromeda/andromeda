// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

interface LockOptions {
  /**
   * The mode of the lock. Default is "exclusive".
   * - "exclusive": Only one holder allowed at a time
   * - "shared": Multiple holders allowed simultaneously
   */
  mode?: "exclusive" | "shared";

  /**
   * If true, the request will fail if the lock cannot be granted immediately.
   * The callback will be invoked with null.
   */
  ifAvailable?: boolean;

  /**
   * If true, any held locks with the same name will be released,
   * and the request will be granted, preempting any queued requests.
   */
  steal?: boolean;

  /**
   * An AbortSignal that can be used to abort the lock request.
   */
  signal?: AbortSignal;
}

/**
 * Information about a lock for query results
 */
interface LockInfo {
  /** The name of the lock */
  name: string;
  /** The mode of the lock */
  mode: "exclusive" | "shared";
  /** An identifier for the client holding or requesting the lock */
  clientId?: string;
}

/**
 * Result of a query operation
 */
interface LockManagerSnapshot {
  /** Currently held locks */
  held: LockInfo[];
  /** Pending lock requests */
  pending: LockInfo[];
}

/**
 * Represents a granted lock
 */
class Lock {
  #name: string;
  #mode: "exclusive" | "shared";

  constructor(name: string, mode: "exclusive" | "shared") {
    this.#name = name;
    this.#mode = mode;
  }

  /**
   * The name of the lock
   */
  get name(): string {
    return this.#name;
  }

  /**
   * The mode of the lock
   */
  get mode(): "exclusive" | "shared" {
    return this.#mode;
  }
}

/**
 * The LockManager interface provides methods for requesting locks and querying lock state
 */
class LockManager {
  /**
   * Request a lock and execute a callback while holding it
   * @param name The name of the lock
   * @param callback The callback to execute while holding the lock
   * @param options Options for the lock request
   * @returns A promise that resolves with the return value of the callback
   */
  async request(
    name: string,
    callback: (lock: Lock | null) => unknown,
    options: LockOptions = {},
  ): Promise<unknown> {
    // Validate name (no leading '-')
    if (typeof name !== "string" || name.startsWith("-")) {
      throw new DOMException("Invalid lock name", "NotSupportedError");
    }

    // Validate options
    if (options.ifAvailable && options.steal) {
      throw new DOMException(
        "Cannot specify both 'ifAvailable' and 'steal' options",
        "NotSupportedError",
      );
    }

    if (typeof callback !== "function") {
      throw new DOMException("Callback must be a function", "TypeError");
    }

    // Check for AbortSignal
    if (options.signal && options.signal.aborted) {
      throw new DOMException("The operation was aborted", "AbortError");
    }

    const mode = options.mode || "exclusive";
    const ifAvailable = options.ifAvailable || false;
    const steal = options.steal || false;

    try {
      const lockIdResult = await __andromeda__.internal_locks_request(
        name,
        mode,
        ifAvailable,
        steal,
      );

      // Handle error responses
      if (lockIdResult.startsWith("error:")) {
        const errorMessage = lockIdResult.substring(6); // Remove 'error:' prefix

        if (errorMessage === "Invalid lock name") {
          throw new DOMException(
            "Invalid lock name",
            "NotSupportedError",
          );
        } else {
          throw new Error(errorMessage);
        }
      }

      if (lockIdResult === "not_available") {
        // Lock not available and ifAvailable was true
        return callback(null);
      }

      const lock = new Lock(name, mode);

      // Set up AbortSignal listener if provided
      let abortHandler: (() => void) | null = null;
      if (options.signal) {
        abortHandler = () => {
          __andromeda__.internal_locks_abort(name, lockIdResult);
          throw new DOMException(
            "The operation was aborted",
            "AbortError",
          );
        };
        options.signal.addEventListener("abort", abortHandler);
      }

      try {
        return await callback(lock);
      } finally {
        if (options.signal && abortHandler) {
          options.signal.removeEventListener("abort", abortHandler);
        }
        try {
          await __andromeda__.internal_locks_release(
            name,
            lockIdResult,
          );
        } catch (releaseError) {
          console.error(`Failed to release lock: ${releaseError}`);
        }
      }
    } catch (error) {
      throw error;
    }
  }

  /**
   * Query the current state of locks
   * @returns A promise that resolves with information about held and pending locks
   */
  async query(): Promise<LockManagerSnapshot> {
    try {
      const resultJson = await __andromeda__.internal_locks_query();
      const result = JSON.parse(resultJson);

      return {
        held: result.held || [],
        pending: result.pending || [],
      };
    } catch (error) {
      throw new Error(`Failed to query locks: ${error}`);
    }
  }
}

const lockManager = new LockManager();

// @ts-ignore - Adding locks property to navigator
navigator.locks = lockManager;

// @ts-ignore - Adding LockManager to global scope
globalThis.LockManager = LockManager;
// @ts-ignore - Adding Lock to global scope
globalThis.Lock = Lock;
