// Advanced routing example
// Pattern matching and multiple HTTP methods

console.log("🚀 Starting routing server...\n");

Andromeda.serve((req) => {
  const url = new URL(req.url);
  const path = url.pathname;
  const method = req.method;

  console.log(`${method} ${path}`);

  if (path === "/" && method === "GET") {
    return new Response(
      JSON.stringify({
        message: "Welcome to Andromeda Routing Example",
        routes: [
          "GET /",
          "GET /about",
          "GET /api/status",
          "POST /api/data",
          "GET /users/:id",
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

  if (path === "/about" && method === "GET") {
    return new Response(
      JSON.stringify({
        name: "Andromeda",
        version: "1.0.0",
        description: "A JavaScript runtime",
      }),
      {
        status: 200,
        headers: {
          "Content-Type": "application/json",
        },
      },
    );
  }

  if (path === "/api/status" && method === "GET") {
    return new Response(
      JSON.stringify({
        status: "ok",
        uptime: Date.now(),
      }),
      {
        status: 200,
        headers: {
          "Content-Type": "application/json",
        },
      },
    );
  }

  if (path === "/api/data" && method === "POST") {
    return new Response(
      JSON.stringify({
        message: "Data received",
        receivedAt: Date.now(),
      }),
      {
        status: 201,
        headers: {
          "Content-Type": "application/json",
        },
      },
    );
  }

  if (path.startsWith("/users/") && method === "GET") {
    const id = path.split("/")[2];

    if (id) {
      return new Response(
        JSON.stringify({
          userId: id,
          name: `User ${id}`,
        }),
        {
          status: 200,
          headers: {
            "Content-Type": "application/json",
          },
        },
      );
    }
  }

  // 404 Not Found
  return new Response(
    JSON.stringify({
      error: "Not Found",
      path: path,
      method: method,
    }),
    {
      status: 404,
      headers: {
        "Content-Type": "application/json",
      },
    },
  );
});

console.log("✅ Server running on http://127.0.0.1:8080/");
console.log("  Try: curl http://127.0.0.1:8080/");
console.log("  Try: curl http://127.0.0.1:8080/about");
console.log("  Try: curl http://127.0.0.1:8080/users/123");
