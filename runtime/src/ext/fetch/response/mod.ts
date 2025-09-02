// deno-lint-ignore-file no-explicit-any

// Get helpers - using functions to avoid timing issues
const getHeaderHelpers = () =>
  globalThis[Symbol.for("andromeda.headers.helpers")] || {};
const getBodyHelpers = () =>
  globalThis[Symbol.for("andromeda.body.helpers")] || {};

class Response {
  #response;
  #headers: any;
  /**
   * The new Response(body, init) constructor steps are:
   * @see https://fetch.spec.whatwg.org/#dom-response
   */
  constructor(body: any, init: ResponseInit) {
    // 1. Set this's response to a new response.
    this.#response = makeResponse(init);

    // 2. Set this's headers to a new Headers object with this's relevant realm, whose header list is this's response's header list and guard is "response".
    this.#headers = new Headers();
    getHeaderHelpers().setHeadersList(
      this.#headers,
      this.#response.headersList || [],
    );
    getHeaderHelpers().setHeadersGuard(this.#headers, "response");

    // 3. Let bodyWithType be null.
    let bodyWithType = null;

    // 4. If body is non-null, then set bodyWithType to the result of extracting body.
    if (body != null) {
      const [extractedBody, type] = getBodyHelpers().extractBody(body);
      bodyWithType = { body: extractedBody, type };
    }
    // 5. Perform initialize a response given this, init, and bodyWithType.
    initializeAResponse(this, init, bodyWithType);
  }

  static getResponse(response: Response) {
    return response.#response;
  }

  get type() {
    return this.#response.type;
  }

  /**
   * The url getter steps are to return this's response's URL.
   */
  get url() {
    return this.#response.url;
  }

  /**
   * Returns true if the response is the result of a redirect; otherwise false.
   */
  get redirected() {
    return this.#response.url.length > 1;
  }

  /** The status getter steps are to return this's response's status. */
  get status() {
    return this.#response.status;
  }

  /** The ok getter steps are to return true if this's response's status is an ok status; otherwise false. */
  get ok() {
    const status = this.#response.status;
    return status >= 200 && status <= 299;
  }

  /** The statusText getter steps are to return this's response's status message. */
  get statusText() {
    return this.#response.statusText;
  }

  /**
   * Gets the headers.
   */
  get headers() {
    return this.#headers;
  }

  // TODO
  get body() {
    return this.#response.body;
  }

  /**
   * Returns a promise that resolves with the result of parsing the response body as text.
   */
  async text(): Promise<string> {
    const body = this.#response.body;
    if (!body) {
      return "";
    }

    // Handle body object with source property (from extractBody)
    let actualBody = body;
    if (body && typeof body === "object" && "source" in body) {
      actualBody = body.source;
    }

    // If body is a Uint8Array, decode it as UTF-8
    if (actualBody instanceof Uint8Array) {
      const decoder = new TextDecoder();
      return decoder.decode(actualBody);
    }

    // If body is already a string
    if (typeof actualBody === "string") {
      return actualBody;
    }

    // Handle object with numeric keys (like the one we get from TLS)
    if (typeof actualBody === "object" && actualBody !== null) {
      const length = actualBody.length || Object.keys(actualBody).length;
      const uint8Array = new Uint8Array(length);
      for (let i = 0; i < length; i++) {
        uint8Array[i] = actualBody[i] || 0;
      }
      const decoder = new TextDecoder();
      return decoder.decode(uint8Array);
    }

    return String(actualBody);
  }

  /**
   * Returns a promise that resolves with the result of parsing the response body as JSON.
   */
  async json(): Promise<any> {
    const text = await this.text();
    return JSON.parse(text);
  }
}

const { getResponse } = Response;

function makeResponse(init: ResponseInit) {
  return {
    aborted: false,
    rangeRequested: false,
    timingAllowPassed: false,
    requestIncludesCredentials: false,
    type: "default",
    status: 200,
    timingInfo: null,
    cacheState: "",
    statusText: "",
    url: "",
    body: null,
    headersList: [],
    ...init,
  };
}

function initializeAResponse(
  response: Response,
  init: ResponseInit,
  body: {
    body: any;
    type: any;
  } | null,
) {
  // 1. If init["status"] is not in the range 200 to 599, inclusive, then throw a RangeError.
  if (
    init.status != null && (init.status < 200 || init.status > 599)
  ) {
    throw new RangeError(
      `The status provided (${init.status}) is not equal to 101 and outside the range [200, 599]`,
    );
  }

  // 2. If init["statusText"] is not the empty string and does not match the reason-phrase token production, then throw a TypeError.
  // TODO: implement RegExp.
  if (
    init.statusText && !isValidReasonPhrase(init.statusText)
  ) {
    throw new TypeError(
      `Invalid status text: "${init.statusText}"`,
    );
  }

  // 3. Set response's response's status to init["status"].
  if (init.status != null) {
    getResponse(response).status = init.status;
  }

  // 4. Set response's response's status message to init["statusText"].
  if (init.statusText != null) {
    getResponse(response).statusText = init.statusText;
  }

  // 5. If init["headers"] exists, then fill response's headers with init["headers"].
  if (init.headers != null) {
    // Fill the headers object with the provided headers
    getHeaderHelpers().fillHeaders(response.headers, init.headers);
  }

  // Handle headersList if provided
  if (init.headersList != null && Array.isArray(init.headersList)) {
    // Update the internal response headersList
    getResponse(response).headersList = init.headersList;
    // Also update the Headers object
    getHeaderHelpers().setHeadersList(response.headers, init.headersList);
  }

  // 6. If body is non-null, then:
  if (body != null) {
    // 1. If response's status is a null body status, then throw a TypeError.
    // NOTE: 101 and 103 are included in null body status due to their use elsewhere. They do not affect this step.
    if (nullBodyStatus(response.status)) {
      throw new TypeError(
        `Response with status ${response.status} cannot have a body.`,
      );
    }
    // 2. Set response's body to body's body.
    getResponse(response).body = body.body;
    // 3. If body's type is non-null and response's header list does not contain `Content-Type`, then append (`Content-Type`, body's type) to response's header list.
  }
}

// Check whether |statusText| is a ByteString and
// matches the Reason-Phrase token production.
// RFC 2616: https://tools.ietf.org/html/rfc2616
// RFC 7230: https://tools.ietf.org/html/rfc7230
// "reason-phrase = *( HTAB / SP / VCHAR / obs-text )"
// https://github.com/chromium/chromium/blob/94.0.4604.1/third_party/blink/renderer/core/fetch/response.cc#L116
function isValidReasonPhrase(statusText: string) {
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

globalThis.Response = Response;
