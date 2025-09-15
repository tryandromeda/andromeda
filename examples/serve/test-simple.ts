// Test basic HTTP functionality

console.log("Testing HTTP server...");
console.log("Andromeda.serve:", typeof Andromeda.serve);

// Simple test with Response object
Andromeda.serve((req) => {
  console.log("Request received!");
  console.log("Method:", req.method);
  console.log("URL:", req.url);

  // Return a proper Response object
  return new Response("Hello from Andromeda HTTP server!", {
    status: 200,
    headers: {
      "Content-Type": "text/plain",
      "X-Powered-By": "Andromeda",
    },
  });
});

console.log("Server running on http://127.0.0.1:8080/");
