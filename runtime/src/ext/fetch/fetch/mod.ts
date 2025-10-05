// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// deno-lint-ignore-file no-explicit-any prefer-const no-unused-vars

// CORS and response filtering helpers (loaded from cors.ts and response_filter.ts)
const corsCheck = (globalThis as any).corsCheck;
const corsPreflightCheck = (globalThis as any).corsPreflightCheck;
const createCORSPreflightRequest = (globalThis as any).createCORSPreflightRequest;
const filterResponse = (globalThis as any).filterResponse;
const createOpaqueRedirectFilteredResponse = (globalThis as any).createOpaqueRedirectFilteredResponse;

function getHeadersAsList(headers: any): [string, string][] {
  const headersList: [string, string][] = [];

  if (headers instanceof Headers) {
    for (const [name, value] of headers.entries()) {
      headersList.push([name, value]);
    }
  } else if (Array.isArray(headers)) {
    headersList.push(...headers);
  } else if (headers && typeof headers === "object") {
    for (const [name, value] of Object.entries(headers)) {
      headersList.push([name, String(value)]);
    }
  }

  return headersList;
}

function setRequestHeader(request: any, name: string, value: string) {
  if (request.headers instanceof Headers) {
    request.headers.set(name, value);
  } else {
    if (!request.headers) {
      request.headers = {};
    }
    request.headers[name] = value;

    if (request.headersList && Array.isArray(request.headersList)) {
      const lowerName = name.toLowerCase();
      const existingIndex = request.headersList.findIndex(
        ([headerName]: [string, string]) =>
          headerName.toLowerCase() === lowerName,
      );
      if (existingIndex >= 0) {
        request.headersList[existingIndex] = [name, value];
      } else {
        request.headersList.push([name, value]);
      }
    }
  }
}

function hasRequestHeader(request: any, name: string): boolean {
  if (request.headers instanceof Headers) {
    return request.headers.has(name);
  } else if (request.headers && typeof request.headers === "object") {
    return name in request.headers || name.toLowerCase() in request.headers;
  }
  return false;
}

const networkError = () => ({
  type: "error",
  status: 0,
  statusText: "",
  headersList: [],
  body: null,
  urlList: [],
});

const createDeferredPromise = () => {
  let res: any;
  let rej: any;
  const promise = new Promise((resolve, reject) => {
    res = resolve;
    rej = reject;
  });

  return { promise, resolve: res, reject: rej };
};

class Fetch {
  // TODO: Event
  constructor() {
    (this as any).dispatcher = {};
    (this as any).connection = null;
    (this as any).dump = false;
    (this as any).state = "ongoing";
  }
}

/**
 * Implementation of the fetch API for Andromeda
 * Based on: https://developer.mozilla.org/ja/docs/Web/API/Window/fetch
 * @see Spec: https://fetch.spec.whatwg.org/#fetch-method/
 *
 * The fetch(input, init) method steps are:
 */
