// Basic "Hello World" example
// Similar to Deno's basic serve example

console.log("🚀 Starting basic server...\n");

Andromeda.serve((req) => {
  console.log(`${req.method} ${new URL(req.url).pathname}`);
  return new Response("Hello world!");
});

console.log("✅ Server running on http://127.0.0.1:8080/");
