// deno-lint-ignore-file no-explicit-any prefer-const no-unused-vars
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// @ts-ignore deno lint stuff
type RequestInfo = Request;

// method => normalized method
const KNOWN_METHODS = {
  "DELETE": "DELETE",
  "delete": "DELETE",
  "GET": "GET",
  "get": "GET",
  "HEAD": "HEAD",
  "head": "HEAD",
  "OPTIONS": "OPTIONS",
  "options": "OPTIONS",
  "PATCH": "PATCH",
  "patch": "PATCH",
  "POST": "POST",
  "post": "POST",
  "PUT": "PUT",
  "put": "PUT",
};

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
  // @ts-ignore deno lint stuff
  method?: keyof typeof KNOWN_METHODS;
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
  // @ts-ignore deno lint stuff
  window?: any;
}

const requestSymbol = Symbol("[[request]]");
const signalSymbol = Symbol("[[signal]]");
const bodySymbol = Symbol("[[body]]");
// @ts-ignore deno lint stuff
class Request {
  [requestSymbol]: any;
  // TODO: comment in nova support module
  // #headers;
  [signalSymbol]: AbortSignal | null = null;
  [bodySymbol]: any = null;

  /** https://fetch.spec.whatwg.org/#request-class */
  constructor(input: RequestInfo, init: RequestInit = { __proto__: null } as any) {
    // 1. Let request be null.
    let request: any = null;

    // 2. Let fallbackMode be null.
    let fallbackMode: any = null;

    // 3. Let baseURL be this’s relevant settings object’s API base URL.
    // const baseUrl = environmentSettingsObject.settingsObject.baseUrl

    // 4. Let signal be null.
    let signal: any = null;

    // 5. If input is a string, then:
    if (typeof input === "string") {
      // 1. Let parsedURL be the result of parsing input with baseURL.
      // 2. If parsedURL is failure, then throw a TypeError.
      // 3. If parsedURL includes credentials, then throw a TypeError.
      // 4. Set request to a new request whose URL is parsedURL.
      // 5. Set fallbackMode to "cors"
      const parsedURL = new URL(input);
      request = newInnerRequest(
        "GET",
        parsedURL as any,
        () => [],
        null,
        true,
      );
    } else {
      // 6. Otherwise:
      //  1. Assert: input is a Request object.
      //  2. Set request to input’s request.
      //  3. Set signal to input’s signal.
      if (!Object.prototype.isPrototypeOf.call(RequestPrototype, input)) {
        throw new TypeError("Unreachable");
      }

      const originalReq = input[requestSymbol];
      // fold in of step 12 from below
      request = cloneInnerRequest(originalReq, true);
      request.redirectCount = 0; // reset to 0 - cloneInnerRequest copies the value
      signal = input[signalSymbol];
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
        // 1. Let parsedReferrer be the result of parsing referrer with baseURL.
        // 2. If parsedReferrer is failure, then throw a TypeError.
        let parsedReferrer;
        try {
          parsedReferrer = new URL(referrer);
        } catch (err) {
          throw new TypeError(`Referrer "${referrer}" is not a valid URL.`, {
            cause: err,
          });
        }

        // 3. If one of the following is true
        // - parsedReferrer’s scheme is "about" and path is the string "client"
        // - parsedReferrer’s origin is not same origin with origin
        // then set request’s referrer to "client".
        // TODO: sameOrigin
        //
        // 4. Otherwise, set request’s referrer to parsedReferrer.
        request.referrer = parsedReferrer;
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

    // 19. If init["credentials"] exists, then set request’s credentials mode to it.
    if (init.credentials !== undefined) {
      request.credentials = init.credentials;
    }

    // 20. If init["cache"] exists, then set request’s cache mode to it.
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

    // 22. If init["redirect"] exists, then set request’s redirect mode to it.
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
      // 2. If method is not a method or method is a forbidden method, then throw a TypeError.
      // 3. Normalize method.
      // 4. Set request’s method to method.
      const method = init.method;
      request.method = KNOWN_METHODS[method] ??
        validateAndNormalizeMethod(method);
    }

    // 26. If init["signal"] exists, then set signal to it.
    if (init.signal !== undefined) {
      signal = init.signal;
    }

    // 27. Set this’s request to request.
    this[requestSymbol] = request;

    // // 28. Set this’s signal to a new AbortSignal object with this’s relevant
    //    // Realm.
    //    // TODO: could this be simplified with AbortSignal.any
    //    // (https://dom.spec.whatwg.org/#dom-abortsignal-any)
    //    const ac = new AbortController()
    // this.#signal = ac.signal;

    // 29 & 30.
    // TODO: AbortSignal
    // if (signal !== null) {
    //   this[_signalCache] = createDependentAbortSignal([signal], prefix);
    // }

    // 31. Set this’s headers to a new Headers object with this’s relevant realm,
    //     whose header list is request’s header list and guard is "request".
    // TODO: removed nova support module
    // setHeadersList(this.#headers, request.headersList);
    // setHeadersGuard(this.#headers, "request");

    // 32. If this’s request’s mode is "no-cors", then:
    const corsSafeListedMethods = ["GET", "HEAD", "POST"] as const;
    const corsSafeListedMethodsSet = new Set(corsSafeListedMethods);
    if (request.mode === "no-cors") {
      // 1. If this’s request’s method is not a CORS-safelisted method,
      // then throw a TypeError.
      if (!corsSafeListedMethodsSet.has(request.method)) {
        throw new TypeError(
          `'${request.method} is unsupported in no-cors mode.`,
        );
      }
      // 2. Set this’s headers’s guard to "request-no-cors".
      // TODO: removed nova support module
      // setHeadersGuard(this.#headers, "request-no-cors");
    }

    // 33.
    // TODO: removed nova support module
    // if (init.headers || Object.keys(init).length > 0) {
    //   const headerList = getHeadersList(this.#headers);
    //   const headers = init.headers ?? headerList.slice(
    //     0,
    //     headerList.length,
    //   );
    //   if (headerList.length !== 0) {
    //     headerList.splice(0, headerList.length);
    //   }
    //   fillHeaders(this.#headers, headers);
    // }

    // 34. Let inputBody be input’s request’s body if input is a Request object; otherwise null.
    let inputBody: any = null;
    if (Object.prototype.isPrototypeOf.call(RequestPrototype, input)) {
      inputBody = input[bodySymbol];
    }

    // 35. If either init["body"] exists and is non-null or inputBody is non-null, and request’s method is `GET` or `HEAD`, then throw a TypeError.
    if (
      (request.method === "GET" || request.method === "HEAD") &&
      ((init.body !== undefined && init.body !== null) ||
        inputBody !== null)
    ) {
      throw new TypeError("Request with GET/HEAD method cannot have body");
    }

    // 36. Let initBody be null.
    let initBody = null;

    // 37. If init["body"] exists and is non-null, then:
    // Let bodyWithType be the result of extracting init["body"], with keepalive set to request’s keepalive.
    // Set initBody to bodyWithType’s body.
    // Let type be bodyWithType’s type.
    // If type is non-null and this’s headers’s header list does not contain `Content-Type`, then append (`Content-Type`, type) to this’s headers.
    // TODO

    // 38. Let inputOrInitBody be initBody if it is non-null; otherwise inputBody.
    const inputOrInitBody = initBody ?? inputBody;

    // 39. If inputOrInitBody is non-null and inputOrInitBody’s source is null, then:
    // If initBody is non-null and init["duplex"] does not exist, then throw a TypeError.
    // If this’s request’s mode is neither "same-origin" nor "cors", then throw a TypeError.
    // Set this’s request’s use-CORS-preflight flag.

    let finalBody = inputOrInitBody;

    // 41. If initBody is null and inputBody is non-null, then:
    if (initBody === null && inputBody !== null) {
      // 1. If input is unusable, then throw a TypeError.
      if (input[bodySymbol] && input[bodySymbol].unusable()) {
        throw new TypeError("Input request's body is unusable");
      }
      // 2. Set finalBody to the result of creating a proxy for inputBody.
      finalBody = (inputBody as any).createProxy();
    }

    // 42. Set this’s request’s body to finalBody.
    request.body = finalBody;

    const url = request.url;
    console.log("url", url);
    const method = request.method;
    console.log("method", method);
    const credentials = request.credentials;
    console.log("credentials", credentials);

    this[requestSymbol] = request;
  }

