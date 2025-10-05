// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * Subresource Integrity (SRI) Implementation
 */
interface SRIMetadata {
  /**
   * Hash algorithm (sha256, sha384, sha512)
   */
  alg: string;
  /**
   * Base64-encoded hash value
   */
  val: string;
  options?: string;
}

interface SRIRequest {
  /**
   * Integrity attribute value from the request (e.g., "sha256-abc123...")
   */
  integrity?: string;
  /**
   * Request origin
   */
  origin?: string;
  /**
   * CORS mode of the request (e.g., "cors", "no-cors", "same-origin")
   */
  mode?: string;
}

interface SRIResponse {
  /**
   * Response body bytes
   */
  body?: Uint8Array | null;
  /**
   * Response type (basic, cors, opaque, etc.)
   */
  type?: string;
  /**
   * Response URL
   */
  url?: string;
}

/**
 * Parse integrity metadata from an integrity attribute string
 *
 * Format: "sha256-hash1 sha384-hash2 sha512-hash3"
 * Multiple hashes can be specified, separated by whitespace
 */
function parseIntegrityMetadata(integrity: string): SRIMetadata[] {
  if (!integrity || integrity.trim() === "") {
    return [];
  }

  const metadata: SRIMetadata[] = [];

  // Split by whitespace to get individual hash expressions
  const expressions = integrity.trim().split(/\s+/);

  for (const expr of expressions) {
    // Skip empty expressions
    if (!expr) continue;

    // Check for options (hash?option1?option2)
    const parts = expr.split("?");
    const hashExpr = parts[0];
    const options = parts.slice(1).join("?");

    // Parse hash expression (algorithm-value)
    const dashIndex = hashExpr.indexOf("-");
    if (dashIndex === -1) {
      // Invalid format, skip
      continue;
    }

    const alg = hashExpr.substring(0, dashIndex).toLowerCase();
    const val = hashExpr.substring(dashIndex + 1);

    // Only accept supported algorithms
    if (alg !== "sha256" && alg !== "sha384" && alg !== "sha512") {
      // Unsupported algorithm, skip
      continue;
    }

    // Validate base64 format (basic check)
    if (!val || !/^[A-Za-z0-9+/]+=*$/.test(val)) {
      // Invalid base64, skip
      continue;
    }

    metadata.push({
      alg,
      val,
      options: options || undefined,
    });
  }

  return metadata;
}

/**
 * Get the strongest metadata from a set
 *
 * Priority: sha512 > sha384 > sha256
 *
 * @param metadataList - Array of metadata objects
 * @returns Array of metadata using the strongest algorithm
 */
function getStrongestMetadata(metadataList: SRIMetadata[]): SRIMetadata[] {
  if (metadataList.length === 0) {
    return [];
  }

  // Check for sha512 (strongest)
  const sha512 = metadataList.filter((m) => m.alg === "sha512");
  if (sha512.length > 0) {
    return sha512;
  }

  // Check for sha384
  const sha384 = metadataList.filter((m) => m.alg === "sha384");
  if (sha384.length > 0) {
    return sha384;
  }

  // Fall back to sha256
  const sha256 = metadataList.filter((m) => m.alg === "sha256");
  return sha256;
}

/**
 * Check if a resource is eligible for integrity validation
 *
 * Per spec, resources are only eligible if they are:
 * - Same-origin, OR
 * - Explicitly granted access via CORS (response type is "cors")
 *
 * This prevents attackers from brute-forcing hashes to read cross-origin data.
 *
 * @param request - The request
 * @param response - The response
 * @returns true if eligible, false otherwise
 */
function isResourceEligibleForIntegrityValidation(
  _request: SRIRequest,
  response: SRIResponse,
): boolean {
  // If no response type, assume not eligible
  if (!response.type) {
    return false;
  }

  // Same-origin resources are always eligible
  // Response type "basic" indicates same-origin
  if (response.type === "basic") {
    return true;
  }

  // CORS resources are eligible if CORS access was explicitly granted
  // Response type "cors" indicates successful CORS
  if (response.type === "cors") {
    return true;
  }

  // Opaque responses (cross-origin without CORS) are NOT eligible
  // This is a critical security requirement
  if (response.type === "opaque") {
    return false;
  }

  // Default to not eligible
  return false;
}

