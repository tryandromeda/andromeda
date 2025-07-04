// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * Brand information for User-Agent Client Hints
 */
interface UADataValues {
  brand: string;
  version: string;
}

/**
 * High entropy values for User-Agent Client Hints
 */
interface UAHighEntropyValues {
  architecture?: string;
  bitness?: string;
  brands?: UADataValues[];
  fullVersionList?: UADataValues[];
  mobile?: boolean;
  model?: string;
  platform?: string;
  platformVersion?: string;
  wow64?: boolean;
  formFactor?: string;
}

/**
 * NavigatorUAData interface implementation according to User-Agent Client Hints specification
 * https://developer.mozilla.org/en-US/docs/Web/API/NavigatorUAData
 */
class NavigatorUAData {
  /**
   * Returns an array of brand information containing the browser name and version
   */
  get brands(): UADataValues[] {
    // Return realistic brand data based on the user agent
    const ua = internal_navigator_user_agent();

    // Parse version from user agent (simplified approach without regex)
    let version = "119.0.0.0";
    const chromeIndex = ua.indexOf("Chrome/");
    if (chromeIndex !== -1) {
      const versionStart = chromeIndex + 7; // Length of "Chrome/"
      const versionEnd = ua.indexOf(" ", versionStart);
      if (versionEnd !== -1) {
        version = ua.substring(versionStart, versionEnd);
      } else {
        version = ua.substring(versionStart);
      }
    }

    return [
      { brand: "Not A(Brand", version: "99.0.0.0" },
      { brand: "Google Chrome", version: version },
      { brand: "Chromium", version: version },
    ];
  }

  /**
   * Returns true if the user-agent is running on a mobile device
   */
  get mobile(): boolean {
    const ua = internal_navigator_user_agent();
    return ua.includes("Mobile") || ua.includes("Android") || ua.includes("iPhone") ||
      ua.includes("iPad");
  }

  /**
   * Returns the platform brand the user-agent is running on
   */
  get platform(): string {
    const ua = internal_navigator_user_agent();

    if (ua.includes("Windows NT")) {
      return "Windows";
    }
    if (ua.includes("Macintosh")) {
      return "macOS";
    }
    if (ua.includes("Linux")) {
      return "Linux";
    }
    if (ua.includes("Android")) {
      return "Android";
    }
    if (ua.includes("iPhone") || ua.includes("iPad")) {
      return "iOS";
    }
    return "Unknown";
  }

  /**
   * Returns a Promise that resolves with high entropy values
   */
  getHighEntropyValues(hints: string[]): Promise<UAHighEntropyValues> {
    const ua = internal_navigator_user_agent();
    const result: UAHighEntropyValues = {};

    for (const hint of hints) {
      switch (hint) {
        case "architecture":
          if (ua.includes("x86_64") || ua.includes("Win64; x64")) {
            result.architecture = "x86";
          } else if (ua.includes("aarch64") || ua.includes("ARM64")) {
            result.architecture = "arm";
          } else {
            result.architecture = "x86";
          }
          break;

        case "bitness":
          result.bitness = ua.includes("Win64") || ua.includes("x86_64") || ua.includes("aarch64") ?
            "64" :
            "32";
          break;

        case "brands":
          result.brands = this.brands;
          break;

        case "fullVersionList":
          result.fullVersionList = this.brands;
          break;

        case "mobile":
          result.mobile = this.mobile;
          break;

        case "model":
          result.model = "";
          break;

        case "platform":
          result.platform = this.platform;
          break;

        case "platformVersion":
          if (ua.includes("Windows NT 10.0")) {
            result.platformVersion = "10.0.0";
          } else if (ua.includes("Windows NT 11.0")) {
            result.platformVersion = "11.0.0";
          } else if (ua.includes("Mac OS X")) {
            // Find Mac OS X version without regex
            const macIndex = ua.indexOf("Mac OS X ");
            if (macIndex !== -1) {
              const versionStart = macIndex + 9; // Length of "Mac OS X "
              const versionEnd = ua.indexOf(")", versionStart);
              if (versionEnd !== -1) {
                const macVersion = ua.substring(versionStart, versionEnd);
                result.platformVersion = macVersion.replace(/_/g, ".");
              } else {
                result.platformVersion = "10.15.7";
              }
            } else {
              result.platformVersion = "10.15.7";
            }
          } else {
            result.platformVersion = "0.0.0";
          }
          break;

        case "wow64":
          result.wow64 = false;
          break;

        case "formFactor":
          result.formFactor = this.mobile ? "Mobile" : "Desktop";
          break;
      }
    }

    return Promise.resolve(result);
  }

  /**
   * Returns a JSON representation of the low entropy properties
   */
  toJSON(): { brands: UADataValues[]; mobile: boolean; platform: string; } {
    return {
      brands: this.brands,
      mobile: this.mobile,
      platform: this.platform,
    };
  }
}

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

  /**
   * Returns a NavigatorUAData object for User-Agent Client Hints
   */
  get userAgentData(): NavigatorUAData {
    return new NavigatorUAData();
  }
}

// Attach the navigator object to globalThis
(globalThis as unknown as { navigator: AndromedaNavigator; }).navigator = new AndromedaNavigator();

// For compatibility, also provide clientInformation alias
(globalThis as unknown as { clientInformation: AndromedaNavigator; })
  .clientInformation = (globalThis as unknown as { navigator: AndromedaNavigator; }).navigator;