  // Returns request’s HTTP method, which is "GET" by default.
  get method() {
    return this[requestSymbol].method;
  }

  // Returns the URL of request as a string.
  get url() {
    // The url getter steps are to return this’s request’s URL, serialized.
    return this[requestSymbol].url;
  }

  // Returns a Headers object consisting of the headers associated with request.
  // Note that headers added in the network layer by the user agent will not
  // be accounted for in this object, e.g., the "Host" header.
  // get headers() {
  //   return this.#headers;
  // }

  // Returns the kind of resource requested by request, e.g., "document"
  // or "script".
  get destination() {
    // The destination getter are to return this’s request’s destination.
    return this[requestSymbol].destination;
  }

  // Returns the referrer of request. Its value can be a same-origin URL if
  // explicitly set in init, the empty string to indicate no referrer, and
  // "about:client" when defaulting to the global’s default. This is used
  // during fetching to determine the value of the `Referer` header of the
  // request being made.
  get referrer() {
    if (this[requestSymbol].referrer === "no-referrer") {
      return "";
    }

    // 2. If this’s request’s referrer is "client", then return
    // "about:client".
    if (this[requestSymbol].referrer === "client") {
      return "about:client";
    }

    // Return this’s request’s referrer, serialized.
    return this[requestSymbol].referrer.toString();
  }

