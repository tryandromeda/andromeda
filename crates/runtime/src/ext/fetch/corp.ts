// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Define minimal type interfaces for request and response
interface CorpRequest {
  origin?: string;
}

interface CorpResponse {
  headersList?: Array<[string, string]>;
  url?: string;
  requestIncludesCredentials?: boolean;
}

/**
 * Performs a cross-origin resource policy check
 *
 * Spec: https://fetch.spec.whatwg.org/#cross-origin-resource-policy-header
 *
 * This check ensures that responses respect the Cross-Origin-Resource-Policy header,
 * which allows servers to opt into protecting their resources from being loaded
 * cross-origin.
 *
 * @param request - The request being made
 * @param response - The response to check
 * @returns true if the response can be used, false if it should be blocked
 */
function corpCheck(request: CorpRequest, response: CorpResponse): boolean {
  // For now, we implement a simplified version without full embedder policy support
  // This is the core CORP check that works for most cases

  // Get the request origin
  const origin = request.origin;
  if (!origin || origin === "client") {
    // If no origin is set, allow the request
    return true;
  }

  // Perform the internal check with "unsafe-none" embedder policy
  // (most permissive, doesn't require CORP by default)
  return corpInternalCheck(origin, "unsafe-none", response, false);
}

/**
 * Performs the cross-origin resource policy internal check
 *
 * Spec: https://fetch.spec.whatwg.org/#cross-origin-resource-policy-internal-check
 *
 * @param origin - The origin making the request
 * @param embedderPolicyValue - The embedder policy value ("unsafe-none", "credentialless", or "require-corp")
 * @param response - The response to check
 * @param forNavigation - Whether this is for a navigation request
 * @returns true if allowed, false if blocked
 */
function corpInternalCheck(
  origin: string,
  embedderPolicyValue: string,
  response: CorpResponse,
  forNavigation: boolean,
): boolean {
  // Step 1: If forNavigation is true and embedderPolicyValue is "unsafe-none", return allowed
  if (forNavigation && embedderPolicyValue === "unsafe-none") {
    return true;
  }

  // Step 2: Let policy be the result of getting `Cross-Origin-Resource-Policy` from response's header list
  let policy = getHeader(
    response.headersList || [],
    "Cross-Origin-Resource-Policy",
  );

  // Note: If multiple CORP headers exist, or the value is invalid, this will be treated as null
  // The spec says: "This means that Cross-Origin-Resource-Policy: same-site, same-origin
  // ends up as allowed below as it will never match anything"

  // Step 3: If policy is neither "same-origin", "same-site", nor "cross-origin", then set policy to null
  if (
    policy !== "same-origin" && policy !== "same-site" &&
    policy !== "cross-origin"
  ) {
    policy = null;
  }

  // Step 4: If policy is null, then switch on embedderPolicyValue
  if (policy === null) {
    switch (embedderPolicyValue) {
      case "unsafe-none":
        // Do nothing, policy stays null
        break;

      case "credentialless":
        // Set policy to "same-origin" if:
        // - response's request-includes-credentials is true, or
        // - forNavigation is true
        if (response.requestIncludesCredentials || forNavigation) {
          policy = "same-origin";
        }
        break;

      case "require-corp":
        // Always set policy to "same-origin"
        policy = "same-origin";
        break;
    }
  }

  // Step 5: Switch on policy
  switch (policy) {
    case null:
    case "cross-origin":
      // Return allowed
      return true;

    case "same-origin": {
      // If origin is same origin with response's URL's origin, return allowed
      // Otherwise, return blocked
      const responseOrigin = getOriginFromURL(response.url || "");
      return isSameOrigin(origin, responseOrigin);
    }

    case "same-site": {
      // If all of the following are true:
      // - origin is schemelessly same site with response's URL's origin
      // - origin's scheme is "https" or response's URL's scheme is not "https"
      // then return allowed. Otherwise, return blocked.
      const responseOrigin = getOriginFromURL(response.url || "");

      const isSchemelesslySameSite = isSchemelesslySameSiteOrigin(
        origin,
        responseOrigin,
      );

      if (!isSchemelesslySameSite) {
        return false;
      }

      // Check the HTTPS requirement per spec:
      // The spec comment says: "Securely-transported responses will only match a
      // securely-transported initiator"
      //
      // This means:
      // - HTTPS response requires HTTPS origin (blocks HTTP → HTTPS)
      // - HTTP response allows any origin (allows HTTPS → HTTP)
      //
      // Spec condition: "origin's scheme is 'https' OR response's URL's scheme is not 'https'"
      // Analysis:
      // - HTTP origin (http) + HTTP response (http): FALSE OR TRUE = TRUE ✓ (allow)
      // - HTTP origin (http) + HTTPS response (https): FALSE OR FALSE = FALSE ✗ (block)
      // - HTTPS origin (https) + HTTP response (http): TRUE OR TRUE = TRUE ✓ (allow)
      // - HTTPS origin (https) + HTTPS response (https): TRUE OR FALSE = TRUE ✓ (allow)
      const originScheme = getSchemeFromOrigin(origin);
      const responseScheme = getSchemeFromURL(response.url || "");

      const allowed = (originScheme === "https") ||
        (responseScheme !== "https");

      return allowed;
    }

    default:
      // Unknown policy value, treat as blocked for safety
      return false;
  }
}

