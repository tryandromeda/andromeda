console.log("🚀 API Server starting...\n");

Andromeda.serve((req) => {
  const url = new URL(req.url);
  const path = url.pathname;
  const method = req.method;

  console.log(`${method} ${path}`);

  if (path === "/" && method === "GET") {
    return new Response(
      JSON.stringify({
        message: "Welcome to Andromeda API",
        version: "1.0.0",
        endpoints: [
          "GET /",
          "GET /users",
          "GET /users/:id",
          "POST /users",
          "GET /health",
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

  if (path === "/health" && method === "GET") {
    return new Response(
      JSON.stringify({
        status: "ok",
        timestamp: Date.now(),
      }),
      {
        status: 200,
        headers: {
          "Content-Type": "application/json",
        },
      },
    );
  }

  if (path === "/users" && method === "GET") {
    const users = [
      { id: 1, name: "Alice", email: "alice@example.com" },
      { id: 2, name: "Bob", email: "bob@example.com" },
      { id: 3, name: "Charlie", email: "charlie@example.com" },
    ];

    return new Response(
      JSON.stringify({
        data: users,
        total: users.length,
      }),
      {
        status: 200,
        headers: {
          "Content-Type": "application/json",
        },
      },
    );
  }

  if (path.startsWith("/users/") && method === "GET") {
    const id = parseInt(path.split("/")[2]);
    const users = [
      { id: 1, name: "Alice", email: "alice@example.com" },
      { id: 2, name: "Bob", email: "bob@example.com" },
      { id: 3, name: "Charlie", email: "charlie@example.com" },
    ];

    const user = users.find((u) => u.id === id);

    if (user) {
      return new Response(
        JSON.stringify({
          data: user,
        }),
        {
          status: 200,
          headers: {
            "Content-Type": "application/json",
          },
        },
      );
    } else {
      return new Response(
        JSON.stringify({
          error: "User not found",
        }),
        {
          status: 404,
          headers: {
            "Content-Type": "application/json",
          },
        },
      );
    }
  }

  if (path === "/users" && method === "POST") {
    return new Response(
      JSON.stringify({
        message: "User created",
        data: {
          id: 4,
          name: "New User",
          email: "newuser@example.com",
        },
      }),
      {
        status: 201,
        headers: {
          "Content-Type": "application/json",
        },
      },
    );
  }

  // 404 Not Found
  return new Response(
    JSON.stringify({
      error: "Not Found",
      path: path,
    }),
    {
      status: 404,
      headers: {
        "Content-Type": "application/json",
      },
    },
  );
});

console.log("✅ API Server running on http://127.0.0.1:8080/");
console.log("  curl http://127.0.0.1:8080/");
console.log("  curl http://127.0.0.1:8080/users");
console.log("  curl http://127.0.0.1:8080/users/1");
console.log("  curl http://127.0.0.1:8080/health");
console.log("");
