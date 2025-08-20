// Test fetch API functionality
console.log("Testing Fetch API...");

// Test Headers
const headers = new Headers();
headers.append("Content-Type", "application/json");
headers.append("X-Custom-Header", "test-value");

console.log("Headers created:");
console.log("  Content-Type:", headers.get("Content-Type"));
console.log("  X-Custom-Header:", headers.get("X-Custom-Header"));

// Test Request
const request = new Request("https://example.com/api", {
  method: "POST",
  headers: {
    "Authorization": "Bearer token123"
  }
});

console.log("\nRequest created:");
console.log("  URL:", request.url);
console.log("  Method:", request.method);
console.log("  Authorization:", request.headers.get("Authorization"));

// Test Response
const response = new Response(JSON.stringify({ message: "Hello" }), {
  status: 200,
  statusText: "OK",
  headers: {
    "Content-Type": "application/json"
  }
});

console.log("\nResponse created:");
console.log("  Status:", response.status);
console.log("  Status Text:", response.statusText);
console.log("  OK:", response.ok);
console.log("  Content-Type:", response.headers.get("Content-Type"));

// Test fetch (mock implementation)
console.log("\nTesting fetch (mock)...");
fetch("https://example.com/test")
  .then((res: Response) => {
    console.log("  Fetch successful!");
    console.log("  Response status:", res.status);
    console.log("  Response ok:", res.ok);
  })
  .catch((err: Error) => {
    console.error("  Fetch failed:", err);
  });

console.log("\nAll tests completed!");