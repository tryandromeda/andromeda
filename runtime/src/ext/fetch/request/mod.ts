// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

import { configureInterface } from "../../webidl/mod.ts";
import { fillHeaders } from "../headers/mod.ts";

type RequestInfo = Request | string;

/** @category Fetch */
interface RequestInit {
  /**
   * A BodyInit object or null to set request's body.
   */
  body?: BodyInit | null;
  /**
   * A string indicating how the request will interact with the browser's cache
   * to set request's cache.
   */
  cache?: RequestCache;
  /**
   * A string indicating whether credentials will be sent with the request
   * always, never, or only when sent to a same-origin URL. Sets request's
   * credentials.
   */
  credentials?: RequestCredentials;
  /**
   * A Headers object, an object literal, or an array of two-item arrays to set
   * request's headers.
   */
  headers?: HeadersInit;
  /**
   * A cryptographic hash of the resource to be fetched by request. Sets
   * request's integrity.
   */
  integrity?: string;
  /**
   * A boolean to set request's keepalive.
   */
  keepalive?: boolean;
  /**
   * A string to set request's method.
   */
  method?: string;
  /**
   * A string to indicate whether the request will use CORS, or will be
   * restricted to same-origin URLs. Sets request's mode.
   */
  mode?: RequestMode;
  /**
   * A string indicating whether request follows redirects, results in an error
   * upon encountering a redirect, or returns the redirect (in an opaque
   * fashion). Sets request's redirect.
   */
  redirect?: RequestRedirect;
  /**
   * A string whose value is a same-origin URL, "about:client", or the empty
   * string, to set request's referrer.
   */
  referrer?: string;
  /**
   * A referrer policy to set request's referrerPolicy.
   */
  referrerPolicy?: ReferrerPolicy;
  /**
   * An AbortSignal to set request's signal.
   */
  signal?: AbortSignal | null;
  /**
   * Can only be null. Used to disassociate request from any Window.
   */
  window?: any;
}

class Request {
  #request;
  #headersCache;
  #getHeaders;
  #signal;

