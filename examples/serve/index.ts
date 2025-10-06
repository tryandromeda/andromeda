// Comprehensive test suite for Andromeda.serve
// Tests all example patterns in one server

console.log("🚀 Andromeda.serve Test Suite\n");

// In-memory data stores for testing
const users = [
  { id: 1, name: "Alice", email: "alice@example.com" },
  { id: 2, name: "Bob", email: "bob@example.com" },
  { id: 3, name: "Charlie", email: "charlie@example.com" },
];

let todos = [
  { id: 1, title: "Learn Andromeda", completed: false },
  { id: 2, title: "Build an app", completed: false },
];
let nextTodoId = 3;

Andromeda.serve(async (req) => {
  const url = new URL(req.url);
  const path = url.pathname;
  const method = req.method;

  console.log(`${method} ${path}${url.search}`);

  // ===== Basic Examples =====

  // Basic: Hello World
  if (path === "/" && method === "GET") {
    return new Response(
      JSON.stringify({
        message: "Andromeda.serve Test Suite",
        version: "1.0.0",
        examples: {
          basic: [
            "GET / - This page",
            "GET /hello - Hello World",
            "GET /user-agent - Show user agent",
          ],
          headers: [
            "GET /headers - Show all request headers",
            "GET /cors - CORS example",
          ],
          query: ["GET /search?q=hello&limit=5 - Query parameters"],
          json: ['POST /greet - JSON request body (send {"name":"Alice"})'],
          users: [
            "GET /users - List all users",
            "GET /users/:id - Get user by ID",
            "POST /users - Create user",
          ],
          todos: [
            "GET /todos - List todos",
            'POST /todos - Create todo (send {"title":"..."})',
            "PUT /todos/:id - Update todo",
            "DELETE /todos/:id - Delete todo",
          ],
          status: ["GET /health - Health check", "GET /about - About info"],
        },
      }),
      {
        status: 200,
        headers: { "Content-Type": "application/json" },
      },
    );
  }

  if (path === "/hello" && method === "GET") {
    return new Response("Hello World!");
  }

  // ===== User-Agent Example =====

  if (path === "/user-agent" && method === "GET") {
    const userAgent = req.headers.get("user-agent") ?? "Unknown";
    return new Response(
      JSON.stringify({
        userAgent: userAgent,
        message: `Hello! Your user agent is: ${userAgent}`,
      }),
      {
        status: 200,
        headers: { "Content-Type": "application/json" },
      },
    );
  }

  // ===== Headers Examples =====

  if (path === "/headers" && method === "GET") {
    const requestHeaders: Record<string, string> = {};
    req.headers.forEach((value, key) => {
      requestHeaders[key] = value;
    });
    cosnole.log("requestHeaders", requestHeaders);

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

  if (path === "/cors" && method === "GET") {
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

  // ===== Query Parameters Example =====

  if (path === "/search" && method === "GET") {
    const searchParams = url.searchParams;
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
          headers: { "Content-Type": "application/json" },
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
        headers: { "Content-Type": "application/json" },
      },
    );
  }

  // ===== JSON Request Example =====

  if (path === "/greet" && method === "POST") {
    try {
      const body = await req.text();
      const { name } = JSON.parse(body);

      if (!name) {
        return new Response(
          JSON.stringify({
            error: "Missing 'name' in request body",
          }),
          {
            status: 400,
            headers: { "Content-Type": "application/json" },
          },
        );
      }

      return new Response(
        JSON.stringify({
          message: `Hello ${name}!`,
          timestamp: Date.now(),
        }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" },
        },
      );
    } catch (error) {
      return new Response(
        JSON.stringify({
          error: "Invalid JSON",
        }),
        {
          status: 400,
          headers: { "Content-Type": "application/json" },
        },
      );
    }
  }

  // ===== Users API (REST) =====

  if (path === "/users" && method === "GET") {
    return new Response(
      JSON.stringify({
        data: users,
        total: users.length,
      }),
      {
        status: 200,
        headers: { "Content-Type": "application/json" },
      },
    );
  }

  if (path.startsWith("/users/") && method === "GET") {
    const id = parseInt(path.split("/")[2]);
    const user = users.find((u) => u.id === id);

    if (user) {
      return new Response(
        JSON.stringify({
          data: user,
        }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" },
        },
      );
    } else {
      return new Response(
        JSON.stringify({
          error: "User not found",
        }),
        {
          status: 404,
          headers: { "Content-Type": "application/json" },
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
        headers: { "Content-Type": "application/json" },
      },
    );
  }

  // ===== Todos API (Full CRUD) =====

  if (path === "/todos" && method === "GET") {
    return new Response(
      JSON.stringify({
        data: todos,
        total: todos.length,
      }),
      {
        status: 200,
        headers: { "Content-Type": "application/json" },
      },
    );
  }

  if (path === "/todos" && method === "POST") {
    try {
      const body = await req.text();
      const { title } = JSON.parse(body);

      if (!title) {
        return new Response(
          JSON.stringify({
            error: "Missing 'title' in request body",
          }),
          {
            status: 400,
            headers: { "Content-Type": "application/json" },
          },
        );
      }

      const newTodo = {
        id: nextTodoId++,
        title: title,
        completed: false,
      };
      todos.push(newTodo);

      return new Response(
        JSON.stringify({
          message: "Todo created",
          data: newTodo,
        }),
        {
          status: 201,
          headers: { "Content-Type": "application/json" },
        },
      );
    } catch (error) {
      return new Response(
        JSON.stringify({
          error: "Invalid JSON",
        }),
        {
          status: 400,
          headers: { "Content-Type": "application/json" },
        },
      );
    }
  }

  if (path.startsWith("/todos/") && method === "PUT") {
    const id = parseInt(path.split("/")[2]);
    const todoIndex = todos.findIndex((t) => t.id === id);

    if (todoIndex === -1) {
      return new Response(
        JSON.stringify({
          error: "Todo not found",
        }),
        {
          status: 404,
          headers: { "Content-Type": "application/json" },
        },
      );
    }

    try {
      const body = await req.text();
      const { title, completed } = JSON.parse(body);

      if (title !== undefined) {
        todos[todoIndex].title = title;
      }
      if (completed !== undefined) {
        todos[todoIndex].completed = completed;
      }

      return new Response(
        JSON.stringify({
          message: "Todo updated",
          data: todos[todoIndex],
        }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" },
        },
      );
    } catch (error) {
      return new Response(
        JSON.stringify({
          error: "Invalid JSON",
        }),
        {
          status: 400,
          headers: { "Content-Type": "application/json" },
        },
      );
    }
  }

  if (path.startsWith("/todos/") && method === "DELETE") {
    const id = parseInt(path.split("/")[2]);
    const todoIndex = todos.findIndex((t) => t.id === id);

    if (todoIndex === -1) {
      return new Response(
        JSON.stringify({
          error: "Todo not found",
        }),
        {
          status: 404,
          headers: { "Content-Type": "application/json" },
        },
      );
    }

    const deletedTodo = todos.splice(todoIndex, 1)[0];

    return new Response(
      JSON.stringify({
        message: "Todo deleted",
        data: deletedTodo,
      }),
      {
        status: 200,
        headers: { "Content-Type": "application/json" },
      },
    );
  }

  // ===== Status Endpoints =====

  if (path === "/health" && method === "GET") {
    return new Response(
      JSON.stringify({
        status: "ok",
        timestamp: Date.now(),
        uptime: Date.now(),
      }),
      {
        status: 200,
        headers: { "Content-Type": "application/json" },
      },
    );
  }

  if (path === "/about" && method === "GET") {
    return new Response(
      JSON.stringify({
        name: "Andromeda",
        version: "1.0.0",
        description: "A JavaScript runtime with HTTP server",
      }),
      {
        status: 200,
        headers: { "Content-Type": "application/json" },
      },
    );
  }

  // ===== 404 Not Found =====

  return new Response(
    JSON.stringify({
      error: "Not Found",
      path: path,
      method: method,
      message: "Try GET / for available endpoints",
    }),
    {
      status: 404,
      headers: { "Content-Type": "application/json" },
    },
  );
});

console.log("✅ Server running on http://127.0.0.1:8080/");
console.log("\n📋 Test Commands:");
console.log("  # Basic");
console.log("  curl http://127.0.0.1:8080/");
console.log("  curl http://127.0.0.1:8080/hello");
console.log("  curl http://127.0.0.1:8080/user-agent");
console.log("\n  # Headers");
console.log("  curl http://127.0.0.1:8080/headers");
console.log("  curl -v http://127.0.0.1:8080/cors");
console.log("\n  # Query Parameters");
console.log("  curl 'http://127.0.0.1:8080/search?q=hello&limit=5'");
console.log("\n  # JSON Request");
console.log(
  '  curl -X POST http://127.0.0.1:8080/greet -d \'{"name":"Alice"}\'',
);
console.log("\n  # Users API");
console.log("  curl http://127.0.0.1:8080/users");
console.log("  curl http://127.0.0.1:8080/users/1");
console.log("  curl -X POST http://127.0.0.1:8080/users");
console.log("\n  # Todos API");
console.log("  curl http://127.0.0.1:8080/todos");
console.log(
  '  curl -X POST http://127.0.0.1:8080/todos -d \'{"title":"Buy milk"}\'',
);
console.log(
  "  curl -X PUT http://127.0.0.1:8080/todos/1 -d '{\"completed\":true}'",
);
console.log("  curl -X DELETE http://127.0.0.1:8080/todos/1");
console.log("\n  # Status");
console.log("  curl http://127.0.0.1:8080/health");
console.log("  curl http://127.0.0.1:8080/about");
console.log("");
