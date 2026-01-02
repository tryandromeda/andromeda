// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * Response Filtering
 * @see https://fetch.spec.whatwg.org/#concept-filtered-response
 */

// deno-lint-ignore-file no-explicit-any

const extractCORSExposedHeaderNames = (globalThis as any)
  .extractCORSExposedHeaderNames;

/**
 * Create a basic filtered response
 * @see https://fetch.spec.whatwg.org/#concept-basic-filtered-response
 *
 * A basic filtered response is a filtered response whose type is "basic" and
 * header list excludes any headers in internal response's header list whose
 * name is a forbidden response-header name.
 */
export function createBasicFilteredResponse(internalResponse: any): any {
  // Forbidden response header names
  const forbiddenHeaders = new Set([
    "set-cookie",
    "set-cookie2",
  ]);

  // Filter out forbidden headers
  const filteredHeadersList = (internalResponse.headersList || []).filter(
    ([name]: [string, string]) => !forbiddenHeaders.has(name.toLowerCase()),
  );

  return {
    ...internalResponse,
    type: "basic",
    headersList: filteredHeadersList,
    internalResponse,
  };
}

/**
 * Create a CORS filtered response
 * @see https://fetch.spec.whatwg.org/#concept-cors-filtered-response
 *
 * A CORS filtered response is a filtered response whose type is "cors" and
 * header list excludes any headers in internal response's header list whose
 * name is not a CORS-safelisted response-header name, given internal response's
 * CORS-exposed header-name list.
 */
export function createCORSFilteredResponse(internalResponse: any): any {
  const isCORSSafelistedResponseHeader = (globalThis as any)
    .isCORSSafelistedResponseHeader;

  // Get CORS-exposed header names from Access-Control-Expose-Headers
  const exposedHeaders = extractCORSExposedHeaderNames(
    internalResponse.headersList || [],
  );

  // Check if wildcard is present
  let hasWildcard = false;
  for (const [name, value] of internalResponse.headersList || []) {
    if (
      name.toLowerCase() === "access-control-expose-headers" &&
      value.includes("*")
    ) {
      hasWildcard = true;
      break;
    }
  }

  // Filter headers based on CORS-safelisted or exposed headers
  const filteredHeadersList = (internalResponse.headersList || []).filter(
    ([name]: [string, string]) => {
      const lowerName = name.toLowerCase();

      // Always allow CORS-safelisted response headers
      if (isCORSSafelistedResponseHeader(name)) {
        return true;
      }

      // Allow if explicitly exposed
      if (exposedHeaders.has(lowerName)) {
        return true;
      }

      // If wildcard, allow all except Set-Cookie headers
      if (
        hasWildcard && lowerName !== "set-cookie" && lowerName !== "set-cookie2"
      ) {
        return true;
      }

      return false;
    },
  );

  return {
    ...internalResponse,
    type: "cors",
    headersList: filteredHeadersList,
    internalResponse,
  };
}

/**
 * Create an opaque filtered response
 * @see https://fetch.spec.whatwg.org/#concept-opaque-filtered-response
 *
 * An opaque filtered response is a filtered response whose type is "opaque",
 * URL list is the empty list, status is 0, status message is the empty byte sequence,
 * header list is empty, and body is null.
 */
export function createOpaqueFilteredResponse(internalResponse: any): any {
  return {
    type: "opaque",
    urlList: [],
    url: "",
    status: 0,
    statusText: "",
    headersList: [],
    body: null,
    internalResponse,
  };
}

/**
 * Create an opaque-redirect filtered response
 * @see https://fetch.spec.whatwg.org/#concept-opaque-redirect-filtered-response
 *
 * An opaque-redirect filtered response is a filtered response whose type is
 * "opaqueredirect", status is 0, status message is the empty byte sequence,
 * header list is empty, and body is null.
 */
export function createOpaqueRedirectFilteredResponse(
  internalResponse: any,
): any {
  return {
    type: "opaqueredirect",
    urlList: internalResponse.urlList || [],
    url: internalResponse.url || "",
    status: 0,
    statusText: "",
    headersList: [],
    body: null,
    internalResponse,
  };
}

/**
 * Filter a response based on request's response tainting and mode
 * @see https://fetch.spec.whatwg.org/#main-fetch step 12
 */
export function filterResponse(response: any, request: any): any {
  // If already filtered, return as-is
  if (response.internalResponse) {
    return response;
  }

  // Determine filtering based on response tainting
  switch (request.responseTainting) {
    case "basic":
      return createBasicFilteredResponse(response);

    case "cors":
      return createCORSFilteredResponse(response);

    case "opaque":
      return createOpaqueFilteredResponse(response);

    default:
      // No filtering needed
      return response;
  }
}

(globalThis as any).createBasicFilteredResponse = createBasicFilteredResponse;
(globalThis as any).createCORSFilteredResponse = createCORSFilteredResponse;
(globalThis as any).createOpaqueFilteredResponse = createOpaqueFilteredResponse;
(globalThis as any).createOpaqueRedirectFilteredResponse =
  createOpaqueRedirectFilteredResponse;
(globalThis as any).filterResponse = filterResponse;
