// Query parameters example
// Similar to URL parsing in Deno examples

console.log("🚀 Starting query params server...\n");

Andromeda.serve((req) => {
  const url = new URL(req.url);
  const searchParams = url.searchParams;

  console.log(`${req.method} ${url.pathname}${url.search}`);

  if (url.pathname === "/search") {
    const query = searchParams.get("q");
    const limit = searchParams.get("limit") || "10";

    if (!query) {
      return new Response(
        JSON.stringify({
          error: "Missing 'q' query parameter",
          example: "/search?q=hello&limit=5",
        }),
        {
          status: 400,
          headers: {
            "Content-Type": "application/json",
          },
        },
      );
    }

    return new Response(
      JSON.stringify({
        query: query,
        limit: parseInt(limit),
        results: [
          `Result 1 for "${query}"`,
          `Result 2 for "${query}"`,
          `Result 3 for "${query}"`,
        ],
      }),
      {
        status: 200,
        headers: {
          "Content-Type": "application/json",
        },
      },
    );
  }

  return new Response(
    JSON.stringify({
      message: "Try /search?q=hello&limit=5",
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
console.log("  Try: curl 'http://127.0.0.1:8080/search?q=hello&limit=5'");
