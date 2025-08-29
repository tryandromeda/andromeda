console.log("=== Navigator.userAgent API Example ===");

console.log("User Agent:", navigator.userAgent);

// Platform detection from user agent
const getPlatformInfo = () =>
  navigator.userAgent.includes("Windows") ?
    "Windows" :
    (navigator.userAgent.includes("Macintosh") ||
        navigator.userAgent.includes("Mac OS") ?
      "macOS" :
      (navigator.userAgent.includes("Linux") ? "Linux" : "Unknown Platform"));

console.log("Detected Platform:", getPlatformInfo());

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
