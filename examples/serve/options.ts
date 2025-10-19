// Andromeda.serve with various options
// Demonstrates how to use ServeOptions to configure the server

console.log("🚀 Starting server with custom options...\n");

// Example 1: Custom port and hostname
console.log("Example 1: Using custom port and hostname");
Andromeda.serve({
  port: 3000,
  hostname: "0.0.0.0",
  handler: (req) => {
    const url = new URL(req.url);
    console.log(`${req.method} ${url.pathname}`);

    if (url.pathname === "/") {
      return new Response(
        JSON.stringify({
          message: "Server running with custom options!",
          port: 3000,
          hostname: "0.0.0.0",
          paths: {
            "/": "Home page",
            "/info": "Server information",
            "/echo": "Echo query parameters",
          },
        }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" },
        },
      );
    }

    if (url.pathname === "/info") {
      return new Response(
        JSON.stringify({
          server: "Andromeda",
          version: "1.0.0",
          options: {
            port: 3000,
            hostname: "0.0.0.0",
            description: "This server is configured with custom options",
          },
        }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" },
        },
      );
    }

    if (url.pathname === "/echo") {
      const params: Record<string, string> = {};
      url.searchParams.forEach((value, key) => {
        params[key] = value;
      });

      return new Response(
        JSON.stringify({
          message: "Echo endpoint",
          queryParams: params,
          method: req.method,
          url: req.url,
        }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" },
        },
      );
    }

    return new Response(
      JSON.stringify({
        error: "Not Found",
        path: url.pathname,
      }),
      {
        status: 404,
        headers: { "Content-Type": "application/json" },
      },
    );
  },
});

console.log("✅ Server is running!");
console.log("   Hostname: 0.0.0.0");
console.log("   Port: 3000");
console.log("\n📋 Test Commands:");
console.log("  curl http://localhost:3000/");
console.log("  curl http://localhost:3000/info");
console.log("  curl 'http://localhost:3000/echo?name=Andromeda&version=1.0'");
console.log("");
