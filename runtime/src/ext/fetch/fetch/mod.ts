// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * Implementation of the fetch API for Andromeda
 * Based on: https://developer.mozilla.org/ja/docs/Web/API/Window/fetch
 * Spec: https://fetch.spec.whatwg.org/#fetch-method/
 */

type RequestInfo = Request | URL;

class Fetch {
  // TODO: Event
  constructor() {
    (this as any).dispatcher = {};
    (this as any).connection = null;
    (this as any).dump = false;
    (this as any).state = "ongoing";
  }
}

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
  controller = fetching({
    request,
  });

  // 13. Return p.
  return p.promise;
};

(globalThis as unknown as { fetch: typeof fetch; }).fetch = fetch;

function createDeferredPromise() {
  let res: any;
  let rej: any;
  const promise = new Promise((resolve, reject) => {
    res = resolve;
    rej = reject;
  });

  return { promise, resolve: res, reject: rej };
}

/**
 * To fetch, given a request request, an optional algorithm processRequestBodyChunkLength, an optional algorithm processRequestEndOfBody,
 * an optional algorithm processEarlyHintsResponse, an optional algorithm processResponse, an optional algorithm processResponseEndOfBody,
 * an optional algorithm processResponseConsumeBody, and an optional boolean useParallelQueue (default false), run the steps below.
 * If given, processRequestBodyChunkLength must be an algorithm accepting an integer representing the number of bytes transmitted.
 * If given, processRequestEndOfBody must be an algorithm accepting no arguments. If given, processEarlyHintsResponse must be
 * an algorithm accepting a response. If given, processResponse must be an algorithm accepting a response. If given,
 * processResponseEndOfBody must be an algorithm accepting a response. If given,
 * processResponseConsumeBody must be an algorithm accepting a response and null, failure, or a byte sequence.
 *
 * The user agent may be asked to suspend the ongoing fetch. The user agent may either accept or ignore the suspension request.
 * The suspended fetch can be resumed. The user agent should ignore the suspension request if the ongoing fetch is updating
 * the response in the HTTP cache for the request.
 *
 * @see https://fetch.spec.whatwg.org/#fetching
 */
