// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * CORS Protocol Implementation
 * @see https://fetch.spec.whatwg.org/#cors-protocol
 */

// deno-lint-ignore-file no-explicit-any

/**
 * CORS-safelisted request headers (excluding forbidden headers)
 * @see https://fetch.spec.whatwg.org/#cors-safelisted-request-header
 */
const CORS_SAFELISTED_REQUEST_HEADERS = new Set([
  "accept",
  "accept-language",
  "content-language",
  "content-type",
]);

/**
 * CORS-safelisted response header names
 * @see https://fetch.spec.whatwg.org/#cors-safelisted-response-header-name
 */
const CORS_SAFELISTED_RESPONSE_HEADERS = new Set([
  "cache-control",
  "content-language",
  "content-length",
  "content-type",
  "expires",
  "last-modified",
  "pragma",
]);

/**
 * Simple methods that don't trigger CORS preflight
 * @see https://fetch.spec.whatwg.org/#cors-safelisted-method
 */
const CORS_SAFELISTED_METHODS = new Set(["GET", "HEAD", "POST"]);

/**
 * CORS non-wildcard request-header names
 * These headers cannot use wildcard in Access-Control-Allow-Headers
 * @see https://fetch.spec.whatwg.org/#cors-non-wildcard-request-header-name
 */
const CORS_NON_WILDCARD_REQUEST_HEADERS = new Set([
  "authorization",
]);

/**
 * Check if a method is CORS-safelisted
 * @see https://fetch.spec.whatwg.org/#cors-safelisted-method
 */
export function isCORSSafelistedMethod(method: string): boolean {
  return CORS_SAFELISTED_METHODS.has(method.toUpperCase());
}

/**
 * Check if a request header is CORS-safelisted
 * @see https://fetch.spec.whatwg.org/#cors-safelisted-request-header
 */
export function isCORSSafelistedRequestHeader(
  name: string,
  value: string,
): boolean {
  const lowerName = name.toLowerCase();

  if (!CORS_SAFELISTED_REQUEST_HEADERS.has(lowerName)) {
    return false;
  }

  // Additional checks for specific headers
  if (lowerName === "content-type") {
    const parsed = value.toLowerCase().split(";")[0].trim();
    return (
      parsed === "application/x-www-form-urlencoded" ||
      parsed === "multipart/form-data" ||
      parsed === "text/plain"
    );
  }

  // Value byte length should not exceed 128
  const encoder = new TextEncoder();
  if (encoder.encode(value).length > 128) {
    return false;
  }

  return true;
}

/**
 * Check if a response header name is CORS-safelisted
 * @see https://fetch.spec.whatwg.org/#cors-safelisted-response-header-name
 */
export function isCORSSafelistedResponseHeader(name: string): boolean {
  return CORS_SAFELISTED_RESPONSE_HEADERS.has(name.toLowerCase());
}

/**
 * Extract CORS-exposed header-name list from a response
 * @see https://fetch.spec.whatwg.org/#concept-cors-exposed-header-name-list
 */
export function extractCORSExposedHeaderNames(
  headersList: Array<[string, string]>,
): Set<string> {
  const exposedHeaders = new Set<string>();

  // Find Access-Control-Expose-Headers
  for (const [name, value] of headersList) {
    if (name.toLowerCase() === "access-control-expose-headers") {
      // Parse comma-separated list
      const headers = value.split(",").map((h) => h.trim().toLowerCase());
      for (const header of headers) {
        if (header === "*") {
          // Wildcard - but we still need to handle it per spec
          continue;
        }
        exposedHeaders.add(header);
      }
    }
  }

  return exposedHeaders;
}

/**
 * CORS check
 * @see https://fetch.spec.whatwg.org/#concept-cors-check
 * 
 * Returns true if the CORS check passes, false otherwise
 */
export function corsCheck(
  request: any,
  response: any,
): boolean {
  // 1. Let origin be the result of getting `Access-Control-Allow-Origin` from response's header list.
  const origin = getHeader(response.headersList, "access-control-allow-origin");

  // 2. If origin is null, then return failure.
  if (origin === null) {
    return false;
  }

  // 3. If request's credentials mode is not "include" and origin is `*`, then return success.
  if (request.credentialsMode !== "include" && origin === "*") {
    return true;
  }

  // 4. If the result of byte-serializing a request origin with request is origin, then:
  const requestOrigin = serializeOrigin(request.origin);
  if (requestOrigin === origin) {
    // 1. If request's credentials mode is not "include", then return success.
    if (request.credentialsMode !== "include") {
      return true;
    }

    // 2. Let credentials be the result of getting `Access-Control-Allow-Credentials` from response's header list.
    const credentials = getHeader(
      response.headersList,
      "access-control-allow-credentials",
    );

    // 3. If credentials is `true`, then return success.
    if (credentials === "true") {
      return true;
    }
  }

  // 5. Return failure.
  return false;
}

/**
 * CORS preflight check
 * @see https://fetch.spec.whatwg.org/#cors-preflight-check
 */
