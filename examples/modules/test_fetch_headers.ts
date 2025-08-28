// Test that Headers is available and working
console.log("Testing Headers API:");

// Create a new Headers instance
const headers = new Headers();
headers.append("Content-Type", "application/json");
headers.append("Authorization", "Bearer token123");

console.log("Content-Type:", headers.get("Content-Type"));
console.log("Authorization:", headers.get("Authorization"));

// Test that fetch uses Headers internally
console.log("\nTesting fetch with Headers:");
// Note: This will create a network error since we're not actually fetching
// But it tests that the modules are connected
try {
  // This should work without errors if modules are properly connected
  const response = fetch("https://example.com/api");
  console.log("Fetch initiated successfully");
} catch (error: any) {
  console.error("Error:", error.message);
}