  /** https://fetch.spec.whatwg.org/#dom-request */
  constructor(input: RequestInfo, init: RequestInit = { __proto__: null }) {
    // 1. Let request be null.
    let request = null;

    // 2. Let fallbackMode be null.
    let fallbackMode = null;

    // 3. Let baseURL be this’s relevant settings object’s API base URL.
    // const baseUrl = environmentSettingsObject.settingsObject.baseUrl

    // 4. Let signal be null.
    let signal = null;

    // 5.
    if (typeof input === "string") {
      const parsedURL = new URL(input);
      request = newInnerRequest(
        "GET",
        parsedURL.href,
        () => [],
        null,
        true,
      );
    } else {
      // 6.
      if (!Object.prototype.isPrototypeOf.call(RequestPrototype, input)) {
        throw new TypeError("Unreachable");
      }
      const originalReq = input.#request;
      // fold in of step 12 from below
      request = cloneInnerRequest(originalReq, true);
      request.redirectCount = 0; // reset to 0 - cloneInnerRequest copies the value
      signal = input.#signal;
    }

    // 7. Let origin be this’s relevant settings object’s origin.
    // 8. Let traversableForUserPrompts be "client".
    // 9. If request’s traversable for user prompts is an environment settings object and its origin is same origin with origin,
    //    then set traversableForUserPrompts to request’s traversable for user prompts.
    // 10. If init["window"] exists and is non-null, then throw a TypeError.
    if (init.window != null) {
      throw new TypeError(`'window' option '${window}' must be null`);
    }
    // 11. If init["window"] exists, then set traversableForUserPrompts to "no-traversable".

    // 12. is folded into the else statement of step 6 above.

    const initHasKey = Object.keys(init).length !== 0;

    // 13. If init is not empty, then:
    if (initHasKey) {
      // 1. If request’s mode is "navigate", then set it to "same-origin".
      if (request.mode === "navigate") {
        request.mode = "same-origin";
      }

      // 2. Unset request’s reload-navigation flag.
      request.reloadNavigation = false;

      // 3. Unset request’s history-navigation flag.
      request.historyNavigation = false;

      // 4. Set request’s origin to "client".
      request.origin = "client";

      // 5. Set request’s referrer to "client"
      request.referrer = "client";

      // 6. Set request’s referrer policy to the empty string.
      request.referrerPolicy = "";

      // 7. Set request’s URL to request’s current URL.
      request.url = request.urlList[request.urlList.length - 1];

      // 8. Set request’s URL list to « request’s URL ».
      request.urlList = [request.url];
    }

    // 14. If init["referrer"] exists, then:
    if (init.referrer !== undefined) {
      // 1. Let referrer be init["referrer"].
      const referrer = init.referrer;

      // 2. If referrer is the empty string, then set request’s referrer to "no-referrer".
      if (referrer === "") {
        request.referrer = "no-referrer";
      } else {
        // 1. Let parsedReferrer be the result of parsing referrer with
        // baseURL.
        // 2. If parsedReferrer is failure, then throw a TypeError.
        let parsedReferrer;
        try {
          parsedReferrer = new URL(referrer, baseUrl);
        } catch (err) {
          throw new TypeError(`Referrer "${referrer}" is not a valid URL.`, {
            cause: err,
          });
        }

        // 3. If one of the following is true
        // - parsedReferrer’s scheme is "about" and path is the string "client"
        // - parsedReferrer’s origin is not same origin with origin
        // then set request’s referrer to "client".
        if (
          (parsedReferrer.protocol === "about:" &&
            parsedReferrer.hostname === "client") ||
          (origin &&
            !sameOrigin(
              parsedReferrer,
              environmentSettingsObject.settingsObject.baseUrl,
            ))
        ) {
          request.referrer = "client";
        } else {
          // 4. Otherwise, set request’s referrer to parsedReferrer.
          request.referrer = parsedReferrer;
        }
      }
    }

    // 15. If init["referrerPolicy"] exists, then set request’s referrer policy
    // to it.
    if (init.referrerPolicy !== undefined) {
      request.referrerPolicy = init.referrerPolicy;
    }

    // 16. Let mode be init["mode"] if it exists, and fallbackMode otherwise.
    let mode;
    if (init.mode !== undefined) {
      mode = init.mode;
    } else {
      mode = fallbackMode;
    }

    // 17. If mode is "navigate", then throw a TypeError.
    if (mode === "navigate") {
      throw new TypeError(
        "Request constructor: invalid request mode navigate.",
      );
    }

    // 18. If mode is non-null, set request’s mode to mode.
    if (mode != null) {
      request.mode = mode;
    }

    // 19. If init["credentials"] exists, then set request’s credentials mode
    // to it.
    if (init.credentials !== undefined) {
      request.credentials = init.credentials;
    }

    // 18. If init["cache"] exists, then set request’s cache mode to it.
    if (init.cache !== undefined) {
      request.cache = init.cache;
    }

    // 21. If request’s cache mode is "only-if-cached" and request’s mode is
    // not "same-origin", then throw a TypeError.
    if (request.cache === "only-if-cached" && request.mode !== "same-origin") {
      throw new TypeError(
        "'only-if-cached' can be set only with 'same-origin' mode",
      );
    }

    // 22.
    if (init.redirect !== undefined) {
      request.redirectMode = init.redirect;
    }

    // 23. If init["integrity"] exists, then set request’s integrity metadata to it.
    if (init.integrity != null) {
      request.integrity = String(init.integrity);
    }

    // 24. If init["keepalive"] exists, then set request’s keepalive to it.
    if (init.keepalive !== undefined) {
      request.keepalive = Boolean(init.keepalive);
    }

    // 25. If init["method"] exists, then:
    if (init.method !== undefined) {
      // 1. Let method be init["method"].
      let method = init.method;

      const mayBeNormalized = normalizedMethodRecords[method];

      if (mayBeNormalized !== undefined) {
        // Note: Bypass validation DELETE, GET, HEAD, OPTIONS, POST, PUT, PATCH and these lowercase ones
        request.method = mayBeNormalized;
      } else {
        // 2. If method is not a method or method is a forbidden method, then
        // throw a TypeError.
        if (!isValidHTTPToken(method)) {
          throw new TypeError(`'${method}' is not a valid HTTP method.`);
        }

        const upperCase = method.toUpperCase();

        if (forbiddenMethodsSet.has(upperCase)) {
          throw new TypeError(`'${method}' HTTP method is unsupported.`);
        }

        // 3. Normalize method.
        // https://fetch.spec.whatwg.org/#concept-method-normalize
        // Note: must be in uppercase
        method = normalizedMethodRecordsBase[upperCase] ?? method;

        // 4. Set request’s method to method.
        request.method = method;
      }

      if (!patchMethodWarning && request.method === "patch") {
        process.emitWarning(
          "Using `patch` is highly likely to result in a `405 Method Not Allowed`. `PATCH` is much more likely to succeed.",
          {
            code: "UNDICI-FETCH-patch",
          },
        );

        patchMethodWarning = true;
      }
    }

    // 26. If init["signal"] exists, then set signal to it.
    if (init.signal !== undefined) {
      signal = init.signal;
    }

    // 27. Set this’s request to request.
    this.#state = request;

    // // 28. Set this’s signal to a new AbortSignal object with this’s relevant
    //    // Realm.
    //    // TODO: could this be simplified with AbortSignal.any
    //    // (https://dom.spec.whatwg.org/#dom-abortsignal-any)
    //    const ac = new AbortController()
    this.#signal = ac.signal;

    // 29 & 30.
    // TODO: AbortSignal
    // if (signal !== null) {
    //   this[_signalCache] = createDependentAbortSignal([signal], prefix);
    // }

    // 31.
    this[_headers] = headersFromHeaderList(request.headerList, "request");

    // 33.
    if (init.headers || ObjectKeys(init).length > 0) {
      const headerList = headerListFromHeaders(this[_headers]);
      const headers = init.headers ?? ArrayPrototypeSlice(
        headerList,
        0,
        headerList.length,
      );
      if (headerList.length !== 0) {
        ArrayPrototypeSplice(headerList, 0, headerList.length);
      }
      fillHeaders(this[_headers], headers);
    }

    // 34.
    let inputBody = null;
    if (ObjectPrototypeIsPrototypeOf(RequestPrototype, input)) {
      inputBody = input[_body];
    }

    // 35.
    if (
      (request.method === "GET" || request.method === "HEAD") &&
      ((init.body !== undefined && init.body !== null) ||
        inputBody !== null)
    ) {
      throw new TypeError("Request with GET/HEAD method cannot have body");
    }

    // 36.
    let initBody = null;

    // 37.
    if (init.body !== undefined && init.body !== null) {
      const res = extractBody(init.body);
      initBody = res.body;
      if (res.contentType !== null && !this[_headers].has("content-type")) {
        this[_headers].append("Content-Type", res.contentType);
      }
    }

    // 38.
    const inputOrInitBody = initBody ?? inputBody;

    // 40.
    let finalBody = inputOrInitBody;

    // 41.
    if (initBody === null && inputBody !== null) {
      if (input[_body] && input[_body].unusable()) {
        throw new TypeError("Input request's body is unusable");
      }
      finalBody = inputBody.createProxy();
    }

    // 42.
    request.body = finalBody;
  }
}

