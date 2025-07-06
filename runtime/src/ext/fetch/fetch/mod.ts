// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * Implementation of the fetch API for Andromeda
 * Based on: https://developer.mozilla.org/ja/docs/Web/API/Window/fetch
 * Spec: https://fetch.spec.whatwg.org/#fetch-method/
 */

type RequestInfo = Request | URL;

/** The fetch(input, init) method steps are: */
const fetch = (input: RequestInfo, init = undefined) => {
  // 1. Let p be a new promise.
  let p = createDeferredPromise();

  // 2. Let requestObject be the result of invoking the initial value
  // of Request as constructor with input and init as arguments.
  // If this throws an exception, reject p with it and return p.
  let request: any;

  try {
    // 3. Let request be requestObject’s request.
    // @ts-ignore deno lint stuff
    request = new Request(input, init);
  } catch (e) {
    p.reject(e);
    return p.promise;
  }

  // 4. If requestObject’s signal is aborted, then:
  // if (request.signal.aborted) {
  // 1. Abort the fetch() call with p, request, null, and
  // requestObject’s signal’s abort reason.
  //
  // TODO: abortFetch
  //
  // 2. Return p.
  // return p.promise;
  // }

  // 5. Let globalObject be request’s client’s global object.
  // const globalObject = request.client.globalObject;

  // 6. If globalObject is a ServiceWorkerGlobalScope object,
  // then set request’s service-workers mode to "none".
  // if (globalObject?.constructor?.name === "ServiceWorkerGlobalScope") {
  //   request.serviceWorkers = "none";
  // }

  // 7. Let responseObject be null.
  let responseObject = null;

  // 8. Let relevantRealm be this’s relevant realm.
  // 9. Let locallyAborted be false.
  // NOTE: This lets us reject promises with predictable timing,
  // when the request to abort comes from the same thread as
  // the call to fetch.
  let locallyAborted = false;

  // 10. Let controller be null.
  let controller = null;

  // TODO: abort controller
  // 11. Add the following abort steps to requestObject’s signal:
  //  1. Set locallyAborted to true.
  //  2. Assert: controller is non-null.
  //  3. Abort controller with requestObject’s signal’s abort reason.
  //  4. Abort the fetch() call with p, request, responseObject,
  //     and requestObject’s signal’s abort reason.

  // 12. Set controller to the result of calling fetch given request
  //     and processResponse given response being these steps:
  //  1. If locallyAborted is true, then abort these steps.
  //  2. If response’s aborted flag is set, then:
  //    1. Let deserializedError be the result of deserialize a serialized abort reason given controller’s serialized abort reason and relevantRealm.
  //    2. Abort the fetch() call with p, request, responseObject, and deserializedError.
  //    3. Abort these steps.
  //  3. If response is a network error, then reject p with a TypeError and abort these steps.
  //  4. Set responseObject to the result of creating a Response object, given response, "immutable", and relevantRealm.
  //  5. Resolve p with responseObject.

  // 13. Return p.
  return p;
};

(globalThis as unknown as { fetch: typeof fetch }).fetch = fetch;

function createDeferredPromise() {
  let res: any;
  let rej: any;
  const promise = new Promise((resolve, reject) => {
    res = resolve;
    rej = reject;
  });

  return { promise, resolve: res, reject: rej };
}
