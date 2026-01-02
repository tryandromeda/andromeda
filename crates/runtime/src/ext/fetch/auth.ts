// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * Represents an authentication challenge from a server
 */
interface AuthChallenge {
  scheme: string;
  realm?: string;
  // Basic auth
  charset?: string;
  // Digest auth
  domain?: string;
  nonce?: string;
  opaque?: string;
  stale?: boolean;
  algorithm?: string;
  qop?: string[];
  userhash?: boolean;
}

/**
 * Represents stored credentials for an origin+realm
 */
interface StoredCredential {
  username: string;
  password: string;
  realm: string;
  origin: string;
}

/**
 * Represents a request that needs authentication
 */
interface AuthRequest {
  url: string;
  method: string;
  headers: { [key: string]: string; };
}

/**
 * Represents a response with authentication challenges
 */
interface AuthResponse {
  status: number;
  headers: { [key: string]: string; };
}

/**
 * Simple in-memory credential store
 * Key: `${origin}:${realm}`
 * Value: StoredCredential
 */
const credentialStore = new Map<string, StoredCredential>();

/**
 * Parse WWW-Authenticate header value into challenges
 *
 * Handles both simple format (Basic realm="...") and complex format (Digest with multiple params)
 * Multiple challenges are separated by commas AFTER the parameters of the previous challenge.
 *
 * @param headerValue - The WWW-Authenticate header value
 * @returns Array of parsed challenges
 */
export function parseAuthChallenges(headerValue: string): AuthChallenge[] {
  const challenges: AuthChallenge[] = [];

  // Strategy: Look for scheme names (words followed by space or comma)
  // Schemes: Basic, Digest, Bearer, etc. (start with capital letter)

  let currentChallenge = "";
  let inQuotes = false;

  for (let i = 0; i < headerValue.length; i++) {
    const char = headerValue[i];

    if (char === '"') {
      inQuotes = !inQuotes;
      currentChallenge += char;
    } else if (char === "," && !inQuotes) {
      // Check if next word (after comma and whitespace) looks like a scheme name
      // Scheme names are typically capitalized words (Basic, Digest, Bearer)
      let nextWord = "";
      let j = i + 1;
      while (j < headerValue.length && /\s/.test(headerValue[j])) j++;
      while (j < headerValue.length && /[A-Za-z0-9-]/.test(headerValue[j])) {
        nextWord += headerValue[j];
        j++;
      }

      // If next word starts with capital and is followed by space or end, it's likely a new scheme
      if (nextWord.length > 0 && /^[A-Z]/.test(nextWord)) {
        // This comma separates challenges
        if (currentChallenge.trim()) {
          const challenge = parseSingleChallenge(currentChallenge.trim());
          if (challenge) challenges.push(challenge);
        }
        currentChallenge = "";
      } else {
        // This comma is within parameters
        currentChallenge += char;
      }
    } else {
      currentChallenge += char;
    }
  }

  if (currentChallenge.trim()) {
    const challenge = parseSingleChallenge(currentChallenge.trim());
    if (challenge) challenges.push(challenge);
  }

  return challenges;
}

/**
 * Parse a single authentication challenge
 *
 * Format: "scheme param1=value1, param2=value2, ..."
 *
 * @param challengeStr - Single challenge string
 * @returns Parsed challenge or null if invalid
 */
function parseSingleChallenge(challengeStr: string): AuthChallenge | null {
  const spaceIdx = challengeStr.indexOf(" ");
  if (spaceIdx === -1) {
    // No parameters, just scheme
    return { scheme: challengeStr.trim().toLowerCase() };
  }

  const scheme = challengeStr.substring(0, spaceIdx).trim().toLowerCase();
  const paramsStr = challengeStr.substring(spaceIdx + 1).trim();

  const challenge: AuthChallenge = { scheme };

  // Parse parameters (key=value pairs)
  const params = parseAuthParams(paramsStr);

  // Map to challenge properties
  challenge.realm = params.realm;
  challenge.charset = params.charset;
  challenge.domain = params.domain;
  challenge.nonce = params.nonce;
  challenge.opaque = params.opaque;
  challenge.stale = params.stale?.toLowerCase() === "true";
  challenge.algorithm = params.algorithm;
  challenge.userhash = params.userhash?.toLowerCase() === "true";

  // Parse qop (can be comma-separated list)
  if (params.qop) {
    challenge.qop = params.qop.split(",").map((q: string) => q.trim());
  }

  return challenge;
}

