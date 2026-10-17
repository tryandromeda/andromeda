// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// deno-lint-ignore-file no-explicit-any

// Get body classes from globalThis (loaded from body files)
const extractBody = (globalThis as any).extractBody;
const _InnerBody = (globalThis as any).InnerBody;
const BodyMixin = (globalThis as any).BodyMixin;
const BODY_SYMBOL = (globalThis as any).BODY_SYMBOL;
const CONTENT_TYPE_SYMBOL = (globalThis as any).CONTENT_TYPE_SYMBOL;

// Import Headers class and utilities from the headers module
const Headers = globalThis.Headers as unknown as {
  new(init?: HeadersInit): Headers;
  setHeadersList(target: Headers, list: Array<[string, string]>): void;
  setHeadersGuard(target: Headers, guard: string): void;
};
const fillHeaders = (globalThis as any).fillHeaders as (
  headers: Headers,
  object: HeadersInit,
) => void;

const { setHeadersList, setHeadersGuard } = Headers;

// Use symbols instead of private fields for internal state
const RESPONSE_INTERNAL = Symbol("response.internal");
const RESPONSE_HEADERS = Symbol("response.headers");

/**
 * Response initialization options.
 * @see https://fetch.spec.whatwg.org/#responseinit
 */
interface ResponseInit {
  status?: number;
  statusText?: string;
  headers?: HeadersInit;
  // Internal properties
  headersList?: Array<[string, string]>;
  type?: ResponseType;
  url?: string;
}

class Response extends BodyMixin {
  /**
   * The new Response(body, init) constructor steps are
   * @see https://fetch.spec.whatwg.org/#dom-response
   */
  constructor(body: BodyInit | null = null, init: ResponseInit = {}) {
    // Extract body if provided (must be done before super)
    const extracted = body !== null ?
      extractBody(body) :
      { body: null, contentType: null };

    // Initialize the BodyMixin with the body and content type
    super(extracted.body, extracted.contentType);

    // Initialize internal fields using symbols (not declared as class fields)
    const internalResponse = makeResponse(init);
    (this as any)[RESPONSE_INTERNAL] = internalResponse;
    (this as any)[RESPONSE_HEADERS] = new Headers();

    // Set this's headers
    setHeadersList(
      (this as any)[RESPONSE_HEADERS],
      internalResponse.headersList || [],
    );
    setHeadersGuard((this as any)[RESPONSE_HEADERS], "response");

    // Perform initialize a response
    initializeAResponse(this, init, extracted.body, extracted.contentType);
  }

  /**
   * Gets the internal response object.
   */
  static getResponse(response: Response): InternalResponse {
    return (response as any)[RESPONSE_INTERNAL];
  }

  /**
   * Gets the response type.
   */
  get type(): ResponseType {
    return (this as any)[RESPONSE_INTERNAL].type;
  }

  /**
   * The url getter steps are to return this's response's URL.
   */
  get url(): string {
    return (this as any)[RESPONSE_INTERNAL].url;
  }

  /**
   * Returns true if the response is the result of a redirect; otherwise false.
   */
  get redirected(): boolean {
    const internal = (this as any)[RESPONSE_INTERNAL];
    const list = internal && internal.urlList;
    return Array.isArray(list) && list.length > 1;
  }

  /**
   * The status getter steps are to return this's response's status.
   */
  get status(): number {
    return (this as any)[RESPONSE_INTERNAL].status;
  }

  /**
   * The ok getter steps are to return true if this's response's status is an ok status; otherwise false.
   */
  get ok(): boolean {
    const status = (this as any)[RESPONSE_INTERNAL].status;
    return status >= 200 && status <= 299;
  }

  /**
   * The statusText getter steps are to return this's response's status message.
   */
  get statusText(): string {
    return (this as any)[RESPONSE_INTERNAL].statusText;
  }

  /**
   * Gets the headers.
   */
  get headers(): Headers {
    return (this as any)[RESPONSE_HEADERS];
  }

  /**
   * Gets the body as a ReadableStream.
   * @see https://fetch.spec.whatwg.org/#dom-body-body
   */
  get body(): ReadableStream<Uint8Array> | null {
    if (!(this as any)[BODY_SYMBOL]) {
      return null;
    }
    return (this as any)[BODY_SYMBOL].stream;
  }

  /**
   * Clones the response.
   * @see https://fetch.spec.whatwg.org/#dom-response-clone
   */
  clone(): Response {
    if (this.bodyUsed) {
      throw new TypeError("Response body is already used");
    }

    const clonedInternalResponse = cloneResponse(
      (this as any)[RESPONSE_INTERNAL],
    );

    let clonedBody: any = null;
    if ((this as any)[BODY_SYMBOL]) {
      clonedBody = (this as any)[BODY_SYMBOL].clone();
    }

    const cloned = new Response(null, {
      status: clonedInternalResponse.status,
      statusText: clonedInternalResponse.statusText,
      headersList: [...clonedInternalResponse.headersList],
    });

    (cloned as any)[BODY_SYMBOL] = clonedBody;
    (cloned as any)[CONTENT_TYPE_SYMBOL] = (this as any)[CONTENT_TYPE_SYMBOL];

    return cloned;
  }

