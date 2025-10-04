// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// deno-lint-ignore-file no-explicit-any

const extractBody = (globalThis as any).extractBody;
const BodyMixin = (globalThis as any).BodyMixin;
const BODY_SYMBOL = (globalThis as any).BODY_SYMBOL;
const CONTENT_TYPE_SYMBOL = (globalThis as any).CONTENT_TYPE_SYMBOL;

const Headers = (globalThis as any).Headers;
const { setHeadersList, setHeadersGuard, getHeadersList } = Headers;

type RequestInfo = Request | string | URL;

type BodyInit =
  | ReadableStream<Uint8Array>
  | Blob
  | ArrayBuffer
  | ArrayBufferView
  | FormData
  | URLSearchParams
  | string;

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

/**
 * Request initialization options.
 * @see https://fetch.spec.whatwg.org/#requestinit
 */
interface RequestInit {
  body?: BodyInit | null;
  cache?: RequestCache;
  credentials?: RequestCredentials;
  headers?: HeadersInit;
  integrity?: string;
  keepalive?: boolean;
  method?: string;
  mode?: RequestMode;
  redirect?: RequestRedirect;
  referrer?: string;
  referrerPolicy?: ReferrerPolicy;
  signal?: AbortSignal | null;
  window?: any;
}

// Use symbols instead of private fields for internal state
const REQUEST_INTERNAL = Symbol("request.internal");
const REQUEST_HEADERS = Symbol("request.headers");
const REQUEST_SIGNAL = Symbol("request.signal");

/**
 * Request class represents an HTTP request.
 * @see https://fetch.spec.whatwg.org/#request-class
 */
