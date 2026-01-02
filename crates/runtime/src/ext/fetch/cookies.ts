// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * Represents a stored cookie
 */
interface StoredCookie {
  name: string;
  value: string;
  domain: string;
  path: string;
  expires?: number; // Timestamp in milliseconds
  maxAge?: number; // Seconds
  secure: boolean;
  httpOnly: boolean;
  sameSite: "Strict" | "Lax" | "None" | undefined;
  partitioned: boolean;
  creationTime: number; // Timestamp when cookie was set
}

/**
 * Cookie jar - stores all cookies
 * Key: `${domain}:${path}:${name}`
 */
const cookieJar = new Map<string, StoredCookie>();

/**
 * Parse a Set-Cookie header value into a cookie object
 *
 * Format: name=value; attribute1; attribute2=value2; ...
 *
 * @param setCookieValue - The Set-Cookie header value
 * @param requestUrl - The URL that sent the Set-Cookie header
 * @returns Parsed cookie or null if invalid
 */
export function parseSetCookie(
  setCookieValue: string,
  requestUrl: string,
): StoredCookie | null {
  const url = new URL(requestUrl);

  // Split into parts
  const parts = setCookieValue.split(";").map(p => p.trim());
  if (parts.length === 0) return null;

  // Parse name=value
  const nameValue = parts[0];
  const eqIdx = nameValue.indexOf("=");
  if (eqIdx === -1) return null;

  const name = nameValue.substring(0, eqIdx).trim();
  let value = nameValue.substring(eqIdx + 1).trim();

  // Remove quotes if present
  if (value.startsWith('"') && value.endsWith('"')) {
    value = value.substring(1, value.length - 1);
  }

  // Initialize cookie with defaults
  const cookie: StoredCookie = {
    name,
    value,
    domain: url.hostname,
    path: "/",
    secure: false,
    httpOnly: false,
    sameSite: "Lax", // Default per spec
    partitioned: false,
    creationTime: Date.now(),
  };

  // Track if Domain was explicitly set (for __Host- validation)
  let domainWasSet = false;

  // Parse attributes
  for (let i = 1; i < parts.length; i++) {
    const attr = parts[i];
    const attrEqIdx = attr.indexOf("=");

    if (attrEqIdx === -1) {
      // Boolean attribute
      const attrName = attr.toLowerCase();
      if (attrName === "secure") {
        cookie.secure = true;
      } else if (attrName === "httponly") {
        cookie.httpOnly = true;
      } else if (attrName === "partitioned") {
        cookie.partitioned = true;
      }
    } else {
      // Key=value attribute
      const attrName = attr.substring(0, attrEqIdx).trim().toLowerCase();
      const attrValue = attr.substring(attrEqIdx + 1).trim();

      if (attrName === "domain") {
        domainWasSet = true;
        // Remove leading dot if present
        const domain = attrValue.startsWith(".") ?
          attrValue.substring(1) :
          attrValue;
        // Domain must be same as or parent of request domain
        if (isDomainMatch(url.hostname, domain)) {
          cookie.domain = domain;
        }
      } else if (attrName === "path") {
        cookie.path = attrValue;
      } else if (attrName === "expires") {
        const expiresDate = new Date(attrValue);
        if (!isNaN(expiresDate.getTime())) {
          cookie.expires = expiresDate.getTime();
        }
      } else if (attrName === "max-age") {
        const maxAge = parseInt(attrValue, 10);
        if (!isNaN(maxAge)) {
          cookie.maxAge = maxAge;
        }
      } else if (attrName === "samesite") {
        const sameSite = attrValue.toLowerCase();
        if (sameSite === "strict") {
          cookie.sameSite = "Strict";
        } else if (sameSite === "lax") {
          cookie.sameSite = "Lax";
        } else if (sameSite === "none") {
          cookie.sameSite = "None";
        }
      }
    }
  }

  // Validate cookie prefixes
  if (!validateCookiePrefix(cookie, requestUrl, domainWasSet)) {
    return null;
  }

  // Don't store cookies that are already expired
  if (isExpired(cookie, Date.now())) {
    return null;
  }

  return cookie;
}