/**
 * Parse authentication parameters from a parameter string
 *
 * Handles quoted and unquoted values
 *
 * @param paramsStr - Parameter string (e.g., 'realm="example", charset="UTF-8"')
 * @returns Object with parameter key-value pairs
 */
function parseAuthParams(paramsStr: string): Record<string, string> {
  const params: Record<string, string> = {};

  let current = "";
  let inQuotes = false;

  for (let i = 0; i < paramsStr.length; i++) {
    const char = paramsStr[i];
    if (char === '"') {
      inQuotes = !inQuotes;
    } else if (char === "," && !inQuotes) {
      processParam(current.trim(), params);
      current = "";
    } else {
      current += char;
    }
  }
  if (current.trim()) {
    processParam(current.trim(), params);
  }

  return params;
}

/**
 * Process a single parameter (key=value) and add to params object
 */
function processParam(param: string, params: Record<string, string>): void {
  const eqIdx = param.indexOf("=");
  if (eqIdx === -1) return;

  const key = param.substring(0, eqIdx).trim().toLowerCase();
  let value = param.substring(eqIdx + 1).trim();

  // Remove quotes if present
  if (value.startsWith('"') && value.endsWith('"')) {
    value = value.substring(1, value.length - 1);
  }

  params[key] = value;
}

/**
 * Generate Basic authentication Authorization header value
 *
 * Format: "Basic <base64(username:password)>"
 *
 * @param username - Username
 * @param password - Password
 * @returns Authorization header value
 */
export function generateBasicAuth(username: string, password: string): string {
  const credentials = `${username}:${password}`;
  const encoded = btoa(credentials);
  return `Basic ${encoded}`;
}

/**
 * Generate Digest authentication Authorization header value
 *
 * Implements RFC 7616 Digest Access Authentication
 *
 * @param username - Username
 * @param password - Password
 * @param challenge - The digest challenge from server
 * @param request - The request being authenticated
 * @param nc - Nonce count (hex string, e.g., "00000001")
 * @returns Authorization header value or null if generation fails
 */
export async function generateDigestAuth(
  username: string,
  password: string,
  challenge: AuthChallenge,
  request: AuthRequest,
  nc: string = "00000001",
): Promise<string | null> {
  if (!challenge.nonce || !challenge.realm) {
    console.error("Digest auth: Missing required nonce or realm");
    return null;
  }

  const algorithm = (challenge.algorithm || "MD5").toUpperCase();
  const realm = challenge.realm;
  const nonce = challenge.nonce;
  const opaque = challenge.opaque;
  const uri = new URL(request.url).pathname + new URL(request.url).search;
  const method = request.method;

  // Generate client nonce (cnonce)
  const cnonce = generateCnonce();

  // Determine qop (quality of protection)
  let qop: string | undefined;
  if (challenge.qop && challenge.qop.length > 0) {
    // Prefer 'auth' over 'auth-int'
    qop = challenge.qop.includes("auth") ? "auth" : challenge.qop[0];
  }

  // Compute response hash
  const response = await computeDigestResponse(
    username,
    password,
    realm,
    nonce,
    uri,
    method,
    algorithm,
    qop,
    nc,
    cnonce,
  );

  if (!response) {
    console.error("Digest auth: Failed to compute response hash");
    return null;
  }

  // Build Authorization header value
  let authValue =
    `Digest username="${username}", realm="${realm}", nonce="${nonce}", uri="${uri}", response="${response}"`;

  if (algorithm && algorithm !== "MD5") {
    authValue += `, algorithm=${algorithm}`;
  }

  if (qop) {
    authValue += `, qop=${qop}, nc=${nc}, cnonce="${cnonce}"`;
  }

  if (opaque) {
    authValue += `, opaque="${opaque}"`;
  }

  return authValue;
}