class Request extends BodyMixin {
  /**
   * The Request(input, init) constructor steps are:
   * @see https://fetch.spec.whatwg.org/#dom-request
   */
  constructor(
    input: RequestInfo,
    init: RequestInit = {},
  ) {
    // Initialize body as null initially
    super(null, null);

    // Initialize internal fields using symbols (not declared as class fields)
    (this as any)[REQUEST_INTERNAL] = null;
    (this as any)[REQUEST_HEADERS] = new Headers();
    (this as any)[REQUEST_SIGNAL] = null;

    // 1. Let request be null.
    let request: any = null;

    // 2. Let fallbackMode be null.
    let fallbackMode: RequestMode | null = null;

    // 3. Let baseURL be this's relevant settings object's API base URL.
    const baseURL = globalThis.location?.href || "http://localhost/";

    // 4. Let signal be null.
    let signal: AbortSignal | null = null;

    // 5. If input is a string, then:
    if (typeof input === "string") {
      // 1. Let parsedURL be the result of parsing input with baseURL.
      const parsedURL = new URL(input, baseURL);
      
      // 2. If parsedURL is failure, then throw a TypeError.
      if (!parsedURL) {
        throw new TypeError("Invalid URL");
      }

      // 3. If parsedURL includes credentials, then throw a TypeError.
      if (parsedURL.username || parsedURL.password) {
        throw new TypeError("Request cannot be constructed from a URL that includes credentials");
      }

      // 4. Set request to a new request whose URL is parsedURL.
      request = newInnerRequest("GET", parsedURL.toString());
      
      // 5. Set fallbackMode to "cors".
      fallbackMode = "cors";
    } else if (input instanceof URL) {
      // Handle URL objects
      const parsedURL = input;
      
      if (parsedURL.username || parsedURL.password) {
        throw new TypeError("Request cannot be constructed from a URL that includes credentials");
      }

      request = newInnerRequest("GET", parsedURL.toString());
      fallbackMode = "cors";
    } else {
      // 6. Otherwise:
      // 1. Assert: input is a Request object.
      if (!(input instanceof Request)) {
        throw new TypeError("Invalid input");
      }

      // 2. Set request to input's request.
      request = (input as any)[REQUEST_INTERNAL];
      
      // 3. Set signal to input's signal.
      signal = (input as any)[REQUEST_SIGNAL];
    }

    // 7-9. Origin, window, and service workers handling (simplified)
    
    // 10. Let window be "client".
    const _windowValue = "client";

    // 11. If init["window"] exists, then:
    if (init.window !== undefined) {
      // 1. If init["window"] is non-null, then throw a TypeError.
      if (init.window !== null) {
        throw new TypeError("Window can only be set to null");
      }
      // 2. Set window to "no-window".
      // windowValue = "no-window"; (not used further in this implementation)
    }

    // 12. Set request to a new request with the following properties:
    request = cloneInnerRequest(request);

    // 13. If init is not empty, then:
    if (Object.keys(init).length > 0) {
      // 1. If request's mode is "navigate", then set it to "same-origin".
      if (request.mode === "navigate") {
        request.mode = "same-origin";
      }

      // 2. Unset request's reload-navigation flag.
      request.reloadNavigation = false;

      // 3. Unset request's history-navigation flag.
      request.historyNavigation = false;

      // 4. Set request's origin to "client".
      request.origin = "client";

      // 5. Set request's referrer to "client"
      request.referrer = "client";

      // 6. Set request's referrer policy to the empty string.
      request.referrerPolicy = "";

      // 7. Set request's URL to request's current URL.
      request.url = request.currentUrl();

      // 8. Set request's URL list to « request's URL ».
      request.urlList = [request.url];
    }

    // 14. If init["referrer"] exists, then:
    if (init.referrer !== undefined) {
      const referrer = init.referrer;
      
      // 1. Let referrerURL be empty string.
      if (referrer === "") {
        // 2. Set request's referrer to "no-referrer".
        request.referrer = "no-referrer";
      } else {
        // 3. Let parsedReferrer be the result of parsing referrer with baseURL.
        try {
          const parsedReferrer = new URL(referrer, baseURL);
          
          // 4. If parsedReferrer is failure, then throw a TypeError.
          // 5. If parsedReferrer's scheme is "about" and path is "client", or parsedReferrer's origin is not same origin with origin, then set request's referrer to "client".
          if (parsedReferrer.protocol === "about:" && parsedReferrer.pathname === "client") {
            request.referrer = "client";
          } else {
            // 6. Otherwise, set request's referrer to parsedReferrer.
            request.referrer = parsedReferrer.toString();
          }
        } catch {
          throw new TypeError("Invalid referrer URL");
        }
      }
    }

    // 15. If init["referrerPolicy"] exists, then set request's referrer policy to it.
    if (init.referrerPolicy !== undefined) {
      request.referrerPolicy = init.referrerPolicy;
    }

    // 16. Let mode be init["mode"] if it exists, and fallbackMode otherwise.
    const mode = init.mode !== undefined ? init.mode : fallbackMode;

    // 17. If mode is "navigate", then throw a TypeError.
    if (mode === "navigate") {
      throw new TypeError("Mode cannot be navigate");
    }

    // 18. If mode is non-null, set request's mode to mode.
    if (mode !== null) {
      request.mode = mode;
    }

    // 19. If init["credentials"] exists, then set request's credentials mode to it.
    if (init.credentials !== undefined) {
      request.credentials = init.credentials;
    }

    // 20. If init["cache"] exists, then set request's cache mode to it.
    if (init.cache !== undefined) {
      request.cache = init.cache;
    }

    // 21. If request's cache mode is "only-if-cached" and request's mode is not "same-origin", then throw a TypeError.
    if (request.cache === "only-if-cached" && request.mode !== "same-origin") {
      throw new TypeError("only-if-cached cache mode requires same-origin mode");
    }

    // 22. If init["redirect"] exists, then set request's redirect mode to it.
    if (init.redirect !== undefined) {
      request.redirect = init.redirect;
    }

    // 23. If init["integrity"] exists, then set request's integrity metadata to it.
    if (init.integrity !== undefined) {
      request.integrity = init.integrity;
    }

    // 24. If init["keepalive"] exists, then set request's keepalive to it.
    if (init.keepalive !== undefined) {
      request.keepalive = init.keepalive;
    }

    // 25. If init["method"] exists, then:
    if (init.method !== undefined) {
      // 1. Let method be init["method"].
      let method = init.method;

      // 2. If method is not a method or method is a forbidden method, throw a TypeError.
      method = validateAndNormalizeMethod(method);

      // 3. Set request's method to method.
      request.method = method;
    }

    // 26. If init["signal"] exists, then set signal to it.
    if (init.signal !== undefined) {
      signal = init.signal;
    }

    // 27. Set this's request to request.
    (this as any)[REQUEST_INTERNAL] = request;

    // 28. Set this's signal to a new AbortSignal object with this's relevant Realm.
    (this as any)[REQUEST_SIGNAL] = signal;

    // 29. Set this's headers to a new Headers object with this's relevant Realm, whose header list is request's header list and guard is "request".
    (this as any)[REQUEST_HEADERS] = new Headers();
    setHeadersList((this as any)[REQUEST_HEADERS], request.headerList || []);
    setHeadersGuard((this as any)[REQUEST_HEADERS], "request");

    // 30. If this's request's mode is "no-cors", then:
    if (request.mode === "no-cors") {
      // 1. If this's request's method is not a CORS-safelisted method, throw a TypeError.
      if (!["GET", "HEAD", "POST"].includes(request.method)) {
        throw new TypeError("Method not allowed in no-cors mode");
      }

      // 2. Set this's headers's guard to "request-no-cors".
      setHeadersGuard((this as any)[REQUEST_HEADERS], "request-no-cors");
    }

    // 31. If init["headers"] exists, then fill this's headers with init["headers"].
    if (init.headers !== undefined) {
      (globalThis as any).fillHeaders((this as any)[REQUEST_HEADERS], init.headers);
    }

    // 32. Let inputBody be input's request's body if input is a Request object; otherwise null.
    let inputBody: any = null;
    if (input instanceof Request) {
      inputBody = (input as any)[BODY_SYMBOL];
    }

    // 33. If either init["body"] exists and is non-null or inputBody is non-null, and request's method is `GET` or `HEAD`, throw a TypeError.
    if (((init.body !== undefined && init.body !== null) || inputBody !== null) && 
        (request.method === "GET" || request.method === "HEAD")) {
      throw new TypeError("Request with GET/HEAD method cannot have body");
    }

    // 34. Let initBody be null.
    let initBody: any = null;

    // 35. If init["body"] exists and is non-null, then:
    if (init.body !== undefined && init.body !== null) {
      // 1. Let bodyWithType be the result of extracting init["body"].
      const extracted = extractBody(init.body);
      initBody = extracted.body;
      const contentType = extracted.contentType;

      // 2. Set initBody to bodyWithType's body.
      // 3. Let type be bodyWithType's type.
      // 4. If type is non-null and this's headers's header list does not contain `Content-Type`, then append (`Content-Type`, type) to this's headers.
      if (contentType && !(this as any)[REQUEST_HEADERS].has("Content-Type")) {
        (this as any)[REQUEST_HEADERS].set("Content-Type", contentType);
      }
    }

    // 36. Let body be initBody.
    let finalBody = initBody;

    // 37. If initBody is null and inputBody is non-null, then:
    if (initBody === null && inputBody !== null) {
      // 1. If input is unusable, then throw a TypeError.
      if (input instanceof Request && input.bodyUsed) {
        throw new TypeError("Cannot construct a Request with a Request that has already been used");
      }

      // 2. Set body to the result of cloning inputBody.
      finalBody = inputBody.clone();
    }

    // 38. Set this's request's body to body.
    (this as any)[BODY_SYMBOL] = finalBody;
    
    // Also update content type
    const contentTypeHeader = (this as any)[REQUEST_HEADERS].get("Content-Type");
    (this as any)[CONTENT_TYPE_SYMBOL] = contentTypeHeader;
  }