/**
 * Check if a domain matches for cookie purposes
 *
 * @param requestDomain - The domain of the request
 * @param cookieDomain - The domain attribute of the cookie
 * @returns True if match
 */
function isDomainMatch(requestDomain: string, cookieDomain: string): boolean {
  if (requestDomain === cookieDomain) {
    return true;
  }

  // Check if requestDomain is subdomain of cookieDomain
  if (requestDomain.endsWith("." + cookieDomain)) {
    return true;
  }

  return false;
}

/**
 * Check if a path matches for cookie purposes
 *
 * @param requestPath - The path of the request
 * @param cookiePath - The path attribute of the cookie
 * @returns True if match
 */
function isPathMatch(requestPath: string, cookiePath: string): boolean {
  if (requestPath === cookiePath) {
    return true;
  }

  if (requestPath.startsWith(cookiePath)) {
    // cookiePath must end with / or requestPath must have / after cookiePath
    if (cookiePath.endsWith("/")) {
      return true;
    }
    if (
      requestPath.length > cookiePath.length &&
      requestPath[cookiePath.length] === "/"
    ) {
      return true;
    }
  }

  return false;
}

/**
 * Validate cookie prefixes (__Secure-, __Host-)
 *
 * @param cookie - The cookie to validate
 * @param requestUrl - The URL that set the cookie
 * @param domainWasSet - Whether Domain attribute was explicitly set
 * @returns True if valid
 */
function validateCookiePrefix(
  cookie: StoredCookie,
  requestUrl: string,
  domainWasSet: boolean,
): boolean {
  const url = new URL(requestUrl);

  if (
    cookie.name.startsWith("__Secure-") || cookie.name.startsWith("__Host-")
  ) {
    // Must be from secure origin
    if (url.protocol !== "https:") {
      return false;
    }
    // Must have Secure attribute
    if (!cookie.secure) {
      return false;
    }
  }

  if (cookie.name.startsWith("__Host-")) {
    // Must have Path=/
    if (cookie.path !== "/") {
      return false;
    }
    // Must not have Domain attribute at all
    if (domainWasSet) {
      return false;
    }
  }

  return true;
}

/**
 * Store a cookie in the cookie jar
 *
 * @param cookie - The cookie to store
 */
export function storeCookie(cookie: StoredCookie): void {
  const key = `${cookie.domain}:${cookie.path}:${cookie.name}`;
  cookieJar.set(key, cookie);
}

/**
 * Get cookies for a request URL
 *
 * Returns all cookies that match the URL's domain and path,
 * are not expired, and pass security checks.
 *
 * @param requestUrl - The URL of the request
 * @returns Array of matching cookies
 */
export function getCookiesForRequest(requestUrl: string): StoredCookie[] {
  const url = new URL(requestUrl);
  const now = Date.now();
  const matchingCookies: StoredCookie[] = [];

  for (const [key, cookie] of cookieJar.entries()) {
    // Check expiry
    if (isExpired(cookie, now)) {
      cookieJar.delete(key);
      continue;
    }

    // Check domain match
    if (!isDomainMatch(url.hostname, cookie.domain)) {
      continue;
    }

    // Check path match
    if (!isPathMatch(url.pathname, cookie.path)) {
      continue;
    }

    // Check secure attribute
    if (cookie.secure && url.protocol !== "https:") {
      continue;
    }

    matchingCookies.push(cookie);
  }

  // Sort by path length (more specific paths first), then by creation time
  matchingCookies.sort((a, b) => {
    if (a.path.length !== b.path.length) {
      return b.path.length - a.path.length;
    }
    return a.creationTime - b.creationTime;
  });

  return matchingCookies;
}

/**
 * Check if a cookie is expired
 *
 * @param cookie - The cookie to check
 * @param now - Current timestamp
 * @returns True if expired
 */