const fetching = (
  {
    request,
    processRequestBodyChunkLength,
    processRequestEndOfBody,
    processResponse,
    processResponseEndOfBody,
    processResponseConsumeBody,
    processEarlyHintsResponse,
  }: {
    request: any;
    processRequestBodyChunkLength?: any;
    processRequestEndOfBody?: any;
    processResponse?: any;
    processResponseEndOfBody?: any;
    processResponseConsumeBody?: any;
    processEarlyHintsResponse?: any;
  },
) => {
  // 1. Assert: request’s mode is "navigate" or processEarlyHintsResponse is null.
  // NOTE: Processing of early hints (responses whose status is 103) is only vetted for navigations.
  if (request.mode === "navigate") {
    throw new Error("error");
  }

  // 2. Let taskDestination be null.
  let taskDestination = null;

  // 3. Let crossOriginIsolatedCapability be false.
  let crossOriginIsolatedCapability = false;

  // 4. Populate request from client given request.
  // populateRequest();

  // 5. If request’s client is non-null, then:
  if (request.client != null) {
    //  1. Set taskDestination to request’s client’s global object.
    taskDestination = request.client.globalObject;
    //  2. Set crossOriginIsolatedCapability to request’s client’s cross-origin isolated capability.
    crossOriginIsolatedCapability =
      request.client.crossOriginIsolatedCapability;
  }

  // TODO
  // 6. If useParallelQueue is true, then set taskDestination to the result of starting a new parallel queue.

  // TODO
  // 7. Let timingInfo be a new fetch timing info whose start time and post-redirect start time are
  //    the coarsened shared current time given crossOriginIsolatedCapability,
  //    and render-blocking is set to request’s render-blocking.
  let timingInfo = 0;

  // 8. Let fetchParams be a new fetch params whose
  //    request is request, timing info is timingInfo,
  //    process request body chunk length is processRequestBodyChunkLength,
  //    process request end-of-body is processRequestEndOfBody,
  //    process early hints response is processEarlyHintsResponse,
  //    process response is processResponse,
  //    process response consume body is processResponseConsumeBody,
  //    process response end-of-body is processResponseEndOfBody,
  //    task destination is taskDestination,
  //    and cross-origin isolated capability is crossOriginIsolatedCapability.
  const fetchParams = {
    controller: new Fetch(),
    timingInfo, // TODO
    processRequestBodyChunkLength,
    processRequestEndOfBody,
    processResponse,
    processResponseConsumeBody,
    processResponseEndOfBody,
    taskDestination,
    crossOriginIsolatedCapability,
  };

  // TODO: step9, 10
  // 9. If request’s body is a byte sequence, then set request’s body to request’s body as a body.
  // 10. If all of the following conditions are true:
  //      - request’s URL’s scheme is an HTTP(S) scheme
  //      - request’s mode is "same-origin", "cors", or "no-cors"
  //      - request’s client is not null, and request’s client’s global object is a Window object
  //      - request’s method is `GET`
  //      - request’s unsafe-request flag is not set or request’s header list is empty
  //    then:
  //      1. Assert: request’s origin is same origin with request’s client’s origin.
  //      2. Let onPreloadedResponseAvailable be an algorithm that runs the following step given a response response:
  //         set fetchParams’s preloaded response candidate to response.
  //      3. Let foundPreloadedResource be the result of invoking consume a preloaded resource for request’s client,
  //         given request’s URL, request’s destination, request’s mode, request’s credentials mode, request’s integrity metadata,
  //         and onPreloadedResponseAvailable.
  //      4. If foundPreloadedResource is true and fetchParams’s preloaded response candidate is null, then set fetchParams’s preloaded response candidate to "pending".

  // 11. If request’s header list does not contain `Accept`, then:
  // if (!request.headersList.contains("accept", true)) {
  //  1. Let value be `*/*`.
  // const value = "*/*";

  // TODO
  //  2. If request’s initiator is "prefetch", then set value to the document `Accept` header value.
  //  3. Otherwise, the user agent should set value to the first matching statement, if any, switching on request’s destination:
  //    ↪︎ "document"
  //    ↪︎ "frame"
  //    ↪︎ "iframe"
  //        the document `Accept` header value
  //    ↪︎ "image"
  //        `image/png,image/svg+xml,image/*;q=0.8,*/*;q=0.5`
  //    ↪︎ "json"
  //        `application/json,*/*;q=0.5`
  //    ↪︎ "style"
  //        `text/css,*/*;q=0.1`

  //  4. Append (`Accept`, value) to request’s header list.
  // request.headersList.append("accept", value, true);
  // }

  // 12. If request’s header list does not contain `Accept-Language`,
  //     then user agents should append
  //     (`Accept-Language, an appropriate header value)
  //     to request’s header list.
  // if (!request.headersList.contains("accept-language", true)) {
  //   request.headersList.append("accept-language", "*", true);
  // }

  // TODO
  // 13. If request’s internal priority is null, then use request’s priority,
  //     initiator, destination, and render-blocking in an implementation-defined
  //     manner to set request’s internal priority to an implementation-defined object.

  // NOTE: The implementation-defined object could encompass stream weight and dependency for HTTP/2, priorities used
  //       in Extensible Prioritization Scheme for HTTP for transports where it applies (including HTTP/3),
  //       and equivalent information used to prioritize dispatch and processing of HTTP/1 fetches. [RFC9218]
  // 14. If request is a subresource request, then:
  //  1. Let record be a new fetch record whose request is request and controller is fetchParams’s controller.
  //  2. Append record to request’s client’s fetch group list of fetch records.

  // 15. Run main fetch given fetchParams.
  mainFetch(fetchParams);

  // 16. Return fetchParams’s controller.
  return fetchParams.controller;
};

/**
 * To populate request from client given a request request:
 * @see https://fetch.spec.whatwg.org/#populate-request-from-client
 */