  /**
   * Returns request's HTTP method.
   * @see https://fetch.spec.whatwg.org/#dom-request-method
   */
  get method(): string {
    return (this as any)[REQUEST_INTERNAL].method;
  }

  /**
   * Returns the URL of request as a string.
   * @see https://fetch.spec.whatwg.org/#dom-request-url
   */
  get url(): string {
    return (this as any)[REQUEST_INTERNAL].url;
  }

  /**
   * Returns a Headers object consisting of the headers associated with request.
   * @see https://fetch.spec.whatwg.org/#dom-request-headers
   */
  get headers(): Headers {
    return (this as any)[REQUEST_HEADERS];
  }

  /**
   * Returns the mode associated with request.
   * @see https://fetch.spec.whatwg.org/#dom-request-mode
   */
  get mode(): RequestMode {
    return (this as any)[REQUEST_INTERNAL].mode;
  }

  /**
   * Returns the credentials mode associated with request.
   * @see https://fetch.spec.whatwg.org/#dom-request-credentials
   */
  get credentials(): RequestCredentials {
    return (this as any)[REQUEST_INTERNAL].credentials;
  }

  /**
   * Returns the cache mode associated with request.
   * @see https://fetch.spec.whatwg.org/#dom-request-cache
   */
  get cache(): RequestCache {
    return (this as any)[REQUEST_INTERNAL].cache;
  }

