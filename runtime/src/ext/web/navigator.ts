// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * Navigator interface implementation according to HTML specification
 * Provides information about the user agent and browser environment
 */
class AndromedaNavigator {
    /**
     * Returns the complete User-Agent header.
     * According to HTML spec, this must return the default User-Agent value.
     */
    get userAgent(): string {
        return internal_navigator_user_agent();
    }

    /**
     * Returns the string "Mozilla" for compatibility
     */
    get appCodeName(): string {
        return "Mozilla";
    }

    /**
     * Returns the string "Netscape" for compatibility
     */
    get appName(): string {
        return "Netscape";
    }

    /**
     * Returns the version of the browser
     */
    get appVersion(): string {
        // According to HTML spec, this should start with "5.0 ("
        // We extract the part after "Mozilla/" from the user agent
        const ua = this.userAgent;
        const mozillaIndex = ua.indexOf("Mozilla/");
        if (mozillaIndex !== -1) {
            return ua.substring(mozillaIndex + 8); // Skip "Mozilla/"
        }
        return "5.0 (Unknown)";
    }

    /**
     * Returns the name of the platform
     */
    get platform(): string {
        const ua = this.userAgent;

        if (ua.includes("Windows NT")) {
            if (ua.includes("Win64; x64")) {
                return "Win32";
            }
            if (ua.includes("ARM64")) {
                return "Win32";
            }
            return "Win32";
        }
        if (ua.includes("Macintosh")) {
            return "MacIntel";
        }
        if (ua.includes("Linux x86_64")) {
            return "Linux x86_64";
        }
        if (ua.includes("Linux aarch64")) {
            return "Linux aarch64";
        }
        if (ua.includes("Linux")) {
            return "Linux";
        }
        return "Unknown";
    }

    /**
     * Returns the string "Gecko" for compatibility
     */
    get product(): string {
        return "Gecko";
    }

    /**
     * Returns the product sub-version
     * For WebKit compatibility mode, returns "20030107"
     */
    get productSub(): string {
        return "20030107"; // WebKit compatibility mode
    }

    /**
     * Returns the vendor string
     * For WebKit compatibility mode, returns "Apple Computer, Inc."
     */
    get vendor(): string {
        return "Apple Computer, Inc."; // WebKit compatibility mode
    }

    /**
     * Returns the vendor sub-version
     */
    get vendorSub(): string {
        return "";
    }
}

// Attach the navigator object to globalThis
(globalThis as unknown as { navigator: AndromedaNavigator }).navigator =
    new AndromedaNavigator();

// For compatibility, also provide clientInformation alias
(globalThis as unknown as { clientInformation: AndromedaNavigator })
    .clientInformation =
        (globalThis as unknown as { navigator: AndromedaNavigator }).navigator;
