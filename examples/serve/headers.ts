// Headers example
// Reading request headers and setting response headers

console.log("🚀 Starting headers server...\n");

Andromeda.serve((req) => {
  const url = new URL(req.url);
  const path = url.pathname;

  console.log(`${req.method} ${path}`);

  if (path === "/headers") {
    // Collect all request headers
    const requestHeaders: Record<string, string> = {};
    req.headers.forEach((value, key) => {
      requestHeaders[key] = value;
    });

    return new Response(
      JSON.stringify({
        message: "Your request headers",
        headers: requestHeaders,
      }),
      {
        status: 200,
        headers: {
          "Content-Type": "application/json",
          "X-Custom-Header": "Andromeda",
          "X-Server-Time": String(Date.now()),
        },
      },
    );
  }

  if (path === "/cors") {
    return new Response(
      JSON.stringify({
        message: "CORS enabled response",
      }),
      {
        status: 200,
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*",
          "Access-Control-Allow-Methods": "GET, POST, PUT, DELETE",
          "Access-Control-Allow-Headers": "Content-Type",
        },
      },
    );
  }

  return new Response(
    JSON.stringify({
      message: "Try /headers or /cors",
    }),
    {
      status: 200,
      headers: {
        "Content-Type": "application/json",
      },
    },
  );
});

console.log("✅ Server running on http://127.0.0.1:8080/");
console.log("  Try: curl http://127.0.0.1:8080/headers");
console.log("  Try: curl -v http://127.0.0.1:8080/cors");
