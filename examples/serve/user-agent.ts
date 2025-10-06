// User-Agent header example
// Similar to Deno's @std/http/user-agent example

console.log("🚀 Starting User-Agent server...\n");

Andromeda.serve((req) => {
  const userAgent = req.headers.get("user-agent") ?? "Unknown";
  const method = req.method;
  const path = new URL(req.url).pathname;

  console.log(`${method} ${path} - User-Agent: ${userAgent}`);

  return new Response(`Hello! Your user agent is: ${userAgent}`, {
    status: 200,
    headers: {
      "Content-Type": "text/plain",
    },
  });
});

console.log("✅ Server running on http://127.0.0.1:8080/");
console.log("  Try: curl http://127.0.0.1:8080/");