const andromedaFetch = (input: RequestInfo, init = undefined) => {
  // 1. Let p be a new promise.
  let p = createDeferredPromise();

  // 2. Let requestObject be the result of invoking the initial value
  // of Request as constructor with input and init as arguments.
  // If this throws an exception, reject p with it and return p.
  let requestObject: any;
  let request: any;

  try {
    // Create a Request object
    requestObject = new Request(input, init);

    // 3. Let request be requestObject's request.
    // Build internal request structure from Request object's public API
    // Handle the case where requestObject.url might be an object with serialized property
    let urlString: string;
    if (typeof requestObject.url === "string") {
      urlString = requestObject.url;
    } else if (requestObject.url && typeof requestObject.url === "object") {
      urlString = requestObject.url.serialized || requestObject.url.href ||
        String(requestObject.url);
    } else {
      throw new TypeError("Invalid URL");
    }

    const url = new URL(urlString);

    // Extract headers from the Headers object
    const headersList = getHeadersAsList(requestObject.headers);

    // Safely access properties
    let mode,
      credentials,
      cache,
      redirect,
      referrer,
      referrerPolicy,
      integrity,
      keepalive,
      body,
      signal,
      destination;
    mode = requestObject.mode || "cors";
    credentials = requestObject.credentials || "same-origin";
    cache = requestObject.cache || "default";
    redirect = requestObject.redirect || "follow";
    referrer = requestObject.referrer || "about:client";
    referrerPolicy = requestObject.referrerPolicy || "";
    integrity = requestObject.integrity || "";
    keepalive = requestObject.keepalive || false;
    body = requestObject.body || null;
    signal = requestObject.signal || null;
    destination = requestObject.destination || "";

    request = {
      url: urlString,
      method: requestObject.method || "GET",
      headersList: headersList,
      headers: requestObject.headers,
      mode: mode,
      credentials: credentials,
      cache: cache,
      redirect: redirect,
      referrer: referrer,
      referrerPolicy: referrerPolicy,
      integrity: integrity,
      keepalive: keepalive,
      currentURL: url,
      localURLsOnly: false,
      urlList: [url],
      responseTainting: "basic",
      redirectMode: redirect,
      redirectCount: 0,
      body: body,
      signal: signal,
      client: null,
      window: null,
      origin: "client",
      policyContainer: null,
      serviceWorkersMode: "all",
      destination: destination,
      priority: null,
      internalPriority: null,
      timingAllowFailedFlag: false,
      preventNoCacheCacheControlHeaderModificationFlag: false,
      done: false,
      reloadNavigation: false,
      historyNavigation: false,
      userActivation: false,
      renderBlocking: false,
      initiator: "",
      unsafeRequestFlag: false,
      useCORSPreflightFlag: false,
      credentialsMode: credentials,
      CORSExposedHeaderNameList: [],
    };
  } catch (e) {
    const errorToReject = e instanceof Error ?
      e :
      new Error("Unknown error creating request");
    p.reject(errorToReject);
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
  //  2. If response's aborted flag is set, then:
  //    1. Let deserializedError be the result of deserialize a serialized abort reason given controller's serialized abort reason and relevantRealm.
  //    2. Abort the fetch() call with p, request, responseObject, and deserializedError.
  //    3. Abort these steps.
  //  3. If response is a network error, then reject p with a TypeError and abort these steps.
  //  4. Set responseObject to the result of creating a Response object, given response, "immutable", and relevantRealm.
  //  5. Resolve p with responseObject.
  controller = fetching({
    request,
    processResponse: (response: any) => {
      if (locallyAborted) {
        return;
      }

      if (response?.type === "error") {
        p.reject(new TypeError("Network error"));
        return;
      }

      // Create a Response object using the Response class
      // Convert body object to Uint8Array if needed
      let bodyData = null;
      if (response.body) {
        if (response.body instanceof Uint8Array) {
          bodyData = response.body;
        } else if (
          typeof response.body === "object" &&
          "length" in response.body &&
          typeof response.body.length === "number" &&
          isFinite(response.body.length)
        ) {
          // Convert object with numeric keys to Uint8Array
          const length = response.body.length;
          bodyData = new Uint8Array(length);
          for (let i = 0; i < length; i++) {
            bodyData[i] = response.body[i] || 0;
          }
        } else if (typeof response.body === "object") {
          // Try to get keys if length is not available or invalid
          const keys = Object.keys(response.body).filter(k => !isNaN(Number(k)));
          if (keys.length > 0) {
            const length = keys.length;
            bodyData = new Uint8Array(length);
            for (let i = 0; i < length; i++) {
              bodyData[i] = response.body[i] || 0;
            }
          }
        }
      }

      responseObject = new Response(bodyData, {
        status: response.status,
        statusText: response.statusText,
      } as any);

      // Add additional properties that might be needed
      Object.defineProperty(responseObject, "url", {
        value: response.urlList?.[0]?.href || request.url,
        writable: false,
        enumerable: true,
        configurable: true,
      });

      Object.defineProperty(responseObject, "type", {
        value: response.type || "basic",
        writable: false,
        enumerable: true,
        configurable: true,
      });

      Object.defineProperty(responseObject, "redirected", {
        value: response.redirected || false,
        writable: false,
        enumerable: true,
        configurable: true,
      });

      p.resolve(responseObject);
    },
  });

  // 13. Return p.
  return p.promise;
};

globalThis.fetch = andromedaFetch;
/**
 * @see https://fetch.spec.whatwg.org/#fetch-response-handover
 */
const fetchResponseHandover = (fetchParams: any, response: any) => {
  // Run processResponse if available
  if (fetchParams.processResponse) {
    fetchParams.processResponse(response);
  }
};

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
const fetching = ({
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
}) => {
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
    request,
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
 * To main fetch, given a fetch params fetchParams and an optional boolean recursive (default false)
 * @see https://fetch.spec.whatwg.org/#main-fetch
 */
const mainFetch = async (fetchParams: any, recursive = false) => {
  // 1. Let request be fetchParams's request.
  const request = fetchParams.request;

  // 2. Let response be null.
  let response: any = null;

  // 3. If request's local-URLs-only flag is set and request's current URL is not local, then set response to a network error.
  if (request.localURLsOnly) {
    const urlForScheme = request.currentURL?.serialized ||
      request.currentURL?.url ||
      String(request.currentURL);
    const parsedURL = new URL(urlForScheme);
    const isLocal = ["about:", "blob:", "data:"].includes(parsedURL.protocol);
    if (!isLocal) {
      response = networkError();
    }
  }

  // 4. Run report Content Security Policy violations for request.
  // TODO: Implement CSP violation reporting

  // 5. Upgrade request to a potentially trustworthy URL, if appropriate.
  // 6. Upgrade a mixed content request to a potentially trustworthy URL, if appropriate.
  // TODO: Implement mixed content upgrade

  // 7. If should request be blocked due to a bad port, should fetching request be blocked as mixed content, should request be blocked by Content Security Policy, or should request be blocked by Integrity Policy Policy returns blocked, then set response to a network error.
  // TODO: Implement blocking checks

  // 8. If request's referrer policy is the empty string, then set request's referrer policy to request's policy container's referrer policy.
  if (request.referrerPolicy === "") {
    request.referrerPolicy = request.policyContainer?.referrerPolicy ||
      "strict-origin-when-cross-origin";
  }

  // 9. If request's referrer is not "no-referrer", then set request's referrer to the result of invoking determine request's referrer. [REFERRER]
  // NOTE: As stated in Referrer Policy, user agents can provide the end user with options to override request's referrer to "no-referrer" or have it expose less sensitive information.
  if (request.referrer !== "no-referrer") {
    request.referrer = request.referrer;
  }

  // 10. Set request's current URL's scheme to "https" if all of the following conditions are true:
  //  - request's current URL's scheme is "http"
  //  - request's current URL's host is a domain
  //  - request's current URL's host's public suffix is not "localhost" or "localhost."
  //  - Matching request's current URL's host per Known HSTS Host Domain Name Matching results in either a superdomain match with an asserted includeSubDomains directive or a congruent match (with or without an asserted includeSubDomains directive) [HSTS]; or DNS resolution for the request finds a matching HTTPS RR per section 9.5 of [SVCB]. [HSTS] [SVCB]
  // NOTE: As all DNS operations are generally implementation-defined, how it is determined that DNS resolution contains an HTTPS RR is also implementation-defined. As DNS operations are not traditionally performed until attempting to obtain a connection, user agents might need to perform DNS operations earlier, consult local DNS caches, or wait until later in the fetch algorithm and potentially unwind logic on discovering the need to change request's current URL's scheme.

  // Handle URL object structure for HTTPS upgrade check
  const currentUrlString = request.currentURL?.serialized ||
    request.currentURL?.url ||
    String(request.currentURL);
  const currentParsedURL = new URL(currentUrlString);

  if (
    currentParsedURL.protocol === "http:" &&
    currentParsedURL.hostname &&
    currentParsedURL.hostname !== "localhost" &&
    currentParsedURL.hostname !== "localhost."
  ) {
    // Update the URL object
    const httpsUrl = currentUrlString.replace(/^http:/, "https:");
    request.currentURL = new URL(httpsUrl);
  }

  // 11. If recursive is false, then run the remaining steps in parallel.
  const runRemainingSteps = async () => {
    // 12. If response is null, then set response to the result of running the steps corresponding to the first matching statement:
    if (response === null) {
      // ↪︎ fetchParams's preloaded response candidate is non-null
      if (
        fetchParams.preloadedResponseCandidate !== null &&
        fetchParams.preloadedResponseCandidate !== undefined
      ) {
        //  1. Wait until fetchParams's preloaded response candidate is not "pending".
        let loopCount = 0;
        while (fetchParams.preloadedResponseCandidate === "pending") {
          loopCount++;
          if (loopCount > 1000) {
            console.error("Infinite loop detected in preloaded response wait");
            response = networkError();
            break;
          }
        }
        //  2. Assert: fetchParams's preloaded response candidate is a response.
        //  3. Return fetchParams's preloaded response candidate.
        return fetchParams.preloadedResponseCandidate;
      }

      // Parse URL properly to access protocol and origin
      const currentUrlString = request.currentURL?.serialized ||
        request.currentURL?.url ||
        String(request.currentURL);

      let currentParsedURL;
      try {
        currentParsedURL = new URL(currentUrlString);
      } catch (e) {
        console.error("URL parsing failed:", e);
        response = networkError();
        return;
      }

      const isSameOrigin = currentParsedURL.origin === request.origin;
      const isDataScheme = currentParsedURL.protocol === "data:";
      const isNavigateOrWebsocket = request.mode === "navigate" ||
        request.mode === "websocket";

      // ↪︎ ︎︎︎request's current URL's origin is same origin with request's origin, and request's response tainting is "basic"
      // ↪︎ request's current URL's scheme is "data"
      // ↪︎ request's mode is "navigate" or "websocket"
      if (
        (isSameOrigin && request.responseTainting === "basic") ||
        isDataScheme ||
        isNavigateOrWebsocket
      ) {
        //  1. Set request's response tainting to "basic".
        request.responseTainting = "basic";
        //  2. Return the result of running scheme fetch given fetchParams.
        // NOTE: HTML assigns any documents and workers created from URLs whose scheme is "data" a unique opaque origin. Service workers can only be created from URLs whose scheme is an HTTP(S) scheme. [HTML] [SW]
        response = await schemeFetch(fetchParams);
      } // ↪︎ request's mode is "same-origin"
      else if (request.mode === "same-origin") {
        //    Return a network error.
        response = networkError();
      } // ↪︎ request's mode is "no-cors"
      else if (request.mode === "no-cors") {
        //  1. If request's redirect mode is not "follow", then return a network error.
        if (request.redirectMode !== "follow") {
          response = networkError();
        } else {
          //  2. Set request's response tainting to "opaque".
          request.responseTainting = "opaque";
          //  3. Return the result of running scheme fetch given fetchParams.
          response = await schemeFetch(fetchParams);
        }
      } // ↪︎ request's current URL's scheme is not an HTTP(S) scheme
      else if (
        currentParsedURL.protocol !== "http:" &&
        currentParsedURL.protocol !== "https:"
      ) {
        //    Return a network error.
        response = networkError();
      } // ↪ request's use-CORS-preflight flag is set
      // ↪ request's unsafe-request flag is set and either request's method is not a CORS-safelisted method or CORS-unsafe request-header names with request's header list is not empty
      else if (
        request.useCORSPreflightFlag ||
        (request.unsafeRequestFlag &&
          (!["GET", "HEAD", "POST"].includes(request.method) || false))
      ) {
        //  1. Set request's response tainting to "cors".
        request.responseTainting = "cors";
        //  2. Let corsWithPreflightResponse be the result of running HTTP fetch given fetchParams and true.
        const corsWithPreflightResponse = await httpFetch(fetchParams, true);
        //  3. If corsWithPreflightResponse is a network error, then clear cache entries using request.
        if (corsWithPreflightResponse?.type === "error") {
          // Clear cache entries (no-op for now)
        }
        //  4. Return corsWithPreflightResponse.
        response = corsWithPreflightResponse;
      } // ↪ Otherwise
      else {
        //  1. Set request's response tainting to "cors".
        request.responseTainting = "cors";
        //  2. Return the result of running HTTP fetch given fetchParams.
        response = await httpFetch(fetchParams);
      }
    }

    // 13. If recursive is true, then return response.
    if (recursive) {
      return response;
    }

    // 14. If response is not a network error and response is not a filtered response, then:
    const responseIsValid = response && response.type !== "error";
    const responseIsFiltered = response &&
      (response.type === "basic" ||
        response.type === "cors" ||
        response.type === "opaque");
    const shouldProcessComplexResponse = responseIsValid && !responseIsFiltered;

    if (shouldProcessComplexResponse) {
      //  1.If request's response tainting is "cors", then:
      if (request.responseTainting === "cors") {
        //    1. Let headerNames be the result of extracting header list values given `Access-Control-Expose-Headers` and response's header list.
        const headerNames = null;
        //    2. If request's credentials mode is not "include" and headerNames contains `*`, then set response's CORS-exposed header-name list to all unique header names in response's header list.
        if (
          request.credentialsMode !== "include" &&
          headerNames?.includes("*")
        ) {
          response.CORSExposedHeaderNameList = [];
        } //    3. Otherwise, if headerNames is non-null or failure, then set response's CORS-exposed header-name list to headerNames.
        else if (headerNames !== null && headerNames !== "failure") {
          response.CORSExposedHeaderNameList = headerNames;
        }
        // NOTE: One of the headerNames can still be `*` at this point, but will only match a header whose name is `*`.
      }

      //  2. Set response to the following filtered response with response as its internal response, depending on request's response tainting:
      // ↪︎ "basic"
      //    basic filtered response
      // ↪︎ "cors"
      //    CORS filtered response
      // "opaque"
      //    opaque filtered response
      const filtered = {
        ...response,
        internalResponse: response,
      };

      switch (request.responseTainting) {
        case "basic":
          filtered.type = "basic";
          break;
        case "cors":
          filtered.type = "cors";
          break;
        case "opaque":
          filtered.type = "opaque";
          break;
      }

      response = filtered;
    }

    // 15. Let internalResponse be response, if response is a network error; otherwise response's internal response.
    const internalResponse = response?.type === "error" ?
      response :
      response?.internalResponse || response;

    // 16. If internalResponse's URL list is empty, then set it to a clone of request's URL list.
    // NOTE: A response's URL list can be empty, e.g., when fetching an about: URL.
    if (
      internalResponse &&
      (!internalResponse.urlList || internalResponse.urlList.length === 0)
    ) {
      internalResponse.urlList = [...request.urlList];
    }

    // 17. Set internalResponse's redirect taint to request's redirect-taint.
    if (internalResponse) {
      internalResponse.redirectTaint = request.redirectTaint;
    }

    // 18. If request's timing allow failed flag is unset, then set internalResponse's timing allow passed flag.
    if (!request.timingAllowFailedFlag && internalResponse) {
      internalResponse.timingAllowPassedFlag = true;
    }

    // 19. If response is not a network error and any of the following returns blocked
    //  - should internalResponse to request be blocked as mixed content
    //  - should internalResponse to request be blocked by Content Security Policy
    //  - should internalResponse to request be blocked due to its MIME type
    //  - should internalResponse to request be blocked due to nosniff
    // then set response and internalResponse to a network error.
    if (response?.type !== "error") {
      if (false || false || false || false) {
        response = internalResponse = networkError();
      }
    }

    // 20. If response's type is "opaque", internalResponse's status is 206, internalResponse's range-requested flag is set, and request's header list does not contain `Range`, then set response and internalResponse to a network error.
    // NOTE: Traditionally, APIs accept a ranged response even if a range was not requested. This prevents a partial response from an earlier ranged request being provided to an API that did not make a range request.
    if (
      response?.type === "opaque" &&
      internalResponse?.status === 206 &&
      internalResponse?.rangeRequestedFlag &&
      !request.headersList?.contains?.("Range")
    ) {
      response = internalResponse = networkError();
    }

    // 21. If response is not a network error and either request's method is `HEAD` or `CONNECT`, or internalResponse's status is a null body status, set internalResponse's body to null and disregard any enqueuing toward it (if any).
    // NOTE: This standardizes the error handling for servers that violate HTTP.
    if (response?.type !== "error" && internalResponse) {
      if (
        request.method === "HEAD" ||
        request.method === "CONNECT" ||
        [101, 103, 204, 205, 304].includes(internalResponse.status)
      ) {
        internalResponse.body = null;
      }
    }

    // 22. If request's integrity metadata is not the empty string, then:
    if (request.integrityMetadata && request.integrityMetadata !== "") {
      //  1. Let processBodyError be this step: run fetch response handover given fetchParams and a network error.
      const processBodyError = () => {
        if (fetchParams.processResponse) {
          fetchParams.processResponse(networkError());
        }
      };

      //  2. If response's body is null, then run processBodyError and abort these steps.
      if (response.body === null) {
        processBodyError();
        return;
      }

      //  3. Let processBody given bytes be these steps:
      const processBody = (bytes: Uint8Array) => {
        //    1. If bytes do not match request's integrity metadata, then run processBodyError and abort these steps. [SRI]
        if (!true) {
          processBodyError();
          return;
        }
        //    2. Set response's body to bytes as a body.
        response.body = bytes;
        //    3. Run fetch response handover given fetchParams and response.
        if (fetchParams.processResponse) {
          fetchParams.processResponse(response);
        }
      };

      //  4. Fully read response's body given processBody and processBodyError.
      processBody(new Uint8Array());
    } else {
      // 23. Otherwise, run fetch response handover given fetchParams and response.
      fetchResponseHandover(fetchParams, response);
    }
  };

  if (!recursive) {
    runRemainingSteps();
    // Don't return anything - the response will be handled via processResponse callback
  } else {
    return await runRemainingSteps();
  }
};

/**
 * @see https://fetch.spec.whatwg.org/#scheme-fetch
 * @description To scheme fetch, given a fetch params fetchParams:
 */
const schemeFetch = async (fetchParams: any) => {
  // 1. If fetchParams is canceled, then return the appropriate network error for fetchParams.
  if (fetchParams.controller?.state === "aborted") {
    return networkError();
  }

  // 2. Let request be fetchParams's request.
  const request = fetchParams.request;

  // 3. Switch on request's current URL's scheme and run the associated steps:

  // Handle URL object that might have different structure
  let urlForScheme;
  if (typeof request.currentURL === "string") {
    urlForScheme = request.currentURL;
  } else if (request.currentURL?.serialized) {
    urlForScheme = request.currentURL.serialized;
  } else if (request.currentURL?.url) {
    urlForScheme = request.currentURL.url;
  } else {
    urlForScheme = String(request.currentURL);
  }

  const parsedURL = new URL(urlForScheme);
  const scheme = parsedURL.protocol.slice(0, -1); // Remove the trailing colon

  switch (scheme) {
    // ↪︎ "about"
    case "about": {
      //    If request's current URL's path is the string "blank", then return a new response whose status message is `OK`,
      //    header list is « (`Content-Type`, `text/html;charset=utf-8`) », and body is the empty byte sequence as a body.
      if (request.currentURL.pathname === "blank") {
        return {
          type: "basic",
          status: 200,
          statusText: "OK",
          headersList: [["Content-Type", "text/html;charset=utf-8"]],
          body: new Uint8Array(),
          urlList: [request.currentURL],
        };
      }
      //    NOTE: URLs such as "about:config" are handled during navigation and result in a network error in the context of fetching.
      return networkError();
    }

    // ↪︎ "blob"
    case "blob": {
      //   1. Let blobURLEntry be request's current URL's blob URL entry.
      const blobURLEntry = null;

      //   2. If request's method is not `GET` or blobURLEntry is null, then return a network error. [FILEAPI]
      //   NOTE: The `GET` method restriction serves no useful purpose other than being interoperable.
      if (request.method !== "GET" || blobURLEntry === null) {
        return networkError();
      }

      //   3. Let requestEnvironment be the result of determining the environment given request.
      // TODO: Implement determining the environment
      const requestEnvironment = null;

      //   4. Let isTopLevelNavigation be true if request's destination is "document"; otherwise, false.
      const isTopLevelNavigation = request.destination === "document";

      //   5. If isTopLevelNavigation is false and requestEnvironment is null, then return a network error.
      if (!isTopLevelNavigation && requestEnvironment === null) {
        return networkError();
      }

      //   6. Let navigationOrEnvironment be the string "navigation" if isTopLevelNavigation is true; otherwise, requestEnvironment.
      const navigationOrEnvironment = isTopLevelNavigation ?
        "navigation" :
        requestEnvironment;

      //   7. Let blob be the result of obtaining a blob object given blobURLEntry and navigationOrEnvironment.
      // TODO: Implement obtaining blob object
      const blob = null;

      //   8. If blob is not a Blob object, then return a network error.
      if (
        !(blob && typeof blob === "object" && "size" in blob && "type" in blob)
      ) {
        return networkError();
      }

      //   9. Let response be a new response.
      const response: any = {
        type: "basic",
        status: 200,
        statusText: "OK",
        headersList: [],
        body: null,
        urlList: [request.currentURL],
      };

      //   10. Let fullLength be blob's size.
      const fullLength = blob.size;

      //   11. Let serializedFullLength be fullLength, serialized and isomorphic encoded.
      const serializedFullLength = String(fullLength);

      //   12. Let type be blob's type.
      const type = blob.type || "";

      //   13. If request's header list does not contain `Range`:
      if (
        !(
          request.headersList &&
          typeof request.headersList.contains === "function" &&
          request.headersList.contains("Range")
        )
      ) {
        //      1. Let bodyWithType be the result of safely extracting blob.
        const bodyWithType = { body: new Uint8Array(), type: blob.type };
        //      2. Set response's status message to `OK`.
        response.statusText = "OK";
        //      3. Set response's body to bodyWithType's body.
        response.body = bodyWithType.body;
        //      4. Set response's header list to « (`Content-Length`, serializedFullLength), (`Content-Type`, type) ».
        response.headersList = [
          ["Content-Length", serializedFullLength],
          ["Content-Type", type],
        ];
      } //   14. Otherwise:
      else {
        //      1. Set response's range-requested flag.
        response.rangeRequestedFlag = true;
        //      2. Let rangeHeader be the result of getting `Range` from request's header list.
        const rangeHeader =
          request.headersList && typeof request.headersList.get === "function" ?
            request.headersList.get("Range") :
            null;
        //      3. Let rangeValue be the result of parsing a single range header value given rangeHeader and true.
        const rangeValue = rangeHeader ? [0, 100] : null;
        //      4. If rangeValue is failure, then return a network error.
        if (rangeValue === null) {
          return networkError();
        }
        //      5. Let (rangeStart, rangeEnd) be rangeValue.
        let [rangeStart, rangeEnd] = rangeValue;
        //      6. If rangeStart is null:
        if (rangeStart === null) {
          //        1. Set rangeStart to fullLength − rangeEnd.
          rangeStart = fullLength - rangeEnd!;
          //        2. Set rangeEnd to rangeStart + rangeEnd − 1.
          rangeEnd = rangeStart + rangeEnd! - 1;
        } //      7. Otherwise:
        else {
          //        1. If rangeStart is greater than or equal to fullLength, then return a network error.
          if (rangeStart >= fullLength) {
            return networkError();
          }
          //        2. If rangeEnd is null or rangeEnd is greater than or equal to fullLength, then set rangeEnd to fullLength − 1.
          if (rangeEnd === null || rangeEnd >= fullLength) {
            rangeEnd = fullLength - 1;
          }
        }
        //      8. Let slicedBlob be the result of invoking slice blob given blob, rangeStart, rangeEnd + 1, and type.
        //         NOTE: A range header denotes an inclusive byte range, while the slice blob algorithm input range does not.
        //         To use the slice blob algorithm, we have to increment rangeEnd.
        const slicedBlob = { size: rangeEnd + 1 - rangeStart, type };
        //      9. Let slicedBodyWithType be the result of safely extracting slicedBlob.
        const slicedBodyWithType = {
          body: new Uint8Array(),
          type: slicedBlob.type,
        };
        //     10. Set response's body to slicedBodyWithType's body.
        response.body = slicedBodyWithType.body;
        //     11. Let serializedSlicedLength be slicedBlob's size, serialized and isomorphic encoded.
        const serializedSlicedLength = String(slicedBlob.size);
        //     12. Let contentRange be the result of invoking build a content range given rangeStart, rangeEnd, and fullLength.
        const contentRange = `bytes ${rangeStart}-${rangeEnd}/${fullLength}`;
        //     13. Set response's status to 206.
        response.status = 206;
        //     14. Set response's header list to « (`Content-Length`, serializedSlicedLength), (`Content-Type`, type), (`Content-Range`, contentRange) ».
        response.headersList = [
          ["Content-Length", serializedSlicedLength],
          ["Content-Type", type],
          ["Content-Range", contentRange],
        ];
      }
      //   15. Return response.
      return response;
    }

    // ↪︎ "data"
    case "data": {
      //   1. Let dataURLStruct be the result of running the data: URL processor on request's current URL.
      const urlString = request.currentURL.href;
      let dataURLStruct = null;
      if (urlString.startsWith("data:")) {
        const commaIndex = urlString.indexOf(",");
        if (commaIndex !== -1) {
          const mimeType = urlString.substring(5, commaIndex) ||
            "text/plain;charset=US-ASCII";
          const data = urlString.substring(commaIndex + 1);
          const body = new TextEncoder().encode(data);
          dataURLStruct = { mimeType, body };
        }
      }
      //   2. If dataURLStruct is failure, then return a network error.
      if (dataURLStruct === null) {
        return networkError();
      }
      //   3. Let mimeType be dataURLStruct's MIME type, serialized.
      const mimeType = dataURLStruct.mimeType;
      //   4. Return a new response whose status message is `OK`, header list is « (`Content-Type`, mimeType) »,
      //      and body is dataURLStruct's body as a body.
      return {
        type: "basic",
        status: 200,
        statusText: "OK",
        headersList: [["Content-Type", mimeType]],
        body: dataURLStruct.body,
        urlList: [request.currentURL],
      };
    }

    // ↪︎ "file"
    case "file": {
      //    For now, unfortunate as it is, file: URLs are left as an exercise for the reader.
      //    When in doubt, return a network error.
      // TODO: Implement file: URL handling
      return networkError();
    }

    // ↪︎ HTTP(S) scheme
    case "http":
    case "https": {
      //    Return the result of running HTTP fetch given fetchParams.
      return await httpFetch(fetchParams);
    }

    default:
      break;
  }

  //  4. Return a network error.
  return networkError();
};

/**
 * @see https://fetch.spec.whatwg.org/#http-fetch
 * @description To HTTP fetch, given a fetch params fetchParams and an optional boolean makeCORSPreflight (default false), run these steps:
 */

const httpFetch = async (fetchParams: any, makeCORSPreflight = false) => {
  // 1. Let request be fetchParams's request.
  const request = fetchParams.request;

  // 2. Let response and internalResponse be null.
  let response: any = null;
  let internalResponse: any = null;

  // 3. If request's service-workers mode is "all", then:
  if (request.serviceWorkersMode === "all") {
    // TODO: Service Workerl
    // 1. Let requestForServiceWorker be a clone of request.
    // 2. If requestForServiceWorker's body is non-null, then:
    //    1. Let transformStream be a new TransformStream.
    //    2. Let transformAlgorithm given chunk be these steps:
    //      1. If fetchParams is canceled, then abort these steps.
    //      2. If chunk is not a Uint8Array object, then terminate fetchParams's controller.
    //      3. Otherwise, enqueue chunk in transformStream.
    //    3. Set up transformStream with transformAlgorithm set to transformAlgorithm.
    //    4. Set requestForServiceWorker's body's stream to the result of requestForServiceWorker's body's stream piped through transformStream.
    // 3. Let serviceWorkerStartTime be the coarsened shared current time given fetchParams's cross-origin isolated capability.
    // 4. Set response to the result of invoking handle fetch for requestForServiceWorker, with fetchParams's controller and fetchParams's cross-origin isolated capability. [HTML] [SW]
    // 5. If response is non-null, then:
    //    1. Set fetchParams's timing info's final service worker start time to serviceWorkerStartTime.
    //    2. If request's body is non-null, then cancel request's body with undefined.
    //    3. Set internalResponse to response, if response is not a filtered response; otherwise to response's internal response.
    //    4. If one of the following is true
    //      - response's type is "error"
    //      - request's mode is "same-origin" and response's type is "cors"
    //      - request's mode is not "no-cors" and response's type is "opaque"
    //      - request's redirect mode is not "manual" and response's type is "opaqueredirect"
    //      - request's redirect mode is not "follow" and response's URL list has more than one item.
    //      then return a network error.
    // service-workers mode is all
  }

  // 4. If response is null, then:
  if (response === null) {
    // 4.1. If makeCORSPreflight is true and one of these conditions is true:
    //    - There is no method cache entry match for request's method using request, and either request's method is not a CORS-safelisted method or request's use-CORS-preflight flag is set.
    //    - There is at least one item in the CORS-unsafe request-header names with request's header list for which there is no header-name cache entry match using request.
    if (
      makeCORSPreflight &&
      (!["GET", "HEAD", "POST"].includes(request.method) ||
        request.useCORSPreflightFlag ||
        false)
    ) {
      // 1. Let preflightResponse be the result of running CORS-preflight fetch given request.
      const preflightResponse = { type: "basic", status: 200 };
      // 2. If preflightResponse is a network error, then return preflightResponse.
      if (preflightResponse?.type === "error") {
        return preflightResponse;
      }
    }

    // 4.2. If request's redirect mode is "follow", then set request's service-workers mode to "none".
    // NOTE: Redirects coming from the network (as opposed to from a service worker) are not to be exposed to a service worker.
    if (request.redirectMode === "follow") {
      request.serviceWorkersMode = "none";
    }

    // 4.3. Set response and internalResponse to the result of running HTTP-network-or-cache fetch given fetchParams.
    const fetchResult = await httpNetworkOrCacheFetch(fetchParams);
    response = internalResponse = fetchResult;

    // 4.4. If request's response tainting is "cors" and a CORS check for request and response returns failure, then return a network error.
    // NOTE: As the CORS check is not to be applied to responses whose status is 304 or 407, or responses from a service worker for that matter, it is applied here.
    if (
      request.responseTainting === "cors" &&
      !(response?.status === 304 || response?.status === 407 || true)
    ) {
      return networkError();
    }

    // 4.5. If the TAO check for request and response returns failure, then set request's timing allow failed flag.
    if (!true) {
      request.timingAllowFailedFlag = true;
    }
  }

  // 5. If either request's response tainting or response's type is "opaque", and the cross-origin resource policy check with request's origin, request's client, request's destination, and internalResponse returns blocked, then return a network error.
  // NOTE: The cross-origin resource policy check runs for responses coming from the network and responses coming from the service worker. This is different from the CORS check, as request's client and the service worker can have different embedder policies.
  if (
    (request.responseTainting === "opaque" || response?.type === "opaque") &&
    "allowed" === "blocked"
  ) {
    return networkError();
  }

  // 6. If internalResponse's status is a redirect status:
  if (
    internalResponse?.status &&
    [301, 302, 303, 307, 308].includes(internalResponse.status)
  ) {
    // 6.1. If internalResponse's status is not 303, request's body is non-null, and the connection uses HTTP/2, then user agents may, and are even encouraged to, transmit an RST_STREAM frame.
    // NOTE: 303 is excluded as certain communities ascribe special status to it.
    // TODO: HTTP/2 RST_STREAM処理

    // 6.2. Switch on request's redirect mode:
    switch (request.redirectMode) {
      // ↪︎ "error"
      case "error":
        // 1. Set response to a network error.
        response = networkError();
        break;

      // ↪︎ "manual"
      case "manual":
        // 1. If request's mode is "navigate", then set fetchParams's controller's next manual redirect steps to run HTTP-redirect fetch given fetchParams and response.
        if (request.mode === "navigate") {
          // TODO: manual redirect処理
          fetchParams.controller.nextManualRedirectSteps = () => response;
        } // 2. Otherwise, set response to an opaque-redirect filtered response whose internal response is internalResponse.
        else {
          response = {
            type: "opaqueredirect",
            status: 0,
            statusText: "",
            headersList: [],
            body: null,
            internalResponse,
          };
        }
        break;

      // ↪︎ "follow"
      case "follow":
        // 1. Set response to the result of running HTTP-redirect fetch given fetchParams and response.
        response = response;
        break;
    }
  }

  // 7. Return response. Typically internalResponse's body's stream is still being enqueued to after returning.
  return response;
};

/**
 * 4.6. HTTP-network fetch
 * @see SPEC https://fetch.spec.whatwg.org/#http-redirect-fetch
 * @description To HTTP-network fetch, given a fetch params fetchParams, an optional boolean includeCredentials (default false), and an optional boolean forceNewConnection (default false), run these steps:
 */
const httpNetworkFetch = async (
  fetchParams: any,
  includeCredentials = false,
  forceNewConnection = false,
) => {
  // 1. Let request be fetchParams's request.
  const request = fetchParams.request;

  // 2. Let response be null.
  let response: any = null;

  // 3. Let timingInfo be fetchParams's timing info.
  const timingInfo = fetchParams.timingInfo;

  // 4. Let networkPartitionKey be the result of determining the network partition key given request.
  const networkPartitionKey = request.currentURL.origin; // Simplified implementation

  // 5. Let newConnection be "yes" if forceNewConnection is true; otherwise "no".
  const newConnection = forceNewConnection ? "yes" : "no";

  // 6. Switch on request's mode:
  let connection: any = null;
  switch (request.mode) {
    case "websocket":
      // Let connection be the result of obtaining a WebSocket connection, given request's current URL.
      connection = { type: "websocket", url: request.currentURL };
      break;
    default:
      // Let connection be the result of obtaining a connection, given networkPartitionKey, request's current URL, includeCredentials, and newConnection.
      connection = {
        type: "http",
        url: request.currentURL,
        networkPartitionKey,
        includeCredentials,
        newConnection,
      };
      break;
  }

  // 7. Run these steps, but abort when fetchParams is canceled:

  // Check if fetchParams is canceled
  if (fetchParams.controller?.state === "aborted") {
    return networkError();
  }

  //  1. If connection is failure, then return a network error.
  if (!connection) {
    return networkError();
  }

  //  2. Set timingInfo's final connection timing info to the result of calling clamp and coarsen connection timing info with connection's timing info, timingInfo's post-redirect start time, and fetchParams's cross-origin isolated capability.
  // NOTE: Simplified implementation
  if (timingInfo) {
    timingInfo.finalConnectionTimingInfo = Date.now();
  }

  //  3. If connection is an HTTP/1.x connection, request's body is non-null, and request's body's source is null, then return a network error.
  if (
    connection.type === "http" &&
    request.body !== null &&
    request.body?.source === null
  ) {
    return networkError();
  }

  //  4. Set timingInfo's final network-request start time to the coarsened shared current time given fetchParams's cross-origin isolated capability.
  if (timingInfo) {
    timingInfo.finalNetworkRequestStartTime = Date.now();
  }

  //  5. Set response to the result of making an HTTP request over connection using request with the following caveats:
  try {
    // Make HTTP request
    const requestStartTime = Date.now();

    // Set timingInfo's final network-response start time
    if (timingInfo) {
      timingInfo.finalNetworkResponseStartTime = requestStartTime;
    }

    // Prepare headers for request
    const headers = request.headersList || getHeadersAsList(request.headers);

    // Check protocol
    if (
      request.currentURL.protocol !== "http:" &&
      request.currentURL.protocol !== "https:"
    ) {
      return networkError();
    }

    let status = 200;
    let statusText = "OK";
    let responseBody: Uint8Array | null = null;
    let responseHeaders: [string, string][] = [];

    // Use TLS for HTTPS connections
    if (request.currentURL.protocol === "https:") {
      try {
        const host = request.currentURL.hostname;
        const port = request.currentURL.port || 443;

        // Connect with TLS
        const rid = await __andromeda__.internal_tls_connect(host, port);

        // Format HTTP request
        const method = request.method || "GET";
        const path = request.currentURL.pathname + request.currentURL.search;

        // Build HTTP request string
        let httpRequest = `${method} ${path} HTTP/1.1\r\n`;
        httpRequest += `Host: ${host}\r\n`;

        // Add headers
        if (headers && Array.isArray(headers)) {
          for (const [name, value] of headers) {
            httpRequest += `${name}: ${value}\r\n`;
          }
        }

        // Add body if present
        let bodyData = "";
        if (request.body) {
          if (request.body instanceof Uint8Array) {
            bodyData = new TextDecoder().decode(request.body);
          } else if (typeof request.body === "string") {
            bodyData = request.body;
          }

          if (!hasRequestHeader(request, "Content-Length")) {
            httpRequest += `Content-Length: ${bodyData.length}\r\n`;
          }
        }

        httpRequest += `Connection: close\r\n`;
        httpRequest += `\r\n`;
        httpRequest += bodyData;

        // Send request
        await __andromeda__.internal_tls_write(rid, httpRequest);

        // Read response
        let responseHex = "";
        const maxRetries = 100;
        let retries = 0;

        while (retries < maxRetries) {
          try {
            const chunk = await __andromeda__.internal_tls_read(rid, 4096);
            if (!chunk || chunk.length === 0) break;
            responseHex += chunk;
          } catch (e) {
            console.error("TLS read error:", e);
            break;
          }
          retries++;
        }

        // Close connection
        await __andromeda__.internal_tls_close(rid);

        // Convert hex to bytes
        function hexToUtf8(hex: string) {
          if (!hex) return "";
          const bytes = new Uint8Array(
            hex.match(/.{1,2}/g)!.map((b) => parseInt(b, 16)),
          );
          return new TextDecoder().decode(bytes);
        }

        // Parse response
        const responseText = hexToUtf8(responseHex);
        const [headerPart, ...bodyParts] = responseText.split("\r\n\r\n");
        const bodyText = bodyParts.join("\r\n\r\n");

        // Parse status line
        const lines = headerPart.split("\r\n");
        const statusLine = lines[0];
        const statusMatch = statusLine.match(/HTTP\/\d\.\d (\d+) (.+)/);

        if (statusMatch) {
          status = parseInt(statusMatch[1]);
          statusText = statusMatch[2];
        }

        // Parse headers
        for (let i = 1; i < lines.length; i++) {
          const line = lines[i];
          const colonIndex = line.indexOf(":");
          if (colonIndex > 0) {
            const name = line.substring(0, colonIndex).trim();
            const value = line.substring(colonIndex + 1).trim();
            responseHeaders.push([name, value]);
          }
        }

        // Set response body
        if (bodyText) {
          responseBody = new TextEncoder().encode(bodyText);
        }
      } catch (error) {
        return networkError();
      }
    } else {
      // For HTTP (non-TLS), return a simple mock response for now
      responseBody = new TextEncoder().encode(
        '{"message": "HTTP not implemented yet"}',
      );
      responseHeaders = [
        ["Content-Type", "application/json"],
        ["Content-Length", String(responseBody.length)],
        ["Date", new Date().toUTCString()],
      ];
    }

    // Handle interim responses (100-199 range)
    if (status >= 100 && status <= 199) {
      if (timingInfo && timingInfo.firstInterimNetworkResponseStartTime === 0) {
        timingInfo.firstInterimNetworkResponseStartTime =
          timingInfo.finalNetworkResponseStartTime;
      }

      if (request.mode === "websocket" && status === 101) {
        // WebSocket upgrade complete
      }

      if (status === 103 && fetchParams.processEarlyHintsResponse) {
        // Process early hints response
        fetchParams.processEarlyHintsResponse(response);
      }
    }

    // Create the response object
    response = {
      type: "basic",
      status: status,
      statusText: statusText,
      headersList: responseHeaders,
      body: responseBody,
      urlList: [request.currentURL],
      ok: status >= 200 && status < 300,
      redirected: false,
      rangeRequestedFlag: false,
      aborted: false,
      timingAllowPassedFlag: true,
    };
  } catch (error) {
    return networkError();
  }

  // Handle TLS client certificate dialog
  // (Simplified - in real implementation this would handle certificate dialogs)

  // To transmit request's body, run these steps:
  try {
    //  1. If body is null and fetchParams's process request end-of-body is non-null, then queue a fetch task
    if (request.body === null && fetchParams.processRequestEndOfBody) {
      // Queue task to run processRequestEndOfBody
      setTimeout(() => {
        if (
          fetchParams.processRequestEndOfBody &&
          !fetchParams.controller?.aborted
        ) {
          fetchParams.processRequestEndOfBody();
        }
      }, 0);
    } //  2. Otherwise, if body is non-null:
    else if (request.body !== null) {
      //    1. Let processBodyChunk given bytes be these steps:
      const processBodyChunk = (bytes: Uint8Array) => {
        //      1. If fetchParams is canceled, then abort these steps.
        if (fetchParams.controller?.state === "aborted") {
          return;
        }
        //      2. Run this step in parallel: transmit bytes.
        // (Simplified - would actually transmit bytes over network)
        //      3. If fetchParams's process request body chunk length is non-null, then run it
        if (fetchParams.processRequestBodyChunkLength) {
          fetchParams.processRequestBodyChunkLength(bytes.length);
        }
      };

      //    2. Let processEndOfBody be these steps:
      const processEndOfBody = () => {
        //      1. If fetchParams is canceled, then abort these steps.
        if (fetchParams.controller?.state === "aborted") {
          return;
        }
        //      2. If fetchParams's process request end-of-body is non-null, then run it
        if (fetchParams.processRequestEndOfBody) {
          fetchParams.processRequestEndOfBody();
        }
      };

      //    3. Let processBodyError given e be these steps:
      const processBodyError = (e: any) => {
        //      1. If fetchParams is canceled, then abort these steps.
        if (fetchParams.controller?.state === "aborted") {
          return;
        }
        //      2. If e is an "AbortError" DOMException, then abort fetchParams's controller.
        if (e?.name === "AbortError") {
          if (fetchParams.controller?.abort) {
            fetchParams.controller.abort();
          }
        } else {
          //      3. Otherwise, terminate fetchParams's controller.
          if (fetchParams.controller?.terminate) {
            fetchParams.controller.terminate();
          }
        }
      };

      //    4. Incrementally read request's body (simplified implementation)
      try {
        if (request.body instanceof Uint8Array) {
          processBodyChunk(request.body);
        } else if (typeof request.body === "string") {
          processBodyChunk(new TextEncoder().encode(request.body));
        }
        processEndOfBody();
      } catch (error) {
        processBodyError(error);
      }
    }
  } catch (error) {
    return networkError();
  }

  // 8. If aborted, then:
  if (fetchParams.controller?.state === "aborted") {
    //  1. If connection uses HTTP/2, then transmit an RST_STREAM frame.
    // (Simplified - in real implementation would send RST_STREAM for HTTP/2)
    //  2. Return the appropriate network error for fetchParams.
    return networkError();
  }

  // 9. Let buffer be an empty byte sequence.
  let buffer = new Uint8Array(0);

  // 10. Let stream be a new ReadableStream.
  // 11. Let pullAlgorithm be the following steps:
  const pullAlgorithm = () => {
    return new Promise<void>((resolve) => {
      // Simplified implementation
      setTimeout(() => {
        if (fetchParams.controller?.state !== "aborted") {
          resolve();
        }
      }, 0);
    });
  };

  // 12. Let cancelAlgorithm be an algorithm that aborts fetchParams's controller
  const cancelAlgorithm = (reason: any) => {
    if (fetchParams.controller?.abort) {
      fetchParams.controller.abort(reason);
    }
  };

  // 13. Set up stream with byte reading support (simplified)
  // 14. Set response's body to a new body whose stream is stream.
  if (response && response.body) {
    // Response body is already set from earlier processing
  }

  // 15. If includeCredentials is true, parse and store Set-Cookie headers
  if (includeCredentials && response) {
    // Simplified - in real implementation would parse Set-Cookie headers
    const setCookieHeaders = response.headersList.filter(
      ([name]: [string, string]) => name.toLowerCase() === "set-cookie",
    );
    // Store cookies (simplified)
  }

  // 16. Run these steps in parallel:
  // (Simplified streaming implementation)
  if (response) {
    try {
      // Handle response body streaming
      if (response.body instanceof Uint8Array) {
        const bytes = response.body;

        // Extract Content-Encoding header
        const contentEncodingHeader = response.headersList.find(
          ([name]: [string, string]) =>
            name.toLowerCase() === "content-encoding",
        );
        const codings = contentEncodingHeader ? [contentEncodingHeader[1]] : [];

        let filteredCoding = "";
        if (codings.length === 0) {
          filteredCoding = "";
        } else if (codings.length > 1) {
          filteredCoding = "multiple";
        } else {
          filteredCoding = codings[0].toLowerCase();
        }

        // Set response body info (simplified)
        if (!response.bodyInfo) {
          response.bodyInfo = {
            contentEncoding: filteredCoding,
            encodedSize: bytes.length,
            decodedSize: bytes.length,
          };
        }

        // Append bytes to buffer
        const newBuffer = new Uint8Array(buffer.length + bytes.length);
        newBuffer.set(buffer);
        newBuffer.set(bytes, buffer.length);
        buffer = newBuffer;
      }
    } catch (error) {
      // If error occurs during streaming, return network error
      return networkError();
    }
  }

  // Handle abort cases
  if (fetchParams.controller?.state === "aborted") {
    if (response) {
      response.aborted = true;
    }
    return networkError();
  }

  // 17. Return response.
  return response || networkError();
};
/**
 * @see https://fetch.spec.whatwg.org/#http-redirect-fetch
 * @description To HTTP-redirect fetch, given a fetch params fetchParams and a response response, run these steps:
 */
const httpRedirectFetch = async (fetchParams: any, response: any) => {
  // 1. Let request be fetchParams's request.
  const request = fetchParams.request;

  // 2. Let internalResponse be response, if response is not a filtered response; otherwise response's internal response.
  const internalResponse = response?.internalResponse || response;

  // 3. Let locationURL be internalResponse's location URL given request's current URL's fragment.
  let locationURL: URL | null = null;
  const locationHeader = internalResponse.headersList?.find(
    ([name]: [string, string]) => name.toLowerCase() === "location",
  );
  if (locationHeader && locationHeader[1]) {
    try {
      locationURL = new URL(locationHeader[1], request.currentURL);
      // Preserve fragment from request's current URL if exists
      if (request.currentURL.hash) {
        locationURL.hash = request.currentURL.hash;
      }
    } catch (e) {
      // Invalid URL, return network error
      return networkError();
    }
  }

  // 4. If locationURL is null, then return response.
  if (locationURL === null) {
    return response;
  }

  // 6. If locationURL's scheme is not an HTTP(S) scheme, then return a network error.
  if (locationURL.protocol !== "http:" && locationURL.protocol !== "https:") {
    return networkError();
  }

  // 7. If request's redirect count is 20, then return a network error.
  if (!request.redirectCount) {
    request.redirectCount = 0;
  }
  if (request.redirectCount >= 20) {
    return networkError();
  }

  // 8. Increase request's redirect count by 1.
  request.redirectCount++;

  // 9. If request's mode is "cors", locationURL includes credentials, and request's origin is not same origin with locationURL's origin, then return a network error.
  if (
    request.mode === "cors" &&
    (locationURL.username || locationURL.password) &&
    request.origin !== locationURL.origin
  ) {
    return networkError();
  }

  // 10. If request's response tainting is "cors" and locationURL includes credentials, then return a network error.
  //     Note: This catches a cross-origin resource redirecting to a same-origin URL.
  if (
    request.responseTainting === "cors" &&
    (locationURL.username || locationURL.password)
  ) {
    return networkError();
  }

  // 11. If internalResponse's status is not 303, request's body is non-null, and request's body's source is null, then return a network error.
  if (
    internalResponse.status !== 303 &&
    request.body !== null &&
    request.body?.source === null
  ) {
    return networkError();
  }

  // 12. If one of the following is true
  //     - internalResponse's status is 301 or 302 and request's method is `POST`
  //     - internalResponse's status is 303 and request's method is not `GET` or `HEAD`
  //     then:
  if (
    ((internalResponse.status === 301 || internalResponse.status === 302) &&
      request.method === "POST") ||
    (internalResponse.status === 303 &&
      request.method !== "GET" &&
      request.method !== "HEAD")
  ) {
    // 12.1. Set request's method to `GET` and request's body to null.
    request.method = "GET";
    request.body = null;

    // 12.2. For each headerName of request-body-header name, delete headerName from request's header list.
    const requestBodyHeaders = [
      "content-encoding",
      "content-language",
      "content-location",
      "content-type",
      "content-length",
    ];
    if (request.headers instanceof Headers) {
      for (const header of requestBodyHeaders) {
        request.headers.delete(header);
      }
    } else if (request.headersList && Array.isArray(request.headersList)) {
      request.headersList = request.headersList.filter(
        ([name]: [string, string]) => !requestBodyHeaders.includes(name.toLowerCase()),
      );
    }
  }

  // 13. If request's current URL's origin is not same origin with locationURL's origin, then for each headerName of CORS non-wildcard request-header name, delete headerName from request's header list.
  //     Note: I.e., the moment another origin is seen after the initial request, the `Authorization` header is removed.
  if (request.currentURL.origin !== locationURL.origin) {
    // CORS non-wildcard request-header names include at minimum "authorization"
    const corsNonWildcardHeaders = ["authorization"];
    if (request.headers instanceof Headers) {
      for (const header of corsNonWildcardHeaders) {
        request.headers.delete(header);
      }
    } else if (request.headersList && Array.isArray(request.headersList)) {
      request.headersList = request.headersList.filter(
        ([name]: [string, string]) => !corsNonWildcardHeaders.includes(name.toLowerCase()),
      );
    }
  }

  // 14. If request's body is non-null, then set request's body to the body of the result of safely extracting request's body's source.
  //     Note: request's body's source's nullity has already been checked.
  if (request.body !== null && request.body.source) {
    // Simplified - keeping existing body since safely extracting is complex
    // In a full implementation, this would properly extract and recreate the body
  }

  // 15. Let timingInfo be fetchParams's timing info.
  const timingInfo = fetchParams.timingInfo;

  // 16. Set timingInfo's redirect end time and post-redirect start time to the coarsened shared current time given fetchParams's cross-origin isolated capability.
  if (timingInfo) {
    timingInfo.redirectEndTime = Date.now();
    timingInfo.postRedirectStartTime = Date.now();
  }

  // 17. If timingInfo's redirect start time is 0, then set timingInfo's redirect start time to timingInfo's start time.
  if (timingInfo && timingInfo.redirectStartTime === 0) {
    timingInfo.redirectStartTime = timingInfo.startTime;
  }

  // 18. Append locationURL to request's URL list.
  if (!request.urlList) {
    request.urlList = [];
  }
  request.urlList.push(locationURL);

  // 19. Invoke set request's referrer policy on redirect on request and internalResponse. [REFERRER]
  // TODO: Implement proper referrer policy handling on redirect
  // This would check for Referrer-Policy header in the response and update request's referrer policy

  // 20. Let recursive be true.
  let recursive = true;

  // 21. If request's redirect mode is "manual", then:
  if (request.redirectMode === "manual") {
    // 21.1. Assert: request's mode is "navigate".
    if (request.mode !== "navigate") {
      throw new Error(
        "Assertion failed: manual redirect mode requires navigate mode",
      );
    }
    // 21.2. Set recursive to false.
    recursive = false;
  }

  // 22. Return the result of running main fetch given fetchParams and recursive.
  //     Note: This has to invoke main fetch to get request's response tainting correct.
  request.currentURL = locationURL;
  return await mainFetch(fetchParams, recursive);
};

/**
 * @see https://fetch.spec.whatwg.org/#http-network-or-cache-fetch
 * @description To HTTP-network-or-cache fetch, given a fetch params fetchParams, an optional boolean isAuthenticationFetch (default false), and an optional boolean isNewConnectionFetch (default false), run these steps:
 */
const httpNetworkOrCacheFetch = async (
  fetchParams: any,
  isAuthenticationFetch = false,
  isNewConnectionFetch = false,
) => {
  // 1. Let request be fetchParams's request.
  const request = fetchParams.request;

  // 2. Let httpFetchParams be null.
  let httpFetchParams = null;

  // 3. Let httpRequest be null.
  let httpRequest = null;

  // 4. Let response be null.
  let response = null;

  // 5. Let storedResponse be null.
  let storedResponse = null;

  // 6. Let httpCache be null.
  let httpCache = null;

  // 7. Let the revalidatingFlag be unset.
  let revalidatingFlag = false;

  // 8. Run these steps, but abort when fetchParams is canceled:
  //  1. If request's traversable for user prompts is "no-traversable" and request's redirect mode is "error", then set httpFetchParams to fetchParams and httpRequest to request.
  if (
    request.traversable === "no-traversable" &&
    request.redirectMode === "error"
  ) {
    httpFetchParams = fetchParams;
    httpRequest = request;
  } else {
    //  2. Otherwise:
    //     1. Set httpRequest to a clone of request.
    httpRequest = { ...request };
    // Properly clone headers if they're a Headers object
    if (request.headers instanceof Headers) {
      httpRequest.headers = new Headers(request.headers);
    }
    //     2. Set httpFetchParams to a copy of fetchParams.
    httpFetchParams = { ...fetchParams };
    //     3. Set httpFetchParams's request to httpRequest.
    httpFetchParams.request = httpRequest;
  }

  //  3. Let includeCredentials be true if one of
  //     - request's credentials mode is "include"
  //     - request's credentials mode is "same-origin" and request's response tainting is "basic"
  //     is true; otherwise false.
  let includeCredentials = httpRequest.credentialsMode === "include" ||
    (httpRequest.credentialsMode === "same-origin" &&
      httpRequest.responseTainting === "basic");

  //  4. If Cross-Origin-Embedder-Policy allows credentials with request returns false, then set includeCredentials to false.
  // TODO: Implement Cross-Origin-Embedder-Policy check

  //  5. Let contentLength be httpRequest's body's length, if httpRequest's body is non-null; otherwise null.
  const contentLength = httpRequest.body?.length || null;

  //  6. Let contentLengthHeaderValue be null.
  let contentLengthHeaderValue = null;

  //  7. If httpRequest's body is null and httpRequest's method is `POST` or `PUT`, then set contentLengthHeaderValue to `0`.
  if (
    httpRequest.body === null &&
    (httpRequest.method === "POST" || httpRequest.method === "PUT")
  ) {
    contentLengthHeaderValue = "0";
  }

  //  8. If contentLength is non-null, then set contentLengthHeaderValue to contentLength, serialized and isomorphic encoded.
  if (contentLength !== null) {
    contentLengthHeaderValue = String(contentLength);
  }

  //  9. If contentLengthHeaderValue is non-null, then append (`Content-Length`, contentLengthHeaderValue) to httpRequest's header list.
  if (contentLengthHeaderValue !== null) {
    setRequestHeader(httpRequest, "Content-Length", contentLengthHeaderValue);
  }

  // 10. If contentLength is non-null and httpRequest's keepalive is true, then:
  if (contentLength !== null && httpRequest.keepalive === true) {
    //  1. Let inflightKeepaliveBytes be 0.
    let inflightKeepaliveBytes = 0;

    //  2. Let group be httpRequest's client's fetch group.
    //  3. Let inflightRecords be the set of fetch records in group whose request's keepalive is true and done flag is unset.
    //  4. For each fetchRecord of inflightRecords:
    // TODO: Implement keepalive tracking

    //  5. If the sum of contentLength and inflightKeepaliveBytes is greater than 64 kibibytes, then return a network error.
    if (contentLength + inflightKeepaliveBytes > 65536) {
      return networkError();
    }
  }

  // 11. If httpRequest's referrer is a URL, then:
  if (httpRequest.referrer && typeof httpRequest.referrer !== "string") {
    //  1. Let referrerValue be httpRequest's referrer, serialized and isomorphic encoded.
    const referrerValue = httpRequest.referrer.href ||
      String(httpRequest.referrer);
    //  2. Append (`Referer`, referrerValue) to httpRequest's header list.
    setRequestHeader(httpRequest, "Referer", referrerValue);
  }

  // 12. Append a request `Origin` header for httpRequest.
  // TODO: Implement proper Origin header handling

  // 13. Append the Fetch metadata headers for httpRequest. [FETCH-METADATA]
  // TODO: Implement Fetch metadata headers

  // 14. If httpRequest's initiator is "prefetch", then set a structured field value given (`Sec-Purpose`, the token prefetch) in httpRequest's header list.
  if (httpRequest.initiator === "prefetch") {
    setRequestHeader(httpRequest, "Sec-Purpose", "prefetch");
  }

  // 15. If httpRequest's header list does not contain `User-Agent`, then user agents should append (`User-Agent`, default `User-Agent` value) to httpRequest's header list.
  if (!hasRequestHeader(httpRequest, "User-Agent")) {
    setRequestHeader(httpRequest, "User-Agent", "Andromeda/1.0");
  }

  // 16. If httpRequest's cache mode is "default" and httpRequest's header list contains `If-Modified-Since`, `If-None-Match`, `If-Unmodified-Since`, `If-Match`, or `If-Range`, then set httpRequest's cache mode to "no-store".
  if (
    httpRequest.cacheMode === "default" &&
    (hasRequestHeader(httpRequest, "If-Modified-Since") ||
      hasRequestHeader(httpRequest, "If-None-Match") ||
      hasRequestHeader(httpRequest, "If-Unmodified-Since") ||
      hasRequestHeader(httpRequest, "If-Match") ||
      hasRequestHeader(httpRequest, "If-Range"))
  ) {
    httpRequest.cacheMode = "no-store";
  }

  // 17. If httpRequest's cache mode is "no-cache", httpRequest's prevent no-cache cache-control header modification flag is unset, and httpRequest's header list does not contain `Cache-Control`, then append (`Cache-Control`, `max-age=0`) to httpRequest's header list.
  if (
    httpRequest.cacheMode === "no-cache" &&
    !httpRequest.preventNoCacheCacheControlHeaderModificationFlag &&
    !hasRequestHeader(httpRequest, "Cache-Control")
  ) {
    setRequestHeader(httpRequest, "Cache-Control", "max-age=0");
  }

  // 18. If httpRequest's cache mode is "no-store" or "reload", then:
  if (
    httpRequest.cacheMode === "no-store" ||
    httpRequest.cacheMode === "reload"
  ) {
    //  1. If httpRequest's header list does not contain `Pragma`, then append (`Pragma`, `no-cache`) to httpRequest's header list.
    if (!hasRequestHeader(httpRequest, "Pragma")) {
      setRequestHeader(httpRequest, "Pragma", "no-cache");
    }
    //  2. If httpRequest's header list does not contain `Cache-Control`, then append (`Cache-Control`, `no-cache`) to httpRequest's header list.
    if (!hasRequestHeader(httpRequest, "Cache-Control")) {
      setRequestHeader(httpRequest, "Cache-Control", "no-cache");
    }
  }

  // 19. If httpRequest's header list contains `Range`, then append (`Accept-Encoding`, `identity`) to httpRequest's header list.
  if (hasRequestHeader(httpRequest, "Range")) {
    setRequestHeader(httpRequest, "Accept-Encoding", "identity");
  }

  // 20. Modify httpRequest's header list per HTTP. Do not append a given header if httpRequest's header list contains that header's name.
  // TODO: Implement additional HTTP headers (Accept-Encoding, Connection, DNT, Host)

  // 21. If includeCredentials is true, then:
  if (includeCredentials) {
    //  1. Append a request `Cookie` header for httpRequest.
    // TODO: Implement Cookie handling

    //  2. If httpRequest's header list does not contain `Authorization`, then:
    if (!hasRequestHeader(httpRequest, "Authorization")) {
      // TODO: Implement Authorization header handling
    }
  }

  // 22. If there's a proxy-authentication entry, use it as appropriate.
  // TODO: Implement proxy authentication

  // 23. Set httpCache to the result of determining the HTTP cache partition, given httpRequest.
  // TODO: Implement cache partitioning
  httpCache = null;

  // 24. If httpCache is null, then set httpRequest's cache mode to "no-store".
  if (httpCache === null) {
    httpRequest.cacheMode = "no-store";
  }

  // 25. If httpRequest's cache mode is neither "no-store" nor "reload", then:
  if (
    httpRequest.cacheMode !== "no-store" &&
    httpRequest.cacheMode !== "reload"
  ) {
    // TODO: Implement cache logic (steps 1 and 2)
    // This includes cache lookup, stale-while-revalidate handling, and cache validation
  }

  // 9. If aborted, then return the appropriate network error for fetchParams.
  if (fetchParams.controller?.state === "aborted") {
    return networkError();
  }

  // 10. If response is null, then:
  if (response === null) {
    // 10.1. If httpRequest's cache mode is "only-if-cached", then return a network error.
    if (httpRequest.cacheMode === "only-if-cached") {
      return networkError();
    }

    // 10.2. Let forwardResponse be the result of running HTTP-network fetch given httpFetchParams, includeCredentials, and isNewConnectionFetch.
    const forwardResponse = await httpNetworkFetch(
      httpFetchParams,
      includeCredentials,
      isNewConnectionFetch,
    );

    // 10.3. If httpRequest's method is unsafe and forwardResponse's status is in the range 200 to 399, inclusive, invalidate appropriate stored responses in httpCache, as per the "Invalidating Stored Responses" chapter of HTTP Caching, and set storedResponse to null. [HTTP-CACHING]
    if (
      [
        "POST",
        "PUT",
        "DELETE",
        "CONNECT",
        "OPTIONS",
        "TRACE",
        "PATCH",
      ].includes(httpRequest.method) &&
      forwardResponse?.status >= 200 &&
      forwardResponse?.status <= 399
    ) {
      // TODO: Invalidate cache
      storedResponse = null;
    }

    // 10.4. If the revalidatingFlag is set and forwardResponse's status is 304, then:
    if (revalidatingFlag && forwardResponse?.status === 304) {
      // TODO: Update stored response with validation
      // 10.4.1. Update storedResponse's header list using forwardResponse's header list
      // 10.4.2. Set response to storedResponse
      // 10.4.3. Set response's cache state to "validated"
    }

    // 10.5. If response is null, then:
    if (response === null) {
      // 10.5.1. Set response to forwardResponse.
      response = forwardResponse;
      // 10.5.2. Store httpRequest and forwardResponse in httpCache
      // TODO: Store in cache
    }
  }

  // 11. Set response's URL list to a clone of httpRequest's URL list.
  if (response && httpRequest.urlList) {
    response.urlList = [...httpRequest.urlList];
  }

  // 12. If httpRequest's header list contains `Range`, then set response's range-requested flag.
  if (hasRequestHeader(httpRequest, "Range")) {
    response.rangeRequestedFlag = true;
  }

  // 13. Set response's request-includes-credentials to includeCredentials.
  if (response) {
    response.requestIncludesCredentials = includeCredentials;
  }

  // 14. If response's status is 401, httpRequest's response tainting is not "cors", includeCredentials is true, and request's window is an environment settings object, then:
  if (
    response?.status === 401 &&
    httpRequest.responseTainting !== "cors" &&
    includeCredentials &&
    request.window
  ) {
    // TODO: Implement authentication challenge handling
    // This would show authentication dialog and retry request
  }

  // 15. If response's status is 407, then:
  if (response?.status === 407) {
    // TODO: Implement proxy authentication challenge handling
  }

  // 16. If all of the following are true
  //     - response's status is 421
  //     - isNewConnectionFetch is false
  //     - request's body is null, or request's body is non-null and request's body's source is non-null
  //     then:
  if (
    response?.status === 421 &&
    !isNewConnectionFetch &&
    (request.body === null || (request.body && request.body.source))
  ) {
    // TODO: Implement HTTP/2 connection coalescing retry logic
    // This would retry the request on a new connection
  }

  // 17. If isAuthenticationFetch is true, then create an authentication entry for request and the given realm.
  if (isAuthenticationFetch) {
    // TODO: Create authentication entry
  }

  // 18. Return response.
  return response;
};