  /**
   * Returns the redirect mode associated with request.
   * @see https://fetch.spec.whatwg.org/#dom-request-redirect
   */
  get redirect(): RequestRedirect {
    return (this as any)[REQUEST_INTERNAL].redirect;
  }

  /**
   * Returns the referrer of request.
   * @see https://fetch.spec.whatwg.org/#dom-request-referrer
   */
  get referrer(): string {
    const ref = (this as any)[REQUEST_INTERNAL].referrer;
    if (ref === "no-referrer") {
      return "";
    }
    if (ref === "client") {
      return "about:client";
    }
    return ref;
  }

  /**
   * Returns the referrer policy associated with request.
   * @see https://fetch.spec.whatwg.org/#dom-request-referrerpolicy
   */
  get referrerPolicy(): ReferrerPolicy {
    return (this as any)[REQUEST_INTERNAL].referrerPolicy || "";
  }

  /**
   * Returns the subresource integrity metadata associated with request.
   * @see https://fetch.spec.whatwg.org/#dom-request-integrity
   */
  get integrity(): string {
    return (this as any)[REQUEST_INTERNAL].integrity || "";
  }

  /**
   * Returns a boolean indicating whether or not request can outlive the global in which it was created.
   * @see https://fetch.spec.whatwg.org/#dom-request-keepalive
   */
  get keepalive(): boolean {
    return (this as any)[REQUEST_INTERNAL].keepalive || false;
  }

  /**
   * Returns the signal associated with request.
   * @see https://fetch.spec.whatwg.org/#dom-request-signal
   */
  get signal(): AbortSignal {
    // If no signal, create a never-aborted signal
    if (!(this as any)[REQUEST_SIGNAL]) {
      (this as any)[REQUEST_SIGNAL] = new AbortSignal();
    }
    return (this as any)[REQUEST_SIGNAL];
  }

  /**
   * Returns the body as a ReadableStream.
   * @see https://fetch.spec.whatwg.org/#dom-body-body
   */
  get body(): ReadableStream<Uint8Array> | null {
    if (!(this as any)[BODY_SYMBOL]) {
      return null;
    }
    return (this as any)[BODY_SYMBOL].stream;
  }