/**
 * Compute cryptographic hash of data
 *
 * @param algorithm - Hash algorithm (sha256, sha384, sha512)
 * @param data - Data to hash
 * @returns Base64-encoded hash, or null if algorithm not supported
 */
async function computeHash(
  algorithm: string,
  data: Uint8Array,
): Promise<string | null> {
  try {
    // Map algorithm name to SubtleCrypto format
    let alg: string;
    if (algorithm === "sha256") {
      alg = "SHA-256";
    } else if (algorithm === "sha384") {
      alg = "SHA-384";
    } else if (algorithm === "sha512") {
      alg = "SHA-512";
    } else {
      // Unsupported algorithm
      return null;
    }

    // Compute hash using Web Crypto API
    const hashBuffer = await crypto.subtle.digest(alg, data);

    // Convert to base64 (proper implementation for binary data)
    const hashArray = new Uint8Array(hashBuffer);
    const base64chars =
      "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let result = "";

    for (let i = 0; i < hashArray.length; i += 3) {
      const b1 = hashArray[i];
      const b2 = i + 1 < hashArray.length ? hashArray[i + 1] : 0;
      const b3 = i + 2 < hashArray.length ? hashArray[i + 2] : 0;

      result += base64chars[b1 >> 2];
      result += base64chars[((b1 & 3) << 4) | (b2 >> 4)];
      result += i + 1 < hashArray.length ?
        base64chars[((b2 & 15) << 2) | (b3 >> 6)] :
        "=";
      result += i + 2 < hashArray.length ? base64chars[b3 & 63] : "=";
    }

    return result;
  } catch (error) {
    // Hash computation failed
    console.error(`SRI: Failed to compute ${algorithm} hash:`, error);
    return null;
  }
}

/**
 * Check if a response matches the integrity metadata
 *
 * This is the main SRI validation algorithm.
 *
 * @param request - The request with integrity metadata
 * @param response - The response to validate
 * @returns Promise<boolean> - true if valid, false if invalid
 */
export async function doesResponseMatchIntegrityMetadata(
  request: SRIRequest,
  response: SRIResponse,
): Promise<boolean> {
  // If no integrity metadata, validation passes (no requirement)
  if (!request.integrity || request.integrity.trim() === "") {
    return true;
  }

  // Parse integrity metadata
  const metadataList = parseIntegrityMetadata(request.integrity);

  // If parsing failed or no valid metadata, treat as no integrity requirement
  if (metadataList.length === 0) {
    return true;
  }

  // Check if resource is eligible for integrity validation
  if (!isResourceEligibleForIntegrityValidation(request, response)) {
    // Per spec: "This algorithm returns false if the response is not eligible
    // for integrity validation since Subresource Integrity requires CORS"
    console.warn(
      "SRI: Resource not eligible for integrity validation (CORS required for cross-origin resources)",
    );
    return false;
  }

  // Get response body
  if (!response.body || response.body.length === 0) {
    // Empty body - check if any hash matches empty data
    const emptyData = new Uint8Array(0);
    const strongest = getStrongestMetadata(metadataList);

    for (const metadata of strongest) {
      const computed = await computeHash(metadata.alg, emptyData);
      if (computed === metadata.val) {
        return true;
      }
    }

    return false;
  }

  // Get the strongest metadata (highest priority algorithm)
  const strongestMetadata = getStrongestMetadata(metadataList);

  // Try each metadata with the strongest algorithm
  for (const metadata of strongestMetadata) {
    // Compute hash of response body
    const computedHash = await computeHash(metadata.alg, response.body);

    if (computedHash === null) {
      // Hash computation failed, try next
      continue;
    }

    // Compare computed hash with expected hash
    if (computedHash === metadata.val) {
      // Match found! Resource integrity verified
      return true;
    }
  }

  // No matching hash found - integrity validation failed
  console.error(
    "SRI: Integrity validation failed. Expected hash does not match computed hash.",
  );
  return false;
}

interface GlobalWithSRI {
  __doesResponseMatchIntegrityMetadata?:
    typeof doesResponseMatchIntegrityMetadata;
}

(globalThis as unknown as GlobalWithSRI).__doesResponseMatchIntegrityMetadata =
  doesResponseMatchIntegrityMetadata;
