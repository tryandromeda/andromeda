// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

interface MixedContentRequest {
  url?: string;
  client?: {
    origin?: string;
  } | null;
  destination?: string;
}

interface MixedContentResponse {
  url?: string;
}

interface MixedContentSettings {
  origin?: string;
}

/**
 * Check if a URL is a priori authenticated (potentially trustworthy)
 * A priori authenticated URLs are those that use secure protocols
 *
 * @param url - URL to check
 * @returns true if URL is authenticated (https://, wss://, file://, etc.)
 */
function isAPrioriAuthenticated(url: string): boolean {
  try {
    const urlObj = new URL(url);
    const scheme = urlObj.protocol.replace(":", "");

    // A priori authenticated schemes per spec
    const authenticatedSchemes = [
      "https",
      "wss",
      "file",
      "data", // data: URLs are considered potentially trustworthy
      "blob", // blob: URLs inherit security context
      "about", // about: URLs are browser-internal
    ];

    return authenticatedSchemes.includes(scheme);
  } catch {
    // If URL parsing fails, consider it not authenticated
    return false;
  }
}

/**
 * Check if an origin is a priori authenticated
 *
 * @param origin - Origin string (e.g., "https://example.com")
 * @returns true if origin uses secure scheme
 */
function isOriginAPrioriAuthenticated(origin: string): boolean {
  // Check if origin starts with secure scheme
  return origin.startsWith("https://") ||
    origin.startsWith("wss://") ||
    origin.startsWith("file://") ||
    origin.startsWith("about:");
}

/**
 * Does settings prohibit mixed security contexts?
 *
 * This algorithm checks if an environment settings object restricts mixed content.
 * Per spec § 4.3, if the settings object's origin is a priori authenticated,
 * then it prohibits mixed security contexts.
 *
 * Special cases:
 * - file: URLs are considered authenticated but don't prohibit mixed content
 * - about: URLs are treated specially and don't prohibit mixed content
 *
 * @param settings - Environment settings object
 * @returns "Prohibits Mixed Security Contexts" or "Does Not Restrict Mixed Security Contexts"
 */
function doesSettingsProhibitMixedSecurityContexts(
  settings: MixedContentSettings | null | undefined,
):
  | "Prohibits Mixed Security Contexts"
  | "Does Not Restrict Mixed Security Contexts"
{
  // If no settings or no origin, don't restrict
  if (!settings || !settings.origin) {
    return "Does Not Restrict Mixed Security Contexts";
  }

  // Special case: file: and about: origins don't prohibit mixed content
  // even though they are considered "secure"
  if (
    settings.origin.startsWith("file://") ||
    settings.origin.startsWith("about:")
  ) {
    return "Does Not Restrict Mixed Security Contexts";
  }

  // If settings' origin is a priori authenticated, prohibit mixed content
  if (isOriginAPrioriAuthenticated(settings.origin)) {
    return "Prohibits Mixed Security Contexts";
  }

  // Note: The spec also checks for embedding documents (frames), but since
  // we don't have full document/browsing context support yet, we simplify
  // to just checking the immediate origin

  return "Does Not Restrict Mixed Security Contexts";
}

/**
 * Should fetching request be blocked as mixed content?
 */
function shouldFetchingRequestBeBlockedAsMixedContent(
  request: MixedContentRequest,
): "allowed" | "blocked" {
  // Get the client's settings
  const client = request.client;

  // Check if settings prohibit mixed security contexts
  const prohibits = doesSettingsProhibitMixedSecurityContexts(client);

  // Return "allowed" if settings don't restrict mixed contexts
  if (prohibits === "Does Not Restrict Mixed Security Contexts") {
    return "allowed";
  }

  // Return "allowed" if request URL is a priori authenticated
  if (request.url && isAPrioriAuthenticated(request.url)) {
    return "allowed";
  }

  // Return "allowed" for top-level document navigations
  if (request.destination === "document") {
    // TODO: implement target browsing context check
    // For now, we allow top-level document navigations
    return "allowed";
  }

  // Note: The spec also mentions user controls to allow mixed content,
  // TODO: Implement mixed content

  // If none of the above conditions are met, block the request
  return "blocked";
}

/**
 * Should response to request be blocked as mixed content?
 */
function shouldResponseToRequestBeBlockedAsMixedContent(
  request: MixedContentRequest,
  response: MixedContentResponse,
): "allowed" | "blocked" {
  // Get the client's settings
  const client = request.client;

  // Check if settings prohibit mixed security contexts
  const prohibits = doesSettingsProhibitMixedSecurityContexts(client);

  // Return "allowed" if settings don't restrict mixed contexts
  if (prohibits === "Does Not Restrict Mixed Security Contexts") {
    return "allowed";
  }

  // Return "allowed" if response URL is a priori authenticated
  if (response.url && isAPrioriAuthenticated(response.url)) {
    return "allowed";
  }

  // Return "allowed" for top-level document navigations
  if (request.destination === "document") {
    return "allowed";
  }

  // If none of the above conditions are met, block the response
  return "blocked";
}

interface GlobalWithMixedContent {
  __shouldFetchingRequestBeBlockedAsMixedContent?:
    typeof shouldFetchingRequestBeBlockedAsMixedContent;
  __shouldResponseToRequestBeBlockedAsMixedContent?:
    typeof shouldResponseToRequestBeBlockedAsMixedContent;
}

(globalThis as unknown as GlobalWithMixedContent)
  .__shouldFetchingRequestBeBlockedAsMixedContent =
    shouldFetchingRequestBeBlockedAsMixedContent;
(globalThis as unknown as GlobalWithMixedContent)
  .__shouldResponseToRequestBeBlockedAsMixedContent =
    shouldResponseToRequestBeBlockedAsMixedContent;