/**
 * Compute the digest response hash
 *
 * response = H(H(A1) ":" nonce ":" H(A2))
 * - With qop=auth: response = H(H(A1) ":" nonce ":" nc ":" cnonce ":" qop ":" H(A2))
 *
 * A1 = username ":" realm ":" password
 * A2 = method ":" uri
 *
 * @returns The response hash string
 */
async function computeDigestResponse(
  username: string,
  password: string,
  realm: string,
  nonce: string,
  uri: string,
  method: string,
  algorithm: string,
  qop?: string,
  nc?: string,
  cnonce?: string,
): Promise<string | null> {
  try {
    // A1 = username:realm:password
    const a1 = `${username}:${realm}:${password}`;
    const ha1 = await hashString(a1, algorithm);

    if (!ha1) return null;

    // A2 = method:uri
    const a2 = `${method}:${uri}`;
    const ha2 = await hashString(a2, algorithm);

    if (!ha2) return null;

    // Response hash
    let responseData: string;
    if (qop && nc && cnonce) {
      responseData = `${ha1}:${nonce}:${nc}:${cnonce}:${qop}:${ha2}`;
    } else {
      responseData = `${ha1}:${nonce}:${ha2}`;
    }

    return await hashString(responseData, algorithm);
  } catch (error) {
    console.error("Error computing digest response:", error);
    return null;
  }
}

/**
 * Hash a string using the specified algorithm
 *
 * @param data - String to hash
 * @param algorithm - Hash algorithm (MD5, SHA-256, SHA-512)
 * @returns Lowercase hex string of the hash
 */
async function hashString(
  data: string,
  algorithm: string,
): Promise<string | null> {
  try {
    // Convert algorithm name to Web Crypto API format
    let cryptoAlgorithm: string;
    if (algorithm === "MD5") {
      // MD5 not supported in Web Crypto API, need fallback
      // For now, return null (can implement MD5 manually if needed)
      console.warn("MD5 not supported in Web Crypto API");
      return null;
    } else if (algorithm === "SHA-256") {
      cryptoAlgorithm = "SHA-256";
    } else if (algorithm === "SHA-512") {
      cryptoAlgorithm = "SHA-512";
    } else {
      console.error(`Unsupported digest algorithm: ${algorithm}`);
      return null;
    }

    const encoder = new TextEncoder();
    const dataBuffer = encoder.encode(data);
    const hashBuffer = await crypto.subtle.digest(cryptoAlgorithm, dataBuffer);
    const hashArray = new Uint8Array(hashBuffer);

    // Convert to lowercase hex string
    return Array.from(hashArray)
      .map(b => b.toString(16).padStart(2, "0"))
      .join("");
  } catch (error) {
    console.error("Error hashing string:", error);
    return null;
  }
}

/**
 * Generate a random client nonce (cnonce)
 *
 * @returns Random hex string
 */
function generateCnonce(): string {
  const array = new Uint8Array(16);
  crypto.getRandomValues(array);
  return Array.from(array)
    .map(b => b.toString(16).padStart(2, "0"))
    .join("");
}

/**
 * Store credentials for an origin and realm
 *
 * @param origin - Origin (e.g., "https://example.com")
 * @param realm - Authentication realm
 * @param username - Username
 * @param password - Password
 */
export function storeCredentials(
  origin: string,
  realm: string,
  username: string,
  password: string,
): void {
  const key = `${origin}:${realm}`;
  credentialStore.set(key, { username, password, realm, origin });
}

/**
 * Retrieve stored credentials for an origin and realm
 *
 * @param origin - Origin (e.g., "https://example.com")
 * @param realm - Authentication realm
 * @returns Stored credentials or null if not found
 */
export function getStoredCredentials(
  origin: string,
  realm: string,
): StoredCredential | null {
  const key = `${origin}:${realm}`;
  return credentialStore.get(key) || null;
}

/**
 * Clear all stored credentials
 */
export function clearAllCredentials(): void {
  credentialStore.clear();
}

/**
 * Clear credentials for a specific origin
 *
 * @param origin - Origin to clear credentials for
 */
