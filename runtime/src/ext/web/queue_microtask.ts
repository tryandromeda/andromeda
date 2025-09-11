// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// deno-lint-ignore-file

function queueMicrotask(callback: () => void): void {
  if (typeof callback !== "function") {
    throw new TypeError(
      "The callback provided as an argument to queueMicrotask must be a function.",
    );
  }

  Promise.resolve().then(() => {
    try {
      callback();
    } catch (error) {
      console.error("Uncaught error in microtask callback:", error);
      throw error;
    }
  });
}

// @ts-ignore globalThis is not readonly
globalThis.queueMicrotask = queueMicrotask;