  /**
   * Creates an error response.
   * @see https://fetch.spec.whatwg.org/#dom-response-error
   */
  static error(): Response {
    const response = new Response(null, {
      type: "error",
      status: 0,
      statusText: "",
    });
    setHeadersGuard((response as any)[RESPONSE_HEADERS], "immutable");
    return response;
  }

  /**
   * Creates a redirect response.
   * @see https://fetch.spec.whatwg.org/#dom-response-redirect
   */
  static redirect(url: string | URL, status: number = 302): Response {
    const parsedURL = new URL(url, globalThis.location?.href);

    if (!parsedURL) {
      throw new TypeError("Invalid URL");
    }

    if (![301, 302, 303, 307, 308].includes(status)) {
      throw new RangeError("Invalid redirect status");
    }

    const response = new Response(null, {
      status,
      statusText: "",
    });

    response.headers.set("Location", parsedURL.toString());

    return response;
  }

  /**
   * Creates a JSON response.
   * @see https://fetch.spec.whatwg.org/#dom-response-json
   */
  static json(data: unknown, init: ResponseInit = {}): Response {
    const bytes = new TextEncoder().encode(JSON.stringify(data));

    const response = new Response(bytes, init);

    response.headers.set("Content-Type", "application/json");

    return response;
  }
}

/**
 * Internal response representation.
 */
interface InternalResponse {
  aborted: boolean;
  rangeRequested: boolean;
  timingAllowPassed: boolean;
  requestIncludesCredentials: boolean;
  type: ResponseType;
  status: number;
  timingInfo: unknown;
  cacheState: string;
  statusText: string;
  url: string;
  headersList: Array<[string, string]>;
}

const { getResponse } = Response;

function makeResponse(init: ResponseInit): InternalResponse {
  return {
    aborted: false,
    rangeRequested: false,
    timingAllowPassed: false,
    requestIncludesCredentials: false,
    type: init.type || "default",
    status: init.status || 200,
    timingInfo: null,
    cacheState: "",
    statusText: init.statusText || "",
    url: init.url || "",
    headersList: init.headersList || [],
  };
}

function initializeAResponse(
  response: Response,
  init: ResponseInit,
  body: any,
  contentType: string | null,
): void {
  if (init.status != null && (init.status < 200 || init.status > 599)) {
    throw new RangeError(
      `The status provided (${init.status}) is outside the range [200, 599]`,
    );
  }

  if (init.statusText && !isValidReasonPhrase(init.statusText)) {
    throw new TypeError(`Invalid status text: "${init.statusText}"`);
  }

  if (init.status != null) {
    getResponse(response).status = init.status;
  }

  if (init.statusText != null) {
    getResponse(response).statusText = init.statusText;
  }

  if (init.headers != null) {
    fillHeaders(response.headers, init.headers);
  }

  // Handle headersList if provided
  if (init.headersList != null && Array.isArray(init.headersList)) {
    getResponse(response).headersList = init.headersList;
    setHeadersList(response.headers, init.headersList);
  }

  if (body != null) {
    if (nullBodyStatus(response.status)) {
      throw new TypeError(
        `Response with status ${response.status} cannot have a body.`,
      );
    }

    if (contentType && !response.headers.has("Content-Type")) {
      response.headers.set("Content-Type", contentType);
    }
  }
}

/**
 * Clones an internal response object.
 */
function cloneResponse(response: InternalResponse): InternalResponse {
  return {
    ...response,
    headersList: response.headersList.map(([k, v]) =>
      [k, v] as [string, string]
    ),
  };
}

/**
 * Checks if a status text is a valid reason phrase.
 * @see https://tools.ietf.org/html/rfc7230
 */
function isValidReasonPhrase(statusText: string): boolean {
  for (let i = 0; i < statusText.length; ++i) {
    const c = statusText.charCodeAt(i);
    if (
      !(
        c === 0x09 || // HTAB
        (c >= 0x20 && c <= 0x7e) || // SP / VCHAR
        (c >= 0x80 && c <= 0xff) // obs-text
      )
    ) {
      return false;
    }
  }
  return true;
}

/**
 * A null body status is a status that is 101, 103, 204, 205, or 304.
 * @see https://fetch.spec.whatwg.org/#null-body-status
 */
function nullBodyStatus(status: number): boolean {
  return status === 101 || status === 103 || status === 204 || status === 205 ||
    status === 304;
}

// Export Response to globalThis
globalThis.Response = Response as unknown as typeof globalThis.Response;
