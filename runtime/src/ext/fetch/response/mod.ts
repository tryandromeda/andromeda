// deno-lint-ignore-file no-explicit-any
interface ResponseInit {
  headers?: HeadersInit;
  status?: number;
  statusText?: string;
}

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

    // TODO: implement module
    // 2. Set this's headers to a new Headers object with this's relevant realm, whose header list is this's response's header list and guard is "response".

    // 3. Let bodyWithType be null.
    let bodyWithType = null;

    // 4. If body is non-null, then set bodyWithType to the result of extracting body.
    if (body != null) {
      const [extractedBody, type] = extractBody(body);
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
}

const { getResponse } = Response;

// TODO: headers
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
    init.statusText && isValidReasonPhrase(init.statusText)
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
    // TODO: get headerlist
    getResponse(response).headers = init.headers;
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

/**
 * TODO: when module is implemented, move to ext/fetch/body
 * To extract a body with type from a byte sequence or BodyInit object,
 * with an optional boolean keepalive (default false)
 * @see https://fetch.spec.whatwg.org/#concept-bodyinit-extract
 */
function extractBody(object: any, _keepalive = false) {
  // 1. Let stream be null.
  // deno-lint-ignore prefer-const
  let stream = null;
  // 2. If object is a ReadableStream object, then set stream to object.
  // TODO: implement ReadableStream
  // 3. Otherwise, if object is a Blob object, set stream to the result of running object's get stream.
  // 4. Otherwise, set stream to a new ReadableStream object, and set up stream with byte reading support.
  // 5. Assert: stream is a ReadableStream object.

  // 6. Let action be null.
  // deno-lint-ignore prefer-const
  let _action = null;

  // 7. Let source be null.
  let source = null;

  // 8. Let length be null.
  // deno-lint-ignore prefer-const
  let length = null;

  // 9. Let type be null.
  let type = null;

  // 10. Switch on object:
  if (typeof object == "string") {
    // scalar value string:
    // Set source to the UTF-8 encoding of object.
    // Set type to `text/plain;charset=UTF-8`.
    source = object;
    type = "text/plain;charset=UTF-8";
  } else {
    console.error("TODO: these are not yet supported");
    // Blob
    // Set source to object.
    // Set length to object's size.
    // If object's type attribute is not the empty byte sequence, set type to its value.

    // byte sequence:
    // Set source to object.

    // BufferSource:
    // Set source to a copy of the bytes held by object.

    // FormData:
    // Set action to this step: run the multipart/form-data encoding algorithm, with object's entry list and UTF-8.

    // Set source to object.

    // Set length to unclear, see html/6424 for improving this.

    // Set type to `multipart/form-data; boundary=`, followed by the multipart/form-data boundary string generated by the multipart/form-data encoding algorithm.

    // URLSearchParams:
    // Set source to the result of running the application/x-www-form-urlencoded serializer with object's list.

    // Set type to `application/x-www-form-urlencoded;charset=UTF-8`.

    // ReadableStream:
    // If keepalive is true, then throw a TypeError.
    // If object is disturbed or locked, then throw a TypeError.
  }

  // 11. If source is a byte sequence, then set action to a step that returns source and length to source's length.

  // 12. If action is non-null, then run these steps in parallel:
  //   1. Run action.
  //      Whenever one or more bytes are available and stream is not errored, enqueue the result of creating a Uint8Array from the available bytes into stream.
  //      When running action is done, close stream.

  // 13. Let body be a body whose stream is stream, source is source, and length is length.
  const body = { stream, source, length };

  // 14. Return (body, type).
  return [body, type];
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

// Export Response to globalThis
globalThis.Response = Response;