function newInnerRequest(
  method: string,
  url: string | (() => string),
  headerList: () => [string, string][],
  body: any,
  maybeBlob: any,
) {
  let blobUrlEntry = null;
  if (
    maybeBlob &&
    typeof url === "string" &&
    url.startsWith("blob:")
  ) {
    // TODO: the blobFromObjectUrl is file api
    // blobUrlEntry = blobFromObjectUrl(url);
    throw new Error("not support blob");
  }
  return {
    methodInner: method,
    get method() {
      return this.methodInner;
    },
    set method(value) {
      this.methodInner = value;
    },
    headerListInner: null,
    get headerList() {
      if (this.headerListInner === null) {
        try {
          this.headerListInner = headerList();
        } catch {
          throw new TypeError("Cannot read headers: request closed");
        }
      }
      return this.headerListInner;
    },
    set headerList(value) {
      this.headerListInner = value;
    },
    body,
    redirectMode: "follow",
    redirectCount: 0,
    urlList: [typeof url === "string" ? () => url : url],
    urlListProcessed: [],
    clientRid: null,
    blobUrlEntry,
    url() {
      if (this.urlListProcessed[0] === undefined) {
        try {
          this.urlListProcessed[0] = this.urlList[0]();
        } catch {
          throw new TypeError("cannot read url: request closed");
        }
      }
      return this.urlListProcessed[0];
    },
    currentUrl() {
      const currentIndex = this.urlList.length - 1;
      if (this.urlListProcessed[currentIndex] === undefined) {
        try {
          this.urlListProcessed[currentIndex] = this.urlList[currentIndex]();
        } catch {
          throw new TypeError("Cannot read url: request closed");
        }
      }
      return this.urlListProcessed[currentIndex];
    },
  };
}

configureInterface(Request);
const RequestPrototype = Request.prototype;
// mixinBody(RequestPrototype, _body, _mimeType);

/** https://fetch.spec.whatwg.org/#concept-request-clone */
function cloneInnerRequest(request: any, skipBody = false) {
  const headerList = request.headerList.push(
    (x) => [x[0], x[1]],
  );

  let body = null;
  if (request.body !== null && !skipBody) {
    body = request.body.clone();
  }

  return {
    method: request.method,
    headerList,
    body,
    redirectMode: request.redirectMode,
    redirectCount: request.redirectCount,
    urlList: [() => request.url()],
    urlListProcessed: [request.url()],
    clientRid: request.clientRid,
    blobUrlEntry: request.blobUrlEntry,
    url() {
      if (this.urlListProcessed[0] === undefined) {
        try {
          this.urlListProcessed[0] = this.urlList[0]();
        } catch {
          throw new TypeError("Cannot read url: request closed");
        }
      }
      return this.urlListProcessed[0];
    },
    currentUrl() {
      const currentIndex = this.urlList.length - 1;
      if (this.urlListProcessed[currentIndex] === undefined) {
        try {
          this.urlListProcessed[currentIndex] = this.urlList[currentIndex]();
        } catch {
          throw new TypeError("Cannot read url: request closed");
        }
      }
      return this.urlListProcessed[currentIndex];
    },
  };
}

function validateAndNormalizeMethod(m: string): string {
  // const upperCase = byteUpperCase(m);
  // TODO: replace and uppercase
  const upperCase = m;
  if (
    upperCase === "CONNECT" || upperCase === "TRACE" || upperCase === "TRACK"
  ) {
    throw new TypeError("Method is forbidden");
  }
  return upperCase;
}
