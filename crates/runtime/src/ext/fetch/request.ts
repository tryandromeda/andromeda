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

type RequestPriority = "high" | "low" | "auto";

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
  priority?: RequestPriority;
  duplex?: "half";
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
   * The Request(input, init) constructor steps are
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

    let request: any = null;

    let fallbackMode: RequestMode | null = null;

    const baseURL = globalThis.location?.href || "http://localhost/";

    let signal: AbortSignal | null = null;

    if (typeof input === "string") {
      const parsedURL = new URL(input, baseURL);

      if (!parsedURL) {
        throw new TypeError("Invalid URL");
      }

      if (parsedURL.username || parsedURL.password) {
        throw new TypeError(
          "Request cannot be constructed from a URL that includes credentials",
        );
      }

      request = newInnerRequest("GET", parsedURL.toString());

      fallbackMode = "cors";
    } else if (input instanceof URL) {
      // Handle URL objects
      const parsedURL = input;

      if (parsedURL.username || parsedURL.password) {
        throw new TypeError(
          "Request cannot be constructed from a URL that includes credentials",
        );
      }

      request = newInnerRequest("GET", parsedURL.toString());
      fallbackMode = "cors";
    } else {
      if (!(input instanceof Request)) {
        throw new TypeError("Invalid input");
      }

      request = (input as any)[REQUEST_INTERNAL];

      signal = (input as any)[REQUEST_SIGNAL];
    }

    // 7-9. Origin, window, and service workers handling (simplified)

    const _windowValue = "client";

    if (init.window !== undefined) {
      if (init.window !== null) {
        throw new TypeError("Window can only be set to null");
      }
    }

    request = cloneInnerRequest(request);

    if (Object.keys(init).length > 0) {
      if (request.mode === "navigate") {
        request.mode = "same-origin";
      }

      request.reloadNavigation = false;

      request.historyNavigation = false;

      request.origin = "client";

      request.referrer = "client";

      request.referrerPolicy = "";

      request.url = request.currentUrl();

      request.urlList = [request.url];
    }

    if (init.referrer !== undefined) {
      const referrer = init.referrer;

      if (referrer === "") {
        request.referrer = "no-referrer";
      } else {
        try {
          const parsedReferrer = new URL(referrer, baseURL);

          if (
            parsedReferrer.protocol === "about:" &&
            parsedReferrer.pathname === "client"
          ) {
            request.referrer = "client";
          } else {
            request.referrer = parsedReferrer.toString();
          }
        } catch {
          throw new TypeError("Invalid referrer URL");
        }
      }
    }

    if (init.referrerPolicy !== undefined) {
      request.referrerPolicy = init.referrerPolicy;
    }

    const mode = init.mode !== undefined ? init.mode : fallbackMode;

    if (mode === "navigate") {
      throw new TypeError("Mode cannot be navigate");
    }

    if (mode !== null) {
      request.mode = mode;
    }

    if (init.credentials !== undefined) {
      request.credentials = init.credentials;
    }

    if (init.cache !== undefined) {
      request.cache = init.cache;
    }

    if (request.cache === "only-if-cached" && request.mode !== "same-origin") {
      throw new TypeError(
        "only-if-cached cache mode requires same-origin mode",
      );
    }

    if (init.redirect !== undefined) {
      request.redirect = init.redirect;
    }

    if (init.integrity !== undefined) {
      request.integrity = init.integrity;
    }

    if (init.keepalive !== undefined) {
      request.keepalive = init.keepalive;
    }

    if (init.method !== undefined) {
      let method = init.method;

      method = validateAndNormalizeMethod(method);

      request.method = method;
    }

    if (init.signal !== undefined) {
      signal = init.signal;
    }

    (this as any)[REQUEST_INTERNAL] = request;

    (this as any)[REQUEST_SIGNAL] = signal;

    (this as any)[REQUEST_HEADERS] = new Headers();
    setHeadersList((this as any)[REQUEST_HEADERS], request.headerList || []);
    setHeadersGuard((this as any)[REQUEST_HEADERS], "request");

    if (request.mode === "no-cors") {
      if (!["GET", "HEAD", "POST"].includes(request.method)) {
        throw new TypeError("Method not allowed in no-cors mode");
      }

      setHeadersGuard((this as any)[REQUEST_HEADERS], "request-no-cors");
    }

    if (init.headers !== undefined) {
      (globalThis as any).fillHeaders(
        (this as any)[REQUEST_HEADERS],
        init.headers,
      );
    }

    let inputBody: any = null;
    if (input instanceof Request) {
      inputBody = (input as any)[BODY_SYMBOL];
    }

    if (
      ((init.body !== undefined && init.body !== null) || inputBody !== null) &&
      (request.method === "GET" || request.method === "HEAD")
    ) {
      throw new TypeError("Request with GET/HEAD method cannot have body");
    }

    let initBody: any = null;

    if (init.body !== undefined && init.body !== null) {
      const extracted = extractBody(init.body);
      initBody = extracted.body;
      const contentType = extracted.contentType;

      if (contentType && !(this as any)[REQUEST_HEADERS].has("Content-Type")) {
        (this as any)[REQUEST_HEADERS].set("Content-Type", contentType);
      }
    }

    let finalBody = initBody;

    if (initBody === null && inputBody !== null) {
      if (input instanceof Request && input.bodyUsed) {
        throw new TypeError(
          "Cannot construct a Request with a Request that has already been used",
        );
      }

      finalBody = inputBody.clone();
    }

    (this as any)[BODY_SYMBOL] = finalBody;

    // Also update content type
    const contentTypeHeader = (this as any)[REQUEST_HEADERS].get(
      "Content-Type",
    );
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
   * Returns a boolean indicating whether or not request is for a reload navigation.
   * @see https://fetch.spec.whatwg.org/#dom-request-isreloadnavigation
   */
  get isReloadNavigation(): boolean {
    return (this as any)[REQUEST_INTERNAL].reloadNavigation || false;
  }

  /**
   * Returns a boolean indicating whether or not request is for a history navigation (a.k.a. back-forward navigation).
   * @see https://fetch.spec.whatwg.org/#dom-request-ishistorynavigation
   */
  get isHistoryNavigation(): boolean {
    return (this as any)[REQUEST_INTERNAL].historyNavigation || false;
  }

  /**
   * Returns the signal associated with request.
   * @see https://fetch.spec.whatwg.org/#dom-request-signal
   */
  get signal(): AbortSignal {
    // If no signal, create a never-aborted signal
    if (!(this as any)[REQUEST_SIGNAL]) {
      (this as any)[REQUEST_SIGNAL] = new AbortController().signal;
    }
    return (this as any)[REQUEST_SIGNAL];
  }

  /**
   * Returns "`half`", meaning the fetch will be half-duplex.
   * @see https://fetch.spec.whatwg.org/#dom-request-duplex
   */
  get duplex(): "half" {
    return "half";
  }

  /**
   * Returns request's priority.
   * @see https://fetch.spec.whatwg.org/#dom-request-priority
   */
  get priority(): RequestPriority {
    return (this as any)[REQUEST_INTERNAL].priority || "auto";
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
    if (this.bodyUsed) {
      throw new TypeError("Cannot clone a request that has already been used");
    }

    const clonedInternalRequest = cloneInnerRequest(
      (this as any)[REQUEST_INTERNAL],
    );

    const cloned = Object.create(Request.prototype);
    (cloned as any)[REQUEST_INTERNAL] = clonedInternalRequest;
    (cloned as any)[REQUEST_SIGNAL] = (this as any)[REQUEST_SIGNAL];

    // Clone headers
    (cloned as any)[REQUEST_HEADERS] = new Headers();
    const headerList = getHeadersList((this as any)[REQUEST_HEADERS]);
    setHeadersList(
      (cloned as any)[REQUEST_HEADERS],
      headerList.map((h: any) => [h[0], h[1]]),
    );
    const currentGuard = ((this as any)[REQUEST_HEADERS] as any).guard ||
      "request";
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

(globalThis as any).Request = Request;