const populateRequest = () => {
  // 1. If request’s traversable for user prompts is "client":
  //  1. Set request’s traversable for user prompts to "no-traversable".
  //  2. If request’s client is non-null:
  //    1. Let global be request’s client’s global object.
  //    2. If global is a Window object and global’s navigable is not null, then set request’s traversable for user prompts to global’s navigable’s traversable navigable.
  // 2. If request’s origin is "client":
  //  1. Assert: request’s client is non-null.
  //  2. Set request’s origin to request’s client’s origin.
  // 3. If request’s policy container is "client":
  //  1. If request’s client is non-null, then set request’s policy container to a clone of request’s client’s policy container. [HTML]
  //  2. Otherwise, set request’s policy container to a new policy container.
};

/**
 * To main fetch, given a fetch params fetchParams and an optional boolean recursive (default false)
 * @see https://fetch.spec.whatwg.org/#main-fetch
 */
const mainFetch = (fetchParams: any) => {
  // 1. Let request be fetchParams’s request.
  // 2. Let response be null.
  // 3. If request’s local-URLs-only flag is set and request’s current URL is not local, then set response to a network error.
  // 4. Run report Content Security Policy violations for request.
  // 5. Upgrade request to a potentially trustworthy URL, if appropriate.
  // 6. Upgrade a mixed content request to a potentially trustworthy URL, if appropriate.
  // 7. If should request be blocked due to a bad port, should fetching request be blocked as mixed content, should request be blocked by Content Security Policy, or should request be blocked by Integrity Policy Policy returns blocked, then set response to a network error.
  // 8. If request’s referrer policy is the empty string, then set request’s referrer policy to request’s policy container’s referrer policy.
  // 9. If request’s referrer is not "no-referrer", then set request’s referrer to the result of invoking determine request’s referrer. [REFERRER]
  // NOTE: As stated in Referrer Policy, user agents can provide the end user with options to override request’s referrer to "no-referrer" or have it expose less sensitive information.
  // 10. Set request’s current URL’s scheme to "https" if all of the following conditions are true:
  //  - request’s current URL’s scheme is "http"
  //  - request’s current URL’s host is a domain
  //  - request’s current URL’s host’s public suffix is not "localhost" or "localhost."
  //  - Matching request’s current URL’s host per Known HSTS Host Domain Name Matching results in either a superdomain match with an asserted includeSubDomains directive or a congruent match (with or without an asserted includeSubDomains directive) [HSTS]; or DNS resolution for the request finds a matching HTTPS RR per section 9.5 of [SVCB]. [HSTS] [SVCB]
  // NOTE: As all DNS operations are generally implementation-defined, how it is determined that DNS resolution contains an HTTPS RR is also implementation-defined. As DNS operations are not traditionally performed until attempting to obtain a connection, user agents might need to perform DNS operations earlier, consult local DNS caches, or wait until later in the fetch algorithm and potentially unwind logic on discovering the need to change request’s current URL’s scheme.
  // 11. If recursive is false, then run the remaining steps in parallel.
  // 12. If response is null, then set response to the result of running the steps corresponding to the first matching statement:
  // ↪︎ fetchParams’s preloaded response candidate is non-null
  //  1. Wait until fetchParams’s preloaded response candidate is not "pending".
  //  2. Assert: fetchParams’s preloaded response candidate is a response.
  //  3. Return fetchParams’s preloaded response candidate.
  // ↪︎ ︎︎︎request’s current URL’s origin is same origin with request’s origin, and request’s response tainting is "basic"
  // ↪︎ request’s current URL’s scheme is "data"
  // ↪︎ request’s mode is "navigate" or "websocket"
  //  1. Set request’s response tainting to "basic".
  //  2. Return the result of running scheme fetch given fetchParams.
  // NOTE: HTML assigns any documents and workers created from URLs whose scheme is "data" a unique opaque origin. Service workers can only be created from URLs whose scheme is an HTTP(S) scheme. [HTML] [SW]
  // ↪︎ request’s mode is "same-origin"
  //    Return a network error.
  // ↪︎ request’s mode is "no-cors"
  //  1. If request’s redirect mode is not "follow", then return a network error.
  //  2. Set request’s response tainting to "opaque".
  //  3. Return the result of running scheme fetch given fetchParams.
  // ↪︎ request’s current URL’s scheme is not an HTTP(S) scheme
  //    Return a network error.
  // ↪ request’s use-CORS-preflight flag is set
  // ↪ request’s unsafe-request flag is set and either request’s method is not a CORS-safelisted method or CORS-unsafe request-header names with request’s header list is not empty
  //  1. Set request’s response tainting to "cors".
  //  2. Let corsWithPreflightResponse be the result of running HTTP fetch given fetchParams and true.
  //  3. If corsWithPreflightResponse is a network error, then clear cache entries using request.
  //  4. Return corsWithPreflightResponse.
  // ↪ Otherwise
  //  1. Set request’s response tainting to "cors".
  //  2. Return the result of running HTTP fetch given fetchParams.
  
  // 13. If recursive is true, then return response.
  
  // 14. If response is not a network error and response is not a filtered response, then:
  //  1.If request’s response tainting is "cors", then:
  //    1. Let headerNames be the result of extracting header list values given `Access-Control-Expose-Headers` and response’s header list.
  //    2. If request’s credentials mode is not "include" and headerNames contains `*`, then set response’s CORS-exposed header-name list to all unique header names in response’s header list.
  //    3. Otherwise, if headerNames is non-null or failure, then set response’s CORS-exposed header-name list to headerNames.
  // NOTE: One of the headerNames can still be `*` at this point, but will only match a header whose name is `*`.
  //  2. Set response to the following filtered response with response as its internal response, depending on request’s response tainting:
  // ↪︎ "basic"
  //    basic filtered response
  // ↪︎ "cors"
  //    CORS filtered response
  // "opaque"
  //    opaque filtered response
  // 15. Let internalResponse be response, if response is a network error; otherwise response’s internal response.
  // 16. If internalResponse’s URL list is empty, then set it to a clone of request’s URL list.
  // NOTE: A response’s URL list can be empty, e.g., when fetching an about: URL.
  // 17. Set internalResponse’s redirect taint to request’s redirect-taint.
  // 18. If request’s timing allow failed flag is unset, then set internalResponse’s timing allow passed flag.
  // 19. If response is not a network error and any of the following returns blocked
  //  - should internalResponse to request be blocked as mixed content
  //  - should internalResponse to request be blocked by Content Security Policy
  //  - should internalResponse to request be blocked due to its MIME type
  //  - should internalResponse to request be blocked due to nosniff
  // then set response and internalResponse to a network error.
  
  // 20. If response’s type is "opaque", internalResponse’s status is 206, internalResponse’s range-requested flag is set, and request’s header list does not contain `Range`, then set response and internalResponse to a network error.
  // NOTE: Traditionally, APIs accept a ranged response even if a range was not requested. This prevents a partial response from an earlier ranged request being provided to an API that did not make a range request.

  // 21. If response is not a network error and either request’s method is `HEAD` or `CONNECT`, or internalResponse’s status is a null body status, set internalResponse’s body to null and disregard any enqueuing toward it (if any).
  // NOTE: This standardizes the error handling for servers that violate HTTP.
  
  // 22. If request’s integrity metadata is not the empty string, then:
  //  1. Let processBodyError be this step: run fetch response handover given fetchParams and a network error.
  //  2. If response’s body is null, then run processBodyError and abort these steps.
  //  3. Let processBody given bytes be these steps:
  //    1. If bytes do not match request’s integrity metadata, then run processBodyError and abort these steps. [SRI]
  //    2. Set response’s body to bytes as a body.
  //    3. Run fetch response handover given fetchParams and response.
  //  4. Fully read response’s body given processBody and processBodyError.
  // 23. Otherwise, run fetch response handover given fetchParams and response.
  };