export function clearCredentialsForOrigin(origin: string): void {
  for (const [key, cred] of credentialStore.entries()) {
    if (cred.origin === origin) {
      credentialStore.delete(key);
    }
  }
}

/**
 * Handle authentication for a request/response pair
 *
 * This is the main entry point for authentication handling.
 *
 * Flow:
 * 1. Parse WWW-Authenticate challenges from response
 * 2. Select most secure challenge (prefer Digest SHA-512 > SHA-256 > Basic)
 * 3. Check for stored credentials for origin+realm
 * 4. Generate Authorization header
 * 5. Return header value or null if auth cannot be handled
 *
 * @param request - The request that received a 401
 * @param response - The 401 response with challenges
 * @returns Authorization header value or null
 */
export async function handleAuthentication(
  request: AuthRequest,
  response: AuthResponse,
): Promise<string | null> {
  // Parse challenges from WWW-Authenticate header
  const wwwAuth = response.headers["www-authenticate"] ||
    response.headers["WWW-Authenticate"];
  if (!wwwAuth) {
    return null;
  }

  const challenges = parseAuthChallenges(wwwAuth);
  if (challenges.length === 0) {
    return null;
  }

  // Select best challenge (prefer Digest with SHA-512 > SHA-256 > Basic)
  const challenge = selectBestChallenge(challenges);
  if (!challenge) {
    return null;
  }

  // Get origin from request URL
  const url = new URL(request.url);
  const origin = url.origin;
  const realm = challenge.realm || "default";

  // Check for stored credentials
  const credentials = getStoredCredentials(origin, realm);
  if (!credentials) {
    // No stored credentials, cannot proceed
    // In a real implementation, this would prompt the user
    return null;
  }

  // Generate Authorization header based on scheme
  if (challenge.scheme === "basic") {
    return generateBasicAuth(credentials.username, credentials.password);
  } else if (challenge.scheme === "digest") {
    return await generateDigestAuth(
      credentials.username,
      credentials.password,
      challenge,
      request,
    );
  } else {
    console.warn(`Unsupported authentication scheme: ${challenge.scheme}`);
    return null;
  }
}

/**
 * Select the best (most secure) challenge from available challenges
 *
 * Priority:
 * 1. Digest with SHA-512
 * 2. Digest with SHA-256
 * 3. Digest with MD5
 * 4. Basic
 *
 * @param challenges - Array of challenges
 * @returns Best challenge or null
 */
function selectBestChallenge(
  challenges: AuthChallenge[],
): AuthChallenge | null {
  if (challenges.length === 0) return null;

  // Look for Digest with SHA-512
  const digestSha512 = challenges.find(
    c => c.scheme === "digest" && c.algorithm?.toUpperCase() === "SHA-512",
  );
  if (digestSha512) return digestSha512;

  // Look for Digest with SHA-256
  const digestSha256 = challenges.find(
    c => c.scheme === "digest" && c.algorithm?.toUpperCase() === "SHA-256",
  );
  if (digestSha256) return digestSha256;

  // Look for Digest with MD5 or unspecified algorithm (defaults to MD5)
  const digestMd5 = challenges.find(
    c =>
      c.scheme === "digest" &&
      (!c.algorithm || c.algorithm.toUpperCase() === "MD5"),
  );
  if (digestMd5) return digestMd5;

  // Look for Basic
  const basic = challenges.find(c => c.scheme === "basic");
  if (basic) return basic;

  // No supported challenge found
  return null;
}