export function corsPreflightCheck(
  request: any,
  response: any,
): boolean {
  // 1. Let methods be the result of getting, decoding, and splitting `Access-Control-Allow-Methods` from response's header list.
  const methodsHeader = getHeader(
    response.headersList,
    "access-control-allow-methods",
  );
  const methods = methodsHeader
    ? methodsHeader.split(",").map((m) => m.trim().toUpperCase())
    : [];

  // 2. Let headerNames be the result of getting, decoding, and splitting `Access-Control-Allow-Headers` from response's header list.
  const headersHeader = getHeader(
    response.headersList,
    "access-control-allow-headers",
  );
  const headerNames = headersHeader
    ? headersHeader.split(",").map((h) => h.trim().toLowerCase())
    : [];

  // 3. If request's method is not in methods and is not a CORS-safelisted method, then return failure.
  if (
    !methods.includes(request.method.toUpperCase()) &&
    !isCORSSafelistedMethod(request.method)
  ) {
    return false;
  }

  // 4. For each header of request's header list's headers:
  for (const [name, value] of request.headersList || []) {
    const lowerName = name.toLowerCase();

    // 1. If header's name is a CORS non-wildcard request-header name and is not in headerNames, then return failure.
    if (
      CORS_NON_WILDCARD_REQUEST_HEADERS.has(lowerName) &&
      !headerNames.includes(lowerName)
    ) {
      return false;
    }

    // 2. If header's name is not in headerNames and header is not a CORS-safelisted request-header, then return failure.
    if (
      !headerNames.includes(lowerName) &&
      !headerNames.includes("*") &&
      !isCORSSafelistedRequestHeader(name, value)
    ) {
      return false;
    }
  }

  // 5. Return success.
  return true;
}

/**
 * Create a CORS preflight request
 * @see https://fetch.spec.whatwg.org/#cors-preflight-fetch
 */
export function createCORSPreflightRequest(request: any): any {
  // 1. Let preflightRequest be a new request whose method is `OPTIONS`,
  //    url is request's url, initiator is request's initiator,
  //    destination is request's destination, origin is request's origin,
  //    referrer is request's referrer, and referrer policy is request's referrer policy.
  const preflightRequest = {
    method: "OPTIONS",
    url: request.url,
    currentURL: request.currentURL,
    headersList: [] as Array<[string, string]>,
    mode: request.mode,
    credentials: request.credentialsMode,
    cache: "no-store",
    redirect: "manual",
    origin: request.origin,
    referrer: request.referrer,
    referrerPolicy: request.referrerPolicy,
    initiator: request.initiator,
    destination: request.destination,
    priority: request.priority,
  };

  // 2. Append (`Accept`, `*/*`) to preflightRequest's header list.
  preflightRequest.headersList.push(["Accept", "*/*"]);

  // 3. Append (`Access-Control-Request-Method`, request's method) to preflightRequest's header list.
  preflightRequest.headersList.push([
    "Access-Control-Request-Method",
    request.method,
  ]);

  // 4. Let headers be the CORS-unsafe request-header names with request's header list.
  const unsafeHeaders: string[] = [];
  for (const [name, value] of request.headersList || []) {
    if (!isCORSSafelistedRequestHeader(name, value)) {
      unsafeHeaders.push(name.toLowerCase());
    }
  }

  // 5. If headers is not empty, then:
  if (unsafeHeaders.length > 0) {
    // 1. Let value be the result of combining headers, separated by `,`.
    const value = unsafeHeaders.join(", ");

    // 2. Append (`Access-Control-Request-Headers`, value) to preflightRequest's header list.
    preflightRequest.headersList.push(["Access-Control-Request-Headers", value]);
  }

  // 6. Return preflightRequest.
  return preflightRequest;
}

/**
 * Helper: Get a header value from a header list
 */
function getHeader(
  headersList: Array<[string, string]>,
  name: string,
): string | null {
  const lowerName = name.toLowerCase();
  for (const [headerName, value] of headersList) {
    if (headerName.toLowerCase() === lowerName) {
      return value;
    }
  }
  return null;
}

/**
 * Helper: Serialize an origin for CORS checks
 */
function serializeOrigin(origin: any): string {
  if (typeof origin === "string") {
    return origin;
  }
  if (origin === null || origin === "null") {
    return "null";
  }
  // Handle URL-like origin objects
  if (origin.protocol && origin.host) {
    return `${origin.protocol}//${origin.host}`;
  }
  return String(origin);
}

// Export CORS utilities to globalThis for use in fetch algorithm
(globalThis as any).corsCheck = corsCheck;
(globalThis as any).corsPreflightCheck = corsPreflightCheck;
(globalThis as any).createCORSPreflightRequest = createCORSPreflightRequest;
(globalThis as any).isCORSSafelistedMethod = isCORSSafelistedMethod;
(globalThis as any).isCORSSafelistedRequestHeader = isCORSSafelistedRequestHeader;
(globalThis as any).extractCORSExposedHeaderNames = extractCORSExposedHeaderNames;
(globalThis as any).CORS_NON_WILDCARD_REQUEST_HEADERS = CORS_NON_WILDCARD_REQUEST_HEADERS;