function isExpired(cookie: StoredCookie, now: number): boolean {
  // Check Max-Age first (takes precedence over Expires)
  if (cookie.maxAge !== undefined) {
    const expiryTime = cookie.creationTime + (cookie.maxAge * 1000);
    if (now >= expiryTime) {
      return true;
    }
  } else if (cookie.expires !== undefined) {
    if (now >= cookie.expires) {
      return true;
    }
  }

  return false;
}

/**
 * Generate Cookie header value for a request
 *
 * @param requestUrl - The URL of the request
 * @returns Cookie header value or null if no cookies
 */
export function generateCookieHeader(requestUrl: string): string | null {
  const cookies = getCookiesForRequest(requestUrl);

  if (cookies.length === 0) {
    return null;
  }

  // Format: name1=value1; name2=value2; ...
  return cookies.map(c => `${c.name}=${c.value}`).join("; ");
}

/**
 * Handle Set-Cookie header from a response
 *
 * @param setCookieValue - The Set-Cookie header value
 * @param requestUrl - The URL that sent the response
 * @returns True if cookie was stored, false otherwise
 */
export function handleSetCookie(
  setCookieValue: string,
  requestUrl: string,
): boolean {
  const cookie = parseSetCookie(setCookieValue, requestUrl);

  if (!cookie) {
    return false;
  }

  storeCookie(cookie);
  return true;
}

/**
 * Clear all cookies from the jar
 */
export function clearAllCookies(): void {
  cookieJar.clear();
}

/**
 * Clear cookies for a specific domain
 *
 * @param domain - Domain to clear cookies for
 */
export function clearCookiesForDomain(domain: string): void {
  for (const [key, cookie] of cookieJar.entries()) {
    if (isDomainMatch(domain, cookie.domain)) {
      cookieJar.delete(key);
    }
  }
}

/**
 * Get all cookies (for debugging/testing)
 *
 * @returns Array of all stored cookies
 */
export function getAllCookies(): StoredCookie[] {
  return Array.from(cookieJar.values());
}

/**
 * Clean up expired cookies
 */
export function cleanupExpiredCookies(): void {
  const now = Date.now();
  for (const [key, cookie] of cookieJar.entries()) {
    if (isExpired(cookie, now)) {
      cookieJar.delete(key);
    }
  }
}