  // Returns the referrer policy associated with request.
  // This is used during fetching to compute the value of the request’s
  // referrer.
  get referrerPolicy() {
    return this[requestSymbol].referrerPolicy;
  }

  // Returns the mode associated with request, which is a string indicating
  // whether the request will use CORS, or will be restricted to same-origin
  // URLs.
  get mode() {
    return this[requestSymbol].mode;
  }

  // Returns the credentials mode associated with request,
  // which is a string indicating whether credentials will be sent with the
  // request always, never, or only when sent to a same-origin URL.
  get credentials() {
    return this[requestSymbol].credentials;
  }

  // Returns the cache mode associated with request,
  // which is a string indicating how the request will
  // interact with the browser’s cache when fetching.
  get cache() {
    return this[requestSymbol].cache;
  }

  // Returns the redirect mode associated with request,
  // which is a string indicating how redirects for the
  // request will be handled during fetching. A request
  // will follow redirects by default.
  get redirect() {
    return this[requestSymbol].redirect;
  }

  get integrity() {
    return this[requestSymbol].integrity;
  }

  // Returns a boolean indicating whether or not request can outlive the
  // global in which it was created.
  get keepalive() {
    return this[requestSymbol].keepalive;
  }

  get isReloadNavigation() {
    return this[requestSymbol].reloadNavigation;
  }

  get isHistoryNavigation() {
    return this[requestSymbol].historyNavigation;
  }

  get signal() {
    return this[signalSymbol];
  }

  get body() {
    return this[requestSymbol].body ? this[requestSymbol].body.stream : null;
  }

  get bodyUsed() {
    return !!this[requestSymbol].body;
  }

  get duplex() {
    return "half";
  }
}

function newInnerRequest(
  method: string,
  url: () => string,
  headerList: () => [string, string][],
  body: any,
  maybeBlob: any,
): any {
  let blobUrlEntry = null;
  if (
    maybeBlob &&
    typeof url === "string"
    // url.startsWith("blob:")
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
    url,
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
function cloneInnerRequest(request: any, skipBody = false): any {
  const headerList = request.headerList.push(
    (x: any) => [x[0], x[1]],
  );

  let body = null;
  if (request.body !== null && !skipBody) {
    body = request.body.clone();
  }

  return {
    mode: request.mode,
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

// TODO: removed nova support module
function configureInterface(interface_: any) {
  configureProperties(interface_);
  configureProperties(interface_.prototype);
  Object.defineProperty(interface_.prototype, Symbol.toStringTag, {
    // @ts-ignore:
    __proto__: null,
    value: interface_.name,
    enumerable: false,
    configurable: true,
    writable: false,
  });
}

// TODO: removed nova support module
function configureProperties(obj: any) {
  const descriptors = Object.getOwnPropertyDescriptors(obj);
  for (const key in descriptors) {
    if (!Object.hasOwn(descriptors, key)) {
      continue;
    }
    if (key === "constructor") continue;
    if (key === "prototype") continue;
    const descriptor = descriptors[key];
    if (
      Reflect.has(descriptor, "value") &&
      typeof descriptor.value === "function"
    ) {
      Object.defineProperty(obj, key, {
        // @ts-ignore:
        __proto__: null,
        enumerable: true,
        writable: true,
        configurable: true,
      });
    } else if (Reflect.has(descriptor, "get")) {
      Object.defineProperty(obj, key, {
        // @ts-ignore:
        __proto__: null,
        enumerable: true,
        configurable: true,
      });
    }
  }
}