  /**
   * Clones the request.
   * @see https://fetch.spec.whatwg.org/#dom-request-clone
   */
  clone(): Request {
    // 1. If this is unusable, then throw a TypeError.
    if (this.bodyUsed) {
      throw new TypeError("Cannot clone a request that has already been used");
    }

    // 2. Let clonedRequest be the result of cloning this's request.
    const clonedInternalRequest = cloneInnerRequest((this as any)[REQUEST_INTERNAL]);

    // 3. Let clonedRequestObject be the result of creating a Request object, given clonedRequest, this's headers's guard, and this's relevant Realm.
    const cloned = Object.create(Request.prototype);
    (cloned as any)[REQUEST_INTERNAL] = clonedInternalRequest;
    (cloned as any)[REQUEST_SIGNAL] = (this as any)[REQUEST_SIGNAL];
    
    // Clone headers
    (cloned as any)[REQUEST_HEADERS] = new Headers();
    const headerList = getHeadersList((this as any)[REQUEST_HEADERS]);
    setHeadersList((cloned as any)[REQUEST_HEADERS], headerList.map((h: any) => [h[0], h[1]]));
    const currentGuard = ((this as any)[REQUEST_HEADERS] as any).guard || "request";
    setHeadersGuard((cloned as any)[REQUEST_HEADERS], currentGuard);

    // Clone body if present
    let clonedBody: any = null;
    if ((this as any)[BODY_SYMBOL]) {
      clonedBody = (this as any)[BODY_SYMBOL].clone();
    }

    // Set up the body
    BodyMixin.call(cloned, clonedBody, (this as any)[CONTENT_TYPE_SYMBOL]);
    (cloned as any)[BODY_SYMBOL] = clonedBody;
    (cloned as any)[CONTENT_TYPE_SYMBOL] = (this as any)[CONTENT_TYPE_SYMBOL];

    // 4. Return clonedRequestObject.
    return cloned;
  }
}

/**
 * Creates a new inner request object.
 */
function newInnerRequest(method: string, url: string): any {
  return {
    method: method,
    url: url,
    headerList: [],
    body: null,
    mode: "cors",
    credentials: "same-origin",
    cache: "default",
    redirect: "follow",
    referrer: "client",
    referrerPolicy: "",
    integrity: "",
    keepalive: false,
    reloadNavigation: false,
    historyNavigation: false,
    urlList: [url],
    currentUrl() {
      return this.url;
    },
  };
}

/**
 * Clones an inner request object.
 * @see https://fetch.spec.whatwg.org/#concept-request-clone
 */
function cloneInnerRequest(request: any, skipBody = false): any {
  const headerList = request.headerList.map((h: any) => [h[0], h[1]]);

  let body = null;
  if (request.body !== null && !skipBody) {
    body = request.body.clone();
  }

  return {
    method: request.method,
    url: request.url,
    headerList,
    body,
    mode: request.mode,
    credentials: request.credentials,
    cache: request.cache,
    redirect: request.redirect,
    referrer: request.referrer,
    referrerPolicy: request.referrerPolicy,
    integrity: request.integrity,
    keepalive: request.keepalive,
    reloadNavigation: request.reloadNavigation || false,
    historyNavigation: request.historyNavigation || false,
    urlList: [...request.urlList],
    currentUrl() {
      return this.url;
    },
  };
}

/**
 * Validates and normalizes an HTTP method.
 */
function validateAndNormalizeMethod(method: string): string {
  // Normalize to uppercase
  const upperMethod = method.toUpperCase();

  // Check if it's a forbidden method
  if (["CONNECT", "TRACE", "TRACK"].includes(upperMethod)) {
    throw new TypeError(`Method ${method} is forbidden`);
  }

  // Return normalized method for known methods
  if (upperMethod in KNOWN_METHODS) {
    return (KNOWN_METHODS as any)[upperMethod];
  }

  // Return as-is for custom methods
  return method;
}

// Export Request to globalThis
(globalThis as any).Request = Request;
