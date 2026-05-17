const db = new Database("todos.sqlite");

db.prepare(`
  CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task TEXT NOT NULL,
    completed INTEGER DEFAULT 0
  )
`).run();


const indexHtml = await Andromeda.readTextFile("examples/todo-app/index.html");

Andromeda.serve(async (req) => {
  const url = new URL(req.url);
  const path = url.pathname;
  const method = req.method;

  if (path === "/" && method === "GET") {
    return new Response(indexHtml, {
      headers: { "Content-Type": "text/html" },
    });
  }

  // Get all Todos
  if (path === "/todos" && method === "GET") {
    try {
      const todos = db.prepare("SELECT * FROM todos ORDER BY id DESC").all();
      return new Response(JSON.stringify(todos), {
        headers: { "Content-Type": "application/json" },
      });
      // deno-lint-ignore no-explicit-any
    } catch (e: any) {
      return new Response(JSON.stringify({ error: e.message }), {
        status: 500,
      });
    }
  }

  // Create Todo
  if (path === "/todos" && method === "POST") {
    try {
      const text = await req.text();
      const body = JSON.parse(text);

      if (!body.task) {
        return new Response(JSON.stringify({ error: "Task is required" }), {
          status: 400,
        });
      }

      const stmt = db.prepare(
        "INSERT INTO todos (task, completed) VALUES (?, ?)",
      );
      stmt.run(body.task, 0);

      return new Response(JSON.stringify({ success: true }), {
        status: 201,
        headers: { "Content-Type": "application/json" },
      });
    } catch {
      return new Response(JSON.stringify({ error: "Invalid JSON" }), {
        status: 400,
      });
    }
  }

  // Update Todo
  if (path.startsWith("/todos/") && method === "PUT") {
    try {
      const id = path.split("/")[2];
      const text = await req.text();
      const body = JSON.parse(text);

      const stmt = db.prepare("UPDATE todos SET completed = ? WHERE id = ?");
      stmt.run(body.completed ? 1 : 0, parseInt(id));

      return new Response(JSON.stringify({ success: true }), {
        headers: { "Content-Type": "application/json" },
      });
    } catch {
      return new Response(JSON.stringify({ error: "Update failed" }), {
        status: 400,
      });
    }
  }

  // Delete Todo
  if (path.startsWith("/todos/") && method === "DELETE") {
    try {
      const id = path.split("/")[2];
      const stmt = db.prepare("DELETE FROM todos WHERE id = ?");
      stmt.run(parseInt(id));

      return new Response(JSON.stringify({ success: true }), {
        headers: { "Content-Type": "application/json" },
      });
    } catch {
      return new Response(JSON.stringify({ error: "Delete failed" }), {
        status: 400,
      });
    }
  }

  return new Response(JSON.stringify({ error: "Not Found" }), {
    status: 404,
    headers: { "Content-Type": "application/json" },
  });
}, {
  port: 8080,
  hostname: "127.0.0.1",
  parallel: 12,
  entry: import.meta.url,
  onListen: ({ hostname, port }) => {
    // deno-lint-ignore no-explicit-any
    console.log(`[${(globalThis as any).name || "main"}] listening on http://${hostname}:${port}`);
  },
},);