/**
 * Gets a header value from the headers list (case-insensitive)
 * Returns null if header is not found or if multiple values exist (per spec)
 */
function getHeader(
  headersList: Array<[string, string]>,
  name: string,
): string | null {
  if (!headersList) {
    return null;
  }

  const lowerName = name.toLowerCase();
  let foundValue: string | null = null;
  let foundCount = 0;

  for (const [headerName, headerValue] of headersList) {
    if (headerName.toLowerCase() === lowerName) {
      foundValue = headerValue;
      foundCount++;
      // If we find multiple headers with the same name, return null
      // per spec: "multiple CORP headers will have the same effect"
      if (foundCount > 1) {
        return null;
      }
    }
  }

  return foundValue;
}

/**
 * Extracts the origin from a URL string
 */
function getOriginFromURL(url: string): string {
  try {
    const parsedURL = new URL(url);
    return parsedURL.origin;
  } catch {
    return "";
  }
}

/**
 * Gets the scheme from an origin string
 */
function getSchemeFromOrigin(origin: string): string {
  try {
    const parsedURL = new URL(origin);
    return parsedURL.protocol.replace(":", "");
  } catch {
    // If origin is not a URL, try to extract scheme directly
    const colonIndex = origin.indexOf(":");
    if (colonIndex !== -1) {
      return origin.substring(0, colonIndex);
    }
    return "";
  }
}

/**
 * Gets the scheme from a URL string
 */
function getSchemeFromURL(url: string): string {
  try {
    const parsedURL = new URL(url);
    return parsedURL.protocol.replace(":", "");
  } catch {
    return "";
  }
}

/**
 * Checks if two origins are same-origin
 *
 * Two origins are same-origin if they have the same scheme, host, and port
 */
function isSameOrigin(originA: string, originB: string): boolean {
  try {
    const urlA = new URL(originA);
    const urlB = new URL(originB);

    return urlA.protocol === urlB.protocol &&
      urlA.hostname === urlB.hostname &&
      urlA.port === urlB.port;
  } catch {
    return false;
  }
}

/**
 * Checks if two origins are schemelessly same site
 *
 * Spec: https://html.spec.whatwg.org/multipage/browsers.html#schemelessly-same-site
 *
 * Two origins are schemelessly same site if:
 * - They are same origin, OR
 * - They have the same registrable domain (e.g., example.com for both sub.example.com and www.example.com)
 */
function isSchemelesslySameSiteOrigin(
  originA: string,
  originB: string,
): boolean {
  try {
    const urlA = new URL(originA);
    const urlB = new URL(originB);

    // Check if same origin first (fast path)
    if (
      urlA.protocol === urlB.protocol &&
      urlA.hostname === urlB.hostname &&
      urlA.port === urlB.port
    ) {
      return true;
    }

    // Check if they have the same registrable domain
    // This is a simplified implementation - a full implementation would use the Public Suffix List
    const domainA = getRegistrableDomain(urlA.hostname);
    const domainB = getRegistrableDomain(urlB.hostname);

    const result = domainA !== "" && domainA === domainB;
    return result;
  } catch {
    return false;
  }
}

/**
 * Gets the registrable domain from a hostname
 *
 * This is a simplified implementation that extracts the domain and TLD.
 * A full implementation would use the Public Suffix List.
 *
 * Examples:
 * - "www.example.com" -> "example.com"
 * - "sub.domain.example.co.uk" -> "example.co.uk" (simplified, should use PSL)
 * - "localhost" -> "localhost"
 * - "127.0.0.1" -> "127.0.0.1" (IP addresses are their own registrable domain)
 */
function getRegistrableDomain(hostname: string): string {
  // Handle IP addresses - each IP is its own registrable domain (not same-site with others)
  if (/^\d+\.\d+\.\d+\.\d+$/.test(hostname) || /^\[.*\]$/.test(hostname)) {
    // Return the IP address itself, which means different IPs are NOT same-site
    return hostname;
  }

  // Handle localhost
  if (hostname === "localhost") {
    return "localhost";
  }

  // Split by dots
  const parts = hostname.split(".");

  // If only one part (e.g., "localhost"), return as-is
  if (parts.length === 1) {
    return hostname;
  }

  // For simplicity, return last two parts for .com, .org, etc.
  // This is a simplification and doesn't handle multi-part TLDs like .co.uk properly
  // A proper implementation would use the Public Suffix List
  if (parts.length >= 2) {
    return parts.slice(-2).join(".");
  }

  return hostname;
}

interface GlobalWithCorp {
  __corpCheck?: typeof corpCheck;
  __corpInternalCheck?: typeof corpInternalCheck;
}

(globalThis as unknown as GlobalWithCorp).__corpCheck = corpCheck;
(globalThis as unknown as GlobalWithCorp).__corpInternalCheck =
  corpInternalCheck;
