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
    let request;
    // const baseURL = getLocationHref();

    // 4.
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
    } else { // 6.
      if (!Object.prototype.isPrototypeOf.call(RequestPrototype, input)) {
        throw new TypeError("Unreachable");
      }
      const originalReq = input.#request;
      // fold in of step 12 from below
      request = cloneInnerRequest(originalReq, true);
      request.redirectCount = 0; // reset to 0 - cloneInnerRequest copies the value
      signal = input.#signal;
    }

    // 12. is folded into the else statement of step 6 above.

    // 22.
    if (init.redirect !== undefined) {
      request.redirectMode = init.redirect;
    }

    // 25.
    if (init.method !== undefined) {
      const method = init.method;
      // fast path: check for known methods
      request.method = KNOWN_METHODS[method] ??
        validateAndNormalizeMethod(method);
    }

    // 26.
    if (init.signal !== undefined) {
      signal = init.signal;
    }

    // NOTE: non standard extension. This handles Deno.HttpClient parameter
    if (init.client !== undefined) {
      if (
        init.client !== null &&
        !ObjectPrototypeIsPrototypeOf(HttpClientPrototype, init.client)
      ) {
        throw webidl.makeException(
          TypeError,
          "`client` must be a Deno.HttpClient",
          prefix,
          "Argument 2",
        );
      }
      request.clientRid = init.client?.[internalRidSymbol] ?? null;
    }

    // 28.
    this[_request] = request;

    // 29 & 30.
    if (signal !== null) {
      this[_signalCache] = createDependentAbortSignal([signal], prefix);
    }

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
