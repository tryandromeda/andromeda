// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// deno-lint-ignore-file no-explicit-any prefer-const no-unused-vars

// CORS and response filtering helpers (loaded from cors.ts and response_filter.ts)
const corsCheck = (globalThis as any).corsCheck;
const corsPreflightCheck = (globalThis as any).corsPreflightCheck;
const createCORSPreflightRequest =
  (globalThis as any).createCORSPreflightRequest;
const filterResponse = (globalThis as any).filterResponse;
const createOpaqueRedirectFilteredResponse =
  (globalThis as any).createOpaqueRedirectFilteredResponse;

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

function hexToUtf8(hex: string) {
  if (!hex) return "";
  const bytes = new Uint8Array(
    hex.match(/.{1,2}/g)!.map((b) => parseInt(b, 16)),
  );
  return new TextDecoder().decode(bytes);
}

const networkError = (opts?: { cause?: unknown }) => ({
  type: "error",
  status: 0,
  statusText: "",
  headersList: [],
  body: null,
  urlList: [],
  // Forwarded as `cause` on the TypeError the caller sees.
  cause: opts?.cause,
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
  // TODO Event
  constructor() {
    (this as any).dispatcher = {};
    (this as any).connection = null;
    (this as any).dump = false;
    (this as any).state = "ongoing";
  }
}

/**
 * Implementation of the fetch API for Andromeda
 * https://developer.mozilla.org/ja/docs/Web/API/Window/fetch
 * @see https://fetch.spec.whatwg.org/#fetch-method/
 *
 * The fetch(input, init) method steps are
 */
const andromedaFetch = (input: RequestInfo, init = undefined) => {
  let p = createDeferredPromise();

  let requestObject: any;
  let request: any;

  try {
    // Create a Request object
    requestObject = new Request(input, init);

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

  // TODO abortFetch
  //



  let responseObject = null;

  let locallyAborted = false;

  let controller = null;

  // TODO abort controller

  controller = fetching({
    request,
    processResponse: (response: any) => {
      if (locallyAborted) {
        return;
      }

      if (response?.type === "error") {
        const err = response.cause !== undefined ?
          new TypeError("Network error", { cause: response.cause } as any) :
          new TypeError("Network error");
        p.reject(err);
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
          const keys = Object.keys(response.body).filter(k =>
            !isNaN(Number(k))
          );
          if (keys.length > 0) {
            const length = keys.length;
            bodyData = new Uint8Array(length);
            for (let i = 0; i < length; i++) {
              bodyData[i] = response.body[i] || 0;
            }
          }
        }
      }

      // Internal header pairs avoid unsupported Headers iteration and isolate later mutations.
      const responseHeadersList: [string, string][] =
        Array.isArray(response.headersList) ?
          response.headersList.map(([k, v]: [string, string]) =>
            [k, v] as [string, string]
          ) :
          [];
      responseObject = new Response(bodyData, {
        status: response.status,
        statusText: response.statusText,
        headersList: responseHeadersList,
      } as any);

      // response.url is the last URL in the chain, not the first.
      const lastUrl = response.urlList && response.urlList.length > 0 ?
        response.urlList[response.urlList.length - 1] :
        null;
      const lastUrlString = lastUrl ?
        (typeof lastUrl === "string" ?
          lastUrl :
          (lastUrl.href || lastUrl.serialized || String(lastUrl))) :
        request.url;
      Object.defineProperty(responseObject, "url", {
        value: lastUrlString,
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

  return p.promise;
};

globalThis.fetch = andromedaFetch;
/**
 * @see https://fetch.spec.whatwg.org/#fetch-response-handover
 */
const fetchResponseHandover = (fetchParams: any, response: any) => {
  // The outer rejection handler can retry handover, so guard against delivering a second response.
  if (fetchParams.handedOff) return;
  fetchParams.handedOff = true;
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
  if (request.mode === "navigate") {
    throw new Error("error");
  }

  let taskDestination = null;

  let crossOriginIsolatedCapability = false;


  if (request.client != null) {
    taskDestination = request.client.globalObject;
    crossOriginIsolatedCapability =
      request.client.crossOriginIsolatedCapability;
  }

  // TODO

  // TODO
  let timingInfo = 0;

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

  // TODO step9, 10


  // TODO



  // TODO

  // The implementation-defined object could encompass stream weight and dependency for HTTP/2, priorities used
  //       in Extensible Prioritization Scheme for HTTP for transports where it applies (including HTTP/3),
  //       and equivalent information used to prioritize dispatch and processing of HTTP/1 fetches. [RFC9218]

  mainFetch(fetchParams).catch((err: unknown) => {
    fetchResponseHandover(fetchParams, networkError({ cause: err }));
  });

  return fetchParams.controller;
};

/**
 * To main fetch, given a fetch params fetchParams and an optional boolean recursive (default false)
 * @see https://fetch.spec.whatwg.org/#main-fetch
 */
const mainFetch = async (fetchParams: any, recursive = false) => {
  const request = fetchParams.request;

  let response: any = null;

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

  // TODO Implement CSP violation reporting

  // TODO Implement mixed content upgrade


  // Check if fetching should be blocked as mixed content
  const shouldBlockMixedContent = (globalThis as unknown as {
    __shouldFetchingRequestBeBlockedAsMixedContent?: (
      req: unknown,
    ) => "allowed" | "blocked";
  }).__shouldFetchingRequestBeBlockedAsMixedContent;

  if (shouldBlockMixedContent) {
    const result = shouldBlockMixedContent(request);
    if (result === "blocked") {
      return networkError();
    }
  }

  // Restricted ports prevent fetch from reaching unsafe network services.
  const badPorts = new Set([
    1,
    7,
    9,
    11,
    13,
    15,
    17,
    19,
    20,
    21,
    22,
    23,
    25,
    37,
    42,
    43,
    53,
    69,
    77,
    79,
    87,
    95,
    101,
    102,
    103,
    104,
    109,
    110,
    111,
    113,
    115,
    117,
    119,
    123,
    135,
    137,
    139,
    143,
    161,
    179,
    389,
    427,
    465,
    512,
    513,
    514,
    515,
    526,
    530,
    531,
    532,
    540,
    548,
    554,
    556,
    563,
    587,
    601,
    636,
    989,
    990,
    993,
    995,
    1719,
    1720,
    1723,
    2049,
    3659,
    4045,
    5060,
    5061,
    6000,
    6566,
    6665,
    6666,
    6667,
    6668,
    6669,
    6697,
    10080,
  ]);

  const requestURL = new URL(request.url);
  if (requestURL.port && badPorts.has(parseInt(requestURL.port, 10))) {
    return networkError();
  }

  // TODO Implement CSP check
  // TODO Implement Integrity Policy check

  if (request.referrerPolicy === "") {
    request.referrerPolicy = request.policyContainer?.referrerPolicy ||
      "strict-origin-when-cross-origin";
  }

  if (request.referrer !== "no-referrer") {
    request.referrer = request.referrer;
  }


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

  const runRemainingSteps = async () => {
    if (response === null) {
      // ↪︎ fetchParams's preloaded response candidate is non-null
      if (
        fetchParams.preloadedResponseCandidate !== null &&
        fetchParams.preloadedResponseCandidate !== undefined
      ) {
        let loopCount = 0;
        while (fetchParams.preloadedResponseCandidate === "pending") {
          loopCount++;
          if (loopCount > 1000) {
            console.error("Infinite loop detected in preloaded response wait");
            response = networkError();
            break;
          }
        }
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
        request.responseTainting = "basic";
        response = await schemeFetch(fetchParams);
      } // ↪︎ request's mode is "same-origin"
      else if (request.mode === "same-origin") {
        //    Return a network error.
        response = networkError();
      } // ↪︎ request's mode is "no-cors"
      else if (request.mode === "no-cors") {
        if (request.redirectMode !== "follow") {
          response = networkError();
        } else {
          request.responseTainting = "opaque";
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
        request.responseTainting = "cors";
        const corsWithPreflightResponse = await httpFetch(fetchParams, true);
        if (corsWithPreflightResponse?.type === "error") {
          // Clear cache entries (no-op for now)
        }
        response = corsWithPreflightResponse;
      } // ↪ Otherwise
      else {
        request.responseTainting = "cors";
        response = await httpFetch(fetchParams);
      }
    }

    if (recursive) {
      return response;
    }

    const responseIsValid = response && response.type !== "error";
    const responseIsFiltered = response &&
      (response.type === "basic" ||
        response.type === "cors" ||
        response.type === "opaque");
    const shouldProcessComplexResponse = responseIsValid && !responseIsFiltered;

    if (shouldProcessComplexResponse) {
      //  1.If request's response tainting is "cors", then
      if (request.responseTainting === "cors") {
        const headerNames: string | null = null;
        if (
          request.credentialsMode !== "include" &&
          headerNames?.includes("*")
        ) {
          response.CORSExposedHeaderNameList = [];
        } //    3. Otherwise, if headerNames is non-null or failure, then set response's CORS-exposed header-name list to headerNames.
        else if (headerNames !== null && headerNames !== "failure") {
          response.CORSExposedHeaderNameList = headerNames;
        }
        // One of the headerNames can still be `*` at this point, but will only match a header whose name is `*`.
      }

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

    let internalResponse = response?.type === "error" ?
      response :
      response?.internalResponse || response;

    // Some non-network responses have no URL history.
    if (
      internalResponse &&
      (!internalResponse.urlList || internalResponse.urlList.length === 0)
    ) {
      internalResponse.urlList = [...request.urlList];
    }

    if (internalResponse) {
      internalResponse.redirectTaint = request.redirectTaint;
    }

    if (!request.timingAllowFailedFlag && internalResponse) {
      internalResponse.timingAllowPassedFlag = true;
    }

    if (response?.type !== "error" && internalResponse) {
      let shouldBlock = false;

      // Check if response should be blocked as mixed content
      const shouldBlockMixedContentResponse = (globalThis as unknown as {
        __shouldResponseToRequestBeBlockedAsMixedContent?: (
          req: unknown,
          res: unknown,
        ) => "allowed" | "blocked";
      }).__shouldResponseToRequestBeBlockedAsMixedContent;

      if (shouldBlockMixedContentResponse) {
        const result = shouldBlockMixedContentResponse(
          request,
          internalResponse,
        );
        if (result === "blocked") {
          shouldBlock = true;
        }
      }

      // TODO Implement CSP check
      // TODO Implement MIME type check
      // TODO Implement nosniff check

      if (shouldBlock) {
        response = networkError();
      }
    }

    if (
      response?.type === "opaque" &&
      internalResponse?.status === 206 &&
      internalResponse?.rangeRequestedFlag &&
      !request.headersList?.contains?.("Range")
    ) {
      response = internalResponse = networkError();
    }

    if (response?.type !== "error" && internalResponse) {
      if (
        request.method === "HEAD" ||
        request.method === "CONNECT" ||
        [101, 103, 204, 205, 304].includes(internalResponse.status)
      ) {
        internalResponse.body = null;
      }
    }

    const sriHook = (globalThis as any).__doesResponseMatchIntegrityMetadata;
    if (
      request.integrity && request.integrity !== "" &&
      typeof sriHook === "function"
    ) {
      const processBodyError = (err?: unknown) => {
        fetchResponseHandover(
          fetchParams,
          networkError(err !== undefined ? { cause: err } : undefined),
        );
      };

      if (response.body === null) {
        processBodyError();
        return;
      }

      const processBody = async (bytes: Uint8Array) => {
        const doesMatch =
          (globalThis as any).__doesResponseMatchIntegrityMetadata;
        if (doesMatch) {
          const integrityValid = await doesMatch(
            {
              integrity: request.integrity,
              origin: request.origin,
              mode: request.mode,
            },
            { body: bytes, type: response.type, url: response.urlList?.[0] },
          );
          if (!integrityValid) {
            processBodyError();
            return;
          }
        }
        response.body = bytes;
        fetchResponseHandover(fetchParams, response);
      };

      processBody(new Uint8Array()).catch((err: unknown) => {
        processBodyError(err);
      });
    } else {
      fetchResponseHandover(fetchParams, response);
    }
  };

  // Keep the chain alive until the response callback fires.
  return await runRemainingSteps();
};

/**
 * @see https://fetch.spec.whatwg.org/#scheme-fetch
 * @description To scheme fetch, given a fetch params fetchParams
 */
const schemeFetch = async (fetchParams: any) => {
  if (fetchParams.controller?.state === "aborted") {
    return networkError();
  }

  const request = fetchParams.request;


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
      return networkError();
    }

    // ↪︎ "blob"
    case "blob": {
      const blobURLEntry = null;

      if (request.method !== "GET" || blobURLEntry === null) {
        return networkError();
      }

      // TODO Implement determining the environment
      const requestEnvironment = null;

      const isTopLevelNavigation = request.destination === "document";

      if (!isTopLevelNavigation && requestEnvironment === null) {
        return networkError();
      }

      const navigationOrEnvironment = isTopLevelNavigation ?
        "navigation" :
        requestEnvironment;

      // TODO Implement obtaining blob object
      const blob = null;

      if (
        !(blob && typeof blob === "object" && "size" in blob && "type" in blob)
      ) {
        return networkError();
      }

      const response: any = {
        type: "basic",
        status: 200,
        statusText: "OK",
        headersList: [],
        body: null,
        urlList: [request.currentURL],
      };

      const fullLength = blob.size;

      const serializedFullLength = String(fullLength);

      const type = blob.type || "";

      if (
        !(
          request.headersList &&
          typeof request.headersList.contains === "function" &&
          request.headersList.contains("Range")
        )
      ) {
        const bodyWithType = { body: new Uint8Array(), type: blob.type };
        response.statusText = "OK";
        response.body = bodyWithType.body;
        response.headersList = [
          ["Content-Length", serializedFullLength],
          ["Content-Type", type],
        ];
      } //   14. Otherwise:
      else {
        response.rangeRequestedFlag = true;
        const rangeHeader =
          request.headersList && typeof request.headersList.get === "function" ?
            request.headersList.get("Range") :
            null;
        const rangeValue = rangeHeader ? [0, 100] : null;
        if (rangeValue === null) {
          return networkError();
        }
        let [rangeStart, rangeEnd] = rangeValue;
        if (rangeStart === null) {
          rangeStart = fullLength - rangeEnd!;
          rangeEnd = rangeStart + rangeEnd! - 1;
        } //      7. Otherwise:
        else {
          if (rangeStart >= fullLength) {
            return networkError();
          }
          if (rangeEnd === null || rangeEnd >= fullLength) {
            rangeEnd = fullLength - 1;
          }
        }
        const slicedBlob = { size: rangeEnd + 1 - rangeStart, type };
        const slicedBodyWithType = {
          body: new Uint8Array(),
          type: slicedBlob.type,
        };
        response.body = slicedBodyWithType.body;
        const serializedSlicedLength = String(slicedBlob.size);
        const contentRange = `bytes ${rangeStart}-${rangeEnd}/${fullLength}`;
        response.status = 206;
        response.headersList = [
          ["Content-Length", serializedSlicedLength],
          ["Content-Type", type],
          ["Content-Range", contentRange],
        ];
      }
      return response;
    }

    // ↪︎ "data"
    case "data": {
      const urlString = request.currentURL.href || String(request.currentURL);
      let dataURLStruct = null;

      if (urlString.startsWith("data:")) {
        const commaIndex = urlString.indexOf(",");
        if (commaIndex !== -1) {
          let mimeTypeStr = urlString.substring(5, commaIndex).trim();
          const dataStr = urlString.substring(commaIndex + 1);

          // Check if base64 encoded
          const isBase64 = mimeTypeStr.endsWith(";base64");
          if (isBase64) {
            mimeTypeStr = mimeTypeStr.slice(0, -7).trim(); // Remove ";base64"
          }

          // Default MIME type if not specified
          const mimeType = mimeTypeStr || "text/plain;charset=US-ASCII";

          let body: Uint8Array;

          if (isBase64) {
            // Base64 decode
            try {
              // Remove whitespace from base64 string
              const cleanedData = dataStr.replace(/\s/g, "");
              // Use atob for base64 decoding
              const binaryString = atob(cleanedData);
              body = new Uint8Array(binaryString.length);
              for (let i = 0; i < binaryString.length; i++) {
                body[i] = binaryString.charCodeAt(i);
              }
            } catch (e) {
              // Invalid base64
              return networkError();
            }
          } else {
            // Percent-decode the data
            try {
              const decoded = decodeURIComponent(dataStr);
              body = new TextEncoder().encode(decoded);
            } catch (e) {
              // Invalid percent-encoding, use as-is
              body = new TextEncoder().encode(dataStr);
            }
          }

          dataURLStruct = { mimeType, body };
        }
      }

      if (dataURLStruct === null) {
        return networkError();
      }
      const mimeType = dataURLStruct.mimeType;
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

      try {
        const fileURL = new URL(
          request.currentURL.href || String(request.currentURL),
        );

        let filePath = decodeURIComponent(fileURL.pathname);

        // File fetches must not mutate the local filesystem.
        if (request.method !== "GET") {
          return networkError();
        }

        // Read the file asynchronously
        const readFileAsync = (__andromeda__ as any).internal_read_file_async;
        if (!readFileAsync) {
          return networkError();
        }

        const body = await readFileAsync(filePath);

        // Detect MIME type from file extension
        const getMimeType = (path: string): string => {
          const ext = path.split(".").pop()?.toLowerCase();
          const mimeTypes: Record<string, string> = {
            "html": "text/html",
            "htm": "text/html",
            "css": "text/css",
            "js": "text/javascript",
            "mjs": "text/javascript",
            "json": "application/json",
            "txt": "text/plain",
            "xml": "application/xml",
            "png": "image/png",
            "jpg": "image/jpeg",
            "jpeg": "image/jpeg",
            "gif": "image/gif",
            "svg": "image/svg+xml",
            "ico": "image/x-icon",
            "pdf": "application/pdf",
            "zip": "application/zip",
            "wasm": "application/wasm",
          };
          return mimeTypes[ext || ""] || "application/octet-stream";
        };

        const mimeType = getMimeType(filePath);

        return {
          type: "basic",
          status: 200,
          statusText: "OK",
          headersList: [
            ["Content-Type", mimeType],
            ["Content-Length", String(body.length)],
          ],
          body: body,
          urlList: [request.currentURL],
        };
      } catch (error) {
        // File not found, permission denied, or other error
        return networkError();
      }
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

  return networkError();
};

/**
 * @see https://fetch.spec.whatwg.org/#http-fetch
 * @description To HTTP fetch, given a fetch params fetchParams and an optional boolean makeCORSPreflight (default false), run these steps
 */

const httpFetch = async (fetchParams: any, makeCORSPreflight = false) => {
  const request = fetchParams.request;

  let response: any = null;
  let internalResponse: any = null;

  if (request.serviceWorkersMode === "all") {
    // TODO Service Workerl
  }

  if (response === null) {
    if (
      makeCORSPreflight &&
      (!["GET", "HEAD", "POST"].includes(request.method) ||
        request.useCORSPreflightFlag ||
        false)
    ) {
      const preflightResponse = { type: "basic", status: 200 };
      if (preflightResponse?.type === "error") {
        return preflightResponse;
      }
    }

    if (request.redirectMode === "follow") {
      request.serviceWorkersMode = "none";
    }

    const fetchResult = await httpNetworkOrCacheFetch(fetchParams);
    response = internalResponse = fetchResult;

    if (
      request.responseTainting === "cors" &&
      !(response?.status === 304 || response?.status === 407 || true)
    ) {
      return networkError();
    }

    if (!true) {
      request.timingAllowFailedFlag = true;
    }
  }

  if (
    (request.responseTainting === "opaque" || response?.type === "opaque")
  ) {
    // Perform CORP check
    const corpCheck = (globalThis as unknown as {
      __corpCheck?: (req: unknown, res: unknown) => boolean;
    }).__corpCheck;
    if (corpCheck && internalResponse) {
      const corpAllowed = corpCheck(request, internalResponse);
      if (!corpAllowed) {
        return networkError();
      }
    }
  }

  if (
    internalResponse?.status &&
    [301, 302, 303, 307, 308].includes(internalResponse.status)
  ) {
    // TODO HTTP/2 RST_STREAM処理

    switch (request.redirectMode) {
      // ↪︎ "error"
      case "error":
        response = networkError();
        break;

      // ↪︎ "manual"
      case "manual":
        if (request.mode === "navigate") {
          // Set up the manual redirect steps to be invoked later by navigation
          if (fetchParams.controller) {
            fetchParams.controller.nextManualRedirectSteps = async () => {
              return await httpRedirectFetch(fetchParams, response);
            };
          }
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
        response = await httpRedirectFetch(fetchParams, response);
        break;
    }
  }

  return response;
};

/**
 * 4.6. HTTP-network fetch
 * @see SPEC https://fetch.spec.whatwg.org/#http-redirect-fetch
 * @description To HTTP-network fetch, given a fetch params fetchParams, an optional boolean includeCredentials (default false), and an optional boolean forceNewConnection (default false), run these steps
 */
const httpNetworkFetch = async (
  fetchParams: any,
  includeCredentials = false,
  forceNewConnection = false,
) => {
  const request = fetchParams.request;

  let response: any = null;

  const timingInfo = fetchParams.timingInfo;

  const networkPartitionKey = request.currentURL.origin; // Simplified implementation

  const newConnection = forceNewConnection ? "yes" : "no";

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


  // Check if fetchParams is canceled
  if (fetchParams.controller?.state === "aborted") {
    return networkError();
  }

  if (!connection) {
    return networkError();
  }

  if (timingInfo) {
    timingInfo.finalConnectionTimingInfo = Date.now();
  }

  if (
    connection.type === "http" &&
    request.body !== null &&
    request.body?.source === null
  ) {
    return networkError();
  }

  if (timingInfo) {
    timingInfo.finalNetworkRequestStartTime = Date.now();
  }

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

        // Track user-supplied header names so defaults below only fill gaps.
        const userHeaderNames = new Set<string>();
        if (headers && Array.isArray(headers)) {
          for (const [name, value] of headers) {
            userHeaderNames.add(name.toLowerCase());
            httpRequest += `${name}: ${value}\r\n`;
          }
        }

        // Defaults only fire when the caller did not supply them, and also
        // land on request.headersList so a redirect carries them through.
        if (!userHeaderNames.has("accept")) {
          httpRequest += `Accept: */*\r\n`;
          if (Array.isArray(request.headersList)) {
            request.headersList.push(["Accept", "*/*"]);
          }
        }
        if (!userHeaderNames.has("user-agent")) {
          httpRequest += `User-Agent: Andromeda/0.1\r\n`;
          if (Array.isArray(request.headersList)) {
            request.headersList.push(["User-Agent", "Andromeda/0.1"]);
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

        // Read response. The 10_000 chunk cap (~40 MB at 4096 B/chunk)
        // bounds runaway streams; the loop exits earlier on EOF.
        let responseHex = "";
        const maxRetries = 10_000;
        let retries = 0;
        let readError: unknown = null;

        while (retries < maxRetries) {
          try {
            const chunk = await __andromeda__.internal_tls_read(rid, 4096);
            if (!chunk || chunk.length === 0) break;
            responseHex += chunk;
          } catch (e) {
            readError = e;
            break;
          }
          retries++;
        }

        await __andromeda__.internal_tls_close(rid);

        if (readError !== null) {
          return networkError({ cause: readError });
        }

        // Convert hex to bytes
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
        } else {
          // Invalid HTTP data must not become a successful response.
          return networkError({
            cause: new Error(
              "Invalid or empty HTTP response (no status line)",
            ),
          });
        }

        // Parse headers
        let isChunked = false;
        for (let i = 1; i < lines.length; i++) {
          const line = lines[i];
          const colonIndex = line.indexOf(":");
          if (colonIndex > 0) {
            const name = line.substring(0, colonIndex).trim();
            const value = line.substring(colonIndex + 1).trim();
            responseHeaders.push([name, value]);
            if (
              name.toLowerCase() === "transfer-encoding" &&
              value.toLowerCase().split(",").map((s) => s.trim()).includes(
                "chunked",
              )
            ) {
              isChunked = true;
            }
          }
        }

        // Strip HTTP/1.1 chunked framing from the body.
        let decodedBody = bodyText;
        if (isChunked && bodyText) {
          decodedBody = "";
          let p = 0;
          while (p < bodyText.length) {
            const crlf = bodyText.indexOf("\r\n", p);
            if (crlf < 0) break;
            const sizeHex = bodyText.slice(p, crlf).split(";")[0].trim();
            if (sizeHex === "") {
              p = crlf + 2;
              continue;
            }
            const size = parseInt(sizeHex, 16);
            if (!Number.isFinite(size) || size < 0) break;
            if (size === 0) break; // last-chunk (trailers ignored)
            const dataStart = crlf + 2;
            decodedBody += bodyText.slice(dataStart, dataStart + size);
            p = dataStart + size + 2; // skip data + trailing CRLF
          }
        }

        // Set response body
        if (decodedBody) {
          responseBody = new TextEncoder().encode(decodedBody);
        }

        // Drop the chunked token from Transfer-Encoding (other codings like
        // gzip stay; if it was the only token, drop the header entirely).
        // Then set Content-Length to the decoded byte length.
        if (isChunked) {
          const decodedLen = responseBody ? responseBody.length : 0;
          const rewritten: [string, string][] = [];
          for (const [name, value] of responseHeaders) {
            if (name.toLowerCase() !== "transfer-encoding") {
              rewritten.push([name, value]);
              continue;
            }
            const remaining = value
              .split(",")
              .map((s) => s.trim())
              .filter((s) => s.toLowerCase() !== "chunked");
            if (remaining.length > 0) {
              rewritten.push([name, remaining.join(", ")]);
            }
          }
          responseHeaders = rewritten;
          const clIndex = responseHeaders.findIndex(([name]) =>
            name.toLowerCase() === "content-length"
          );
          if (clIndex >= 0) {
            responseHeaders[clIndex] = ["Content-Length", String(decodedLen)];
          } else {
            responseHeaders.push(["Content-Length", String(decodedLen)]);
          }
        }
      } catch (error) {
        return networkError({ cause: error });
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
    return networkError({ cause: error });
  }

  // TODO Support TLS client certificate selection.

  // To transmit request's body, run these steps
  try {
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
      const processBodyChunk = (bytes: Uint8Array) => {
        if (fetchParams.controller?.state === "aborted") {
          return;
        }
        // TODO Transmit request body chunks over the connection.
        if (fetchParams.processRequestBodyChunkLength) {
          fetchParams.processRequestBodyChunkLength(bytes.length);
        }
      };

      const processEndOfBody = () => {
        if (fetchParams.controller?.state === "aborted") {
          return;
        }
        if (fetchParams.processRequestEndOfBody) {
          fetchParams.processRequestEndOfBody();
        }
      };

      const processBodyError = (e: any) => {
        if (fetchParams.controller?.state === "aborted") {
          return;
        }
        if (e?.name === "AbortError") {
          if (fetchParams.controller?.abort) {
            fetchParams.controller.abort();
          }
        } else {
          if (fetchParams.controller?.terminate) {
            fetchParams.controller.terminate();
          }
        }
      };

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

  if (fetchParams.controller?.state === "aborted") {
    // TODO Signal cancellation to the HTTP/2 peer.
    return networkError();
  }

  let buffer = new Uint8Array(0);

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

  const cancelAlgorithm = (reason: any) => {
    if (fetchParams.controller?.abort) {
      fetchParams.controller.abort(reason);
    }
  };

  if (response && response.body) {
    // Response body is already set from earlier processing
  }

  if (includeCredentials && response) {
    // TODO Process Set-Cookie headers.
    const setCookieHeaders = response.headersList.filter(
      ([name]: [string, string]) => name.toLowerCase() === "set-cookie",
    );
    // Store cookies (simplified)
  }

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

  return response || networkError();
};
/**
 * @see https://fetch.spec.whatwg.org/#http-redirect-fetch
 * @description To HTTP-redirect fetch, given a fetch params fetchParams and a response response, run these steps
 */
const httpRedirectFetch = async (fetchParams: any, response: any) => {
  const request = fetchParams.request;

  const internalResponse = response?.internalResponse || response;

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

  if (locationURL === null) {
    return response;
  }

  if (locationURL.protocol !== "http:" && locationURL.protocol !== "https:") {
    return networkError();
  }

  if (!request.redirectCount) {
    request.redirectCount = 0;
  }
  if (request.redirectCount >= 20) {
    return networkError();
  }

  request.redirectCount++;

  // Clone before the redirect-specific deletes below. The internal request
  // shares headers with the user's Request object, so mutating in place
  // would leak Authorization / body-header drops back to the caller.
  if (request.headers instanceof Headers) {
    const originalGuard = (request.headers as any).guard;
    const cloned = new Headers();
    request.headers.forEach((value: string, name: string) => {
      cloned.append(name, value);
    });
    // Carry the guard over; new Headers() starts at "none".
    if (originalGuard && (globalThis as any).setHeadersGuard) {
      (globalThis as any).setHeadersGuard(cloned, originalGuard);
    }
    request.headers = cloned;
  }
  if (Array.isArray(request.headersList)) {
    request.headersList = [...request.headersList];
  }

  if (
    request.mode === "cors" &&
    (locationURL.username || locationURL.password) &&
    request.origin !== locationURL.origin
  ) {
    return networkError();
  }

  if (
    request.responseTainting === "cors" &&
    (locationURL.username || locationURL.password)
  ) {
    return networkError();
  }

  if (
    internalResponse.status !== 303 &&
    request.body !== null &&
    request.body?.source === null
  ) {
    return networkError();
  }

  if (
    ((internalResponse.status === 301 || internalResponse.status === 302) &&
      request.method === "POST") ||
    (internalResponse.status === 303 &&
      request.method !== "GET" &&
      request.method !== "HEAD")
  ) {
    request.method = "GET";
    request.body = null;

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
        ([name]: [string, string]) =>
          !requestBodyHeaders.includes(name.toLowerCase()),
      );
    }
  }

  if (request.currentURL.origin !== locationURL.origin) {
    // CORS non-wildcard request-header names include at minimum "authorization"
    const corsNonWildcardHeaders = ["authorization"];
    if (request.headers instanceof Headers) {
      for (const header of corsNonWildcardHeaders) {
        request.headers.delete(header);
      }
    } else if (request.headersList && Array.isArray(request.headersList)) {
      request.headersList = request.headersList.filter(
        ([name]: [string, string]) =>
          !corsNonWildcardHeaders.includes(name.toLowerCase()),
      );
    }
  }

  if (request.body !== null && request.body.source) {
    // TODO Recreate the body safely for redirected requests.
  }

  const timingInfo = fetchParams.timingInfo;

  if (timingInfo) {
    timingInfo.redirectEndTime = Date.now();
    timingInfo.postRedirectStartTime = Date.now();
  }

  if (timingInfo && timingInfo.redirectStartTime === 0) {
    timingInfo.redirectStartTime = timingInfo.startTime;
  }

  if (!request.urlList) {
    request.urlList = [];
  }
  request.urlList.push(locationURL);

  const referrerPolicyHeader = internalResponse.headersList?.find(
    ([name]: [string, string]) => name.toLowerCase() === "referrer-policy",
  );
  if (referrerPolicyHeader && referrerPolicyHeader[1]) {
    const policy = referrerPolicyHeader[1].trim();
    // Valid referrer policies per spec
    const validPolicies = [
      "no-referrer",
      "no-referrer-when-downgrade",
      "same-origin",
      "origin",
      "strict-origin",
      "origin-when-cross-origin",
      "strict-origin-when-cross-origin",
      "unsafe-url",
    ];
    if (validPolicies.includes(policy)) {
      request.referrerPolicy = policy;
    }
  }

  let recursive = true;

  if (request.redirectMode === "manual") {
    if (request.mode !== "navigate") {
      throw new Error(
        "Assertion failed: manual redirect mode requires navigate mode",
      );
    }
    recursive = false;
  }

  request.currentURL = locationURL;
  const redirectedResponse = await mainFetch(fetchParams, recursive);
  // Flag the response as redirected and carry the full URL list, so
  // Response.url ends up as the final URL rather than the original.
  if (redirectedResponse && redirectedResponse.type !== "error") {
    redirectedResponse.redirected = true;
    const internal = redirectedResponse.internalResponse || redirectedResponse;
    internal.redirected = true;
    internal.urlList = [...request.urlList];
    if (redirectedResponse !== internal) {
      redirectedResponse.urlList = [...request.urlList];
    }
  }
  return redirectedResponse;
};

/**
 * @see https://fetch.spec.whatwg.org/#http-network-or-cache-fetch
 * @description To HTTP-network-or-cache fetch, given a fetch params fetchParams, an optional boolean isAuthenticationFetch (default false), and an optional boolean isNewConnectionFetch (default false), run these steps
 */
const httpNetworkOrCacheFetch = async (
  fetchParams: any,
  isAuthenticationFetch = false,
  isNewConnectionFetch = false,
) => {
  const request = fetchParams.request;

  let httpFetchParams = null;

  let httpRequest = null;

  let response = null;

  let storedResponse = null;

  let httpCache = null;

  let revalidatingFlag = false;

  if (
    request.traversable === "no-traversable" &&
    request.redirectMode === "error"
  ) {
    httpFetchParams = fetchParams;
    httpRequest = request;
  } else {
    httpRequest = { ...request };
    // `new Headers(req)` would yield an empty Headers here (fillHeaders does
    // not iterate Headers instances), so clone via forEach + append.
    if (request.headers instanceof Headers) {
      const originalGuard = (request.headers as any).guard;
      const cloned = new Headers();
      request.headers.forEach((value: string, name: string) => {
        cloned.append(name, value);
      });
      if (originalGuard && (globalThis as any).setHeadersGuard) {
        (globalThis as any).setHeadersGuard(cloned, originalGuard);
      }
      httpRequest.headers = cloned;
    }
    if (Array.isArray(request.headersList)) {
      httpRequest.headersList = [...request.headersList];
    }
    httpFetchParams = { ...fetchParams };
    httpFetchParams.request = httpRequest;
  }

  let includeCredentials = httpRequest.credentialsMode === "include" ||
    (httpRequest.credentialsMode === "same-origin" &&
      httpRequest.responseTainting === "basic");

  // TODO Implement Cross-Origin-Embedder-Policy check

  const contentLength = httpRequest.body?.length || null;

  let contentLengthHeaderValue = null;

  if (
    httpRequest.body === null &&
    (httpRequest.method === "POST" || httpRequest.method === "PUT")
  ) {
    contentLengthHeaderValue = "0";
  }

  if (contentLength !== null) {
    contentLengthHeaderValue = String(contentLength);
  }

  if (contentLengthHeaderValue !== null) {
    setRequestHeader(httpRequest, "Content-Length", contentLengthHeaderValue);
  }

  if (contentLength !== null && httpRequest.keepalive === true) {
    let inflightKeepaliveBytes = 0;

    // TODO Implement keepalive tracking

    if (contentLength + inflightKeepaliveBytes > 65536) {
      return networkError();
    }
  }

  if (httpRequest.referrer && typeof httpRequest.referrer !== "string") {
    const referrerValue = httpRequest.referrer.href ||
      String(httpRequest.referrer);
    setRequestHeader(httpRequest, "Referer", referrerValue);
  }

  if (
    httpRequest.mode === "cors" ||
    (httpRequest.method !== "GET" && httpRequest.method !== "HEAD")
  ) {
    if (!hasRequestHeader(httpRequest, "Origin")) {
      const origin = httpRequest.origin || "null";
      setRequestHeader(httpRequest, "Origin", origin);
    }
  }

  if (!hasRequestHeader(httpRequest, "Sec-Fetch-Site")) {
    const requestOrigin = httpRequest.origin;
    const targetOrigin =
      new URL(httpRequest.currentURL.href || httpRequest.currentURL).origin;
    let site = "same-origin";
    if (requestOrigin !== targetOrigin) {
      site = "cross-site"; // TODO Check for same-site requests.
    }
    setRequestHeader(httpRequest, "Sec-Fetch-Site", site);
  }

  if (!hasRequestHeader(httpRequest, "Sec-Fetch-Mode")) {
    setRequestHeader(httpRequest, "Sec-Fetch-Mode", httpRequest.mode || "cors");
  }

  if (!hasRequestHeader(httpRequest, "Sec-Fetch-Dest")) {
    setRequestHeader(
      httpRequest,
      "Sec-Fetch-Dest",
      httpRequest.destination || "empty",
    );
  }

  if (httpRequest.initiator === "prefetch") {
    setRequestHeader(httpRequest, "Sec-Purpose", "prefetch");
  }

  if (!hasRequestHeader(httpRequest, "User-Agent")) {
    setRequestHeader(httpRequest, "User-Agent", "Andromeda/1.0");
  }

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

  if (
    httpRequest.cacheMode === "no-cache" &&
    !httpRequest.preventNoCacheCacheControlHeaderModificationFlag &&
    !hasRequestHeader(httpRequest, "Cache-Control")
  ) {
    setRequestHeader(httpRequest, "Cache-Control", "max-age=0");
  }

  if (
    httpRequest.cacheMode === "no-store" ||
    httpRequest.cacheMode === "reload"
  ) {
    if (!hasRequestHeader(httpRequest, "Pragma")) {
      setRequestHeader(httpRequest, "Pragma", "no-cache");
    }
    if (!hasRequestHeader(httpRequest, "Cache-Control")) {
      setRequestHeader(httpRequest, "Cache-Control", "no-cache");
    }
  }

  if (
    hasRequestHeader(httpRequest, "Range") &&
    !hasRequestHeader(httpRequest, "Accept-Encoding")
  ) {
    setRequestHeader(httpRequest, "Accept-Encoding", "identity");
  }

  // TODO Implement additional HTTP headers (Accept-Encoding, Connection, DNT, Host)

  if (includeCredentials) {
    const generateCookieHeader = (globalThis as any).generateCookieHeader;
    if (generateCookieHeader && !hasRequestHeader(httpRequest, "Cookie")) {
      const cookieHeader = generateCookieHeader(
        httpRequest.currentURL.href || String(httpRequest.currentURL),
      );
      if (cookieHeader) {
        setRequestHeader(httpRequest, "Cookie", cookieHeader);
      }
    }

    if (!hasRequestHeader(httpRequest, "Authorization")) {
      // Check if we have stored credentials for this origin
      const getStoredCredentials = (globalThis as any).getStoredCredentials;
      if (getStoredCredentials) {
        const requestURL = new URL(
          httpRequest.currentURL.href || String(httpRequest.currentURL),
        );
        const origin = requestURL.origin;

        // Try to get credentials for common realm (empty string means any realm)
        const credentials = getStoredCredentials(origin, "");
        if (credentials) {
          // Generate Basic auth header from stored credentials
          const generateBasicAuth = (globalThis as any).generateBasicAuth;
          if (generateBasicAuth) {
            const authHeader = generateBasicAuth(
              credentials.username,
              credentials.password,
            );
            setRequestHeader(httpRequest, "Authorization", authHeader);
          }
        }
      }
    }
  }

  // TODO Implement proxy authentication

  // TODO Implement cache partitioning
  httpCache = null;

  if (httpCache === null) {
    httpRequest.cacheMode = "no-store";
  }

  if (
    httpRequest.cacheMode !== "no-store" &&
    httpRequest.cacheMode !== "reload"
  ) {
    // TODO Implement cache logic (steps 1 and 2)
    // This includes cache lookup, stale-while-revalidate handling, and cache validation
  }

  if (fetchParams.controller?.state === "aborted") {
    return networkError();
  }

  if (response === null) {
    if (httpRequest.cacheMode === "only-if-cached") {
      return networkError();
    }

    const forwardResponse = await httpNetworkFetch(
      httpFetchParams,
      includeCredentials,
      isNewConnectionFetch,
    );

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
      // TODO Invalidate cache
      storedResponse = null;
    }

    if (revalidatingFlag && forwardResponse?.status === 304) {
      // TODO Update stored response with validation
    }

    if (response === null) {
      response = forwardResponse;
      // TODO Store in cache
    }
  }

  if (response && httpRequest.urlList) {
    response.urlList = [...httpRequest.urlList];
  }

  if (hasRequestHeader(httpRequest, "Range")) {
    response.rangeRequestedFlag = true;
  }

  if (response) {
    response.requestIncludesCredentials = includeCredentials;
  }

  if (
    response?.status === 401 &&
    httpRequest.responseTainting !== "cors" &&
    includeCredentials &&
    request.window
  ) {
    // TODO Implement authentication challenge handling
    // This would show authentication dialog and retry request
  }

  if (response?.status === 407) {
    // TODO Implement proxy authentication challenge handling
  }

  if (
    response?.status === 421 &&
    !isNewConnectionFetch &&
    (request.body === null || (request.body && request.body.source))
  ) {
    // TODO Implement HTTP/2 connection coalescing retry logic
    // This would retry the request on a new connection
  }

  if (isAuthenticationFetch) {
    // TODO Create authentication entry
  }

  return response;
};