// Export functions to globalThis for use in fetch implementation
(globalThis as unknown as {
  __parseSetCookie?: typeof parseSetCookie;
  __storeCookie?: typeof storeCookie;
  __getCookiesForRequest?: typeof getCookiesForRequest;
  __generateCookieHeader?: typeof generateCookieHeader;
  __handleSetCookie?: typeof handleSetCookie;
  __clearAllCookies?: typeof clearAllCookies;
  __clearCookiesForDomain?: typeof clearCookiesForDomain;
  __getAllCookies?: typeof getAllCookies;
  __cleanupExpiredCookies?: typeof cleanupExpiredCookies;
}).__parseSetCookie = parseSetCookie;
(globalThis as unknown as {
  __parseSetCookie?: typeof parseSetCookie;
  __storeCookie?: typeof storeCookie;
  __getCookiesForRequest?: typeof getCookiesForRequest;
  __generateCookieHeader?: typeof generateCookieHeader;
  __handleSetCookie?: typeof handleSetCookie;
  __clearAllCookies?: typeof clearAllCookies;
  __clearCookiesForDomain?: typeof clearCookiesForDomain;
  __getAllCookies?: typeof getAllCookies;
  __cleanupExpiredCookies?: typeof cleanupExpiredCookies;
}).__storeCookie = storeCookie;
(globalThis as unknown as {
  __parseSetCookie?: typeof parseSetCookie;
  __storeCookie?: typeof storeCookie;
  __getCookiesForRequest?: typeof getCookiesForRequest;
  __generateCookieHeader?: typeof generateCookieHeader;
  __handleSetCookie?: typeof handleSetCookie;
  __clearAllCookies?: typeof clearAllCookies;
  __clearCookiesForDomain?: typeof clearCookiesForDomain;
  __getAllCookies?: typeof getAllCookies;
  __cleanupExpiredCookies?: typeof cleanupExpiredCookies;
}).__getCookiesForRequest = getCookiesForRequest;
(globalThis as unknown as {
  __parseSetCookie?: typeof parseSetCookie;
  __storeCookie?: typeof storeCookie;
  __getCookiesForRequest?: typeof getCookiesForRequest;
  __generateCookieHeader?: typeof generateCookieHeader;
  __handleSetCookie?: typeof handleSetCookie;
  __clearAllCookies?: typeof clearAllCookies;
  __clearCookiesForDomain?: typeof clearCookiesForDomain;
  __getAllCookies?: typeof getAllCookies;
  __cleanupExpiredCookies?: typeof cleanupExpiredCookies;
}).__generateCookieHeader = generateCookieHeader;
(globalThis as unknown as {
  __parseSetCookie?: typeof parseSetCookie;
  __storeCookie?: typeof storeCookie;
  __getCookiesForRequest?: typeof getCookiesForRequest;
  __generateCookieHeader?: typeof generateCookieHeader;
  __handleSetCookie?: typeof handleSetCookie;
  __clearAllCookies?: typeof clearAllCookies;
  __clearCookiesForDomain?: typeof clearCookiesForDomain;
  __getAllCookies?: typeof getAllCookies;
  __cleanupExpiredCookies?: typeof cleanupExpiredCookies;
}).__handleSetCookie = handleSetCookie;
(globalThis as unknown as {
  __parseSetCookie?: typeof parseSetCookie;
  __storeCookie?: typeof storeCookie;
  __getCookiesForRequest?: typeof getCookiesForRequest;
  __generateCookieHeader?: typeof generateCookieHeader;
  __handleSetCookie?: typeof handleSetCookie;
  __clearAllCookies?: typeof clearAllCookies;
  __clearCookiesForDomain?: typeof clearCookiesForDomain;
  __getAllCookies?: typeof getAllCookies;
  __cleanupExpiredCookies?: typeof cleanupExpiredCookies;
}).__clearAllCookies = clearAllCookies;
(globalThis as unknown as {
  __parseSetCookie?: typeof parseSetCookie;
  __storeCookie?: typeof storeCookie;
  __getCookiesForRequest?: typeof getCookiesForRequest;
  __generateCookieHeader?: typeof generateCookieHeader;
  __handleSetCookie?: typeof handleSetCookie;
  __clearAllCookies?: typeof clearAllCookies;
  __clearCookiesForDomain?: typeof clearCookiesForDomain;
  __getAllCookies?: typeof getAllCookies;
  __cleanupExpiredCookies?: typeof cleanupExpiredCookies;
}).__clearCookiesForDomain = clearCookiesForDomain;
(globalThis as unknown as {
  __parseSetCookie?: typeof parseSetCookie;
  __storeCookie?: typeof storeCookie;
  __getCookiesForRequest?: typeof getCookiesForRequest;
  __generateCookieHeader?: typeof generateCookieHeader;
  __handleSetCookie?: typeof handleSetCookie;
  __clearAllCookies?: typeof clearAllCookies;
  __clearCookiesForDomain?: typeof clearCookiesForDomain;
  __getAllCookies?: typeof getAllCookies;
  __cleanupExpiredCookies?: typeof cleanupExpiredCookies;
}).__getAllCookies = getAllCookies;
(globalThis as unknown as {
  __parseSetCookie?: typeof parseSetCookie;
  __storeCookie?: typeof storeCookie;
  __getCookiesForRequest?: typeof getCookiesForRequest;
  __generateCookieHeader?: typeof generateCookieHeader;
  __handleSetCookie?: typeof handleSetCookie;
  __clearAllCookies?: typeof clearAllCookies;
  __clearCookiesForDomain?: typeof clearCookiesForDomain;
  __getAllCookies?: typeof getAllCookies;
  __cleanupExpiredCookies?: typeof cleanupExpiredCookies;
}).__cleanupExpiredCookies = cleanupExpiredCookies;