// Export functions to globalThis for use in fetch implementation
(globalThis as unknown as {
  __parseAuthChallenges?: typeof parseAuthChallenges;
  __generateBasicAuth?: typeof generateBasicAuth;
  __generateDigestAuth?: typeof generateDigestAuth;
  __storeCredentials?: typeof storeCredentials;
  __getStoredCredentials?: typeof getStoredCredentials;
  __clearAllCredentials?: typeof clearAllCredentials;
  __clearCredentialsForOrigin?: typeof clearCredentialsForOrigin;
  __handleAuthentication?: typeof handleAuthentication;
}).__parseAuthChallenges = parseAuthChallenges;
(globalThis as unknown as {
  __parseAuthChallenges?: typeof parseAuthChallenges;
  __generateBasicAuth?: typeof generateBasicAuth;
  __generateDigestAuth?: typeof generateDigestAuth;
  __storeCredentials?: typeof storeCredentials;
  __getStoredCredentials?: typeof getStoredCredentials;
  __clearAllCredentials?: typeof clearAllCredentials;
  __clearCredentialsForOrigin?: typeof clearCredentialsForOrigin;
  __handleAuthentication?: typeof handleAuthentication;
}).__generateBasicAuth = generateBasicAuth;
(globalThis as unknown as {
  __parseAuthChallenges?: typeof parseAuthChallenges;
  __generateBasicAuth?: typeof generateBasicAuth;
  __generateDigestAuth?: typeof generateDigestAuth;
  __storeCredentials?: typeof storeCredentials;
  __getStoredCredentials?: typeof getStoredCredentials;
  __clearAllCredentials?: typeof clearAllCredentials;
  __clearCredentialsForOrigin?: typeof clearCredentialsForOrigin;
  __handleAuthentication?: typeof handleAuthentication;
}).__generateDigestAuth = generateDigestAuth;
(globalThis as unknown as {
  __parseAuthChallenges?: typeof parseAuthChallenges;
  __generateBasicAuth?: typeof generateBasicAuth;
  __generateDigestAuth?: typeof generateDigestAuth;
  __storeCredentials?: typeof storeCredentials;
  __getStoredCredentials?: typeof getStoredCredentials;
  __clearAllCredentials?: typeof clearAllCredentials;
  __clearCredentialsForOrigin?: typeof clearCredentialsForOrigin;
  __handleAuthentication?: typeof handleAuthentication;
}).__storeCredentials = storeCredentials;
(globalThis as unknown as {
  __parseAuthChallenges?: typeof parseAuthChallenges;
  __generateBasicAuth?: typeof generateBasicAuth;
  __generateDigestAuth?: typeof generateDigestAuth;
  __storeCredentials?: typeof storeCredentials;
  __getStoredCredentials?: typeof getStoredCredentials;
  __clearAllCredentials?: typeof clearAllCredentials;
  __clearCredentialsForOrigin?: typeof clearCredentialsForOrigin;
  __handleAuthentication?: typeof handleAuthentication;
}).__getStoredCredentials = getStoredCredentials;
(globalThis as unknown as {
  __parseAuthChallenges?: typeof parseAuthChallenges;
  __generateBasicAuth?: typeof generateBasicAuth;
  __generateDigestAuth?: typeof generateDigestAuth;
  __storeCredentials?: typeof storeCredentials;
  __getStoredCredentials?: typeof getStoredCredentials;
  __clearAllCredentials?: typeof clearAllCredentials;
  __clearCredentialsForOrigin?: typeof clearCredentialsForOrigin;
  __handleAuthentication?: typeof handleAuthentication;
}).__clearAllCredentials = clearAllCredentials;
(globalThis as unknown as {
  __parseAuthChallenges?: typeof parseAuthChallenges;
  __generateBasicAuth?: typeof generateBasicAuth;
  __generateDigestAuth?: typeof generateDigestAuth;
  __storeCredentials?: typeof storeCredentials;
  __getStoredCredentials?: typeof getStoredCredentials;
  __clearAllCredentials?: typeof clearAllCredentials;
  __clearCredentialsForOrigin?: typeof clearCredentialsForOrigin;
  __handleAuthentication?: typeof handleAuthentication;
}).__clearCredentialsForOrigin = clearCredentialsForOrigin;
(globalThis as unknown as {
  __parseAuthChallenges?: typeof parseAuthChallenges;
  __generateBasicAuth?: typeof generateBasicAuth;
  __generateDigestAuth?: typeof generateDigestAuth;
  __storeCredentials?: typeof storeCredentials;
  __getStoredCredentials?: typeof getStoredCredentials;
  __clearAllCredentials?: typeof clearAllCredentials;
  __clearCredentialsForOrigin?: typeof clearCredentialsForOrigin;
  __handleAuthentication?: typeof handleAuthentication;
}).__handleAuthentication = handleAuthentication;
