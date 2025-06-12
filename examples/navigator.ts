// Navigator.userAgent API example for Andromeda
// This demonstrates the navigator.userAgent implementation according to the HTML specification
// Reference: https://html.spec.whatwg.org/multipage/system-state.html#dom-navigator-useragent

console.log("=== Navigator.userAgent API Example ===");

// Basic usage - get the user agent string
console.log("User Agent:", navigator.userAgent);

// Common use case: Browser/runtime detection (though feature detection is preferred)
function detectRuntime() {
  const ua = navigator.userAgent;

  if (ua.includes("Andromeda")) {
    return "Andromeda Runtime";
  } else if (ua.includes("Chrome")) {
    return "Chrome";
  } else if (ua.includes("Firefox")) {
    return "Firefox";
  } else if (ua.includes("Safari")) {
    return "Safari";
  } else {
    return "Unknown";
  }
}

console.log("Detected Runtime:", detectRuntime());

// Platform detection from user agent
function getPlatformInfo() {
  const ua = navigator.userAgent;

  if (ua.includes("Windows")) {
    return "Windows";
  } else if (ua.includes("Macintosh") || ua.includes("Mac OS")) {
    return "macOS";
  } else if (ua.includes("Linux")) {
    return "Linux";
  } else {
    return "Unknown Platform";
  }
}

console.log("Detected Platform:", getPlatformInfo());

// All NavigatorID properties as per HTML spec
console.log("\n=== Complete Navigator Properties ===");
console.log("appCodeName:", navigator.appCodeName); // Always "Mozilla"
console.log("appName:", navigator.appName); // Always "Netscape"
console.log("appVersion:", navigator.appVersion); // Version info
console.log("platform:", navigator.platform); // Platform name
console.log("product:", navigator.product); // Always "Gecko"
console.log("productSub:", navigator.productSub); // Product sub-version
console.log("userAgent:", navigator.userAgent); // Complete user agent string
console.log("vendor:", navigator.vendor); // Vendor string
console.log("vendorSub:", navigator.vendorSub); // Always empty string

// clientInformation alias (legacy compatibility)
console.log("\n=== Legacy Compatibility ===");
console.log("clientInformation.userAgent:", clientInformation.userAgent);
console.log("clientInformation === navigator:", clientInformation === navigator);

console.log("\n✅ Navigator.userAgent API example completed successfully!");
