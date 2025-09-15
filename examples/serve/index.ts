// Example HTTP server using Andromeda.serve

// Simple server responding with "Hello World"
const server = Andromeda.serve((req) => {
  console.log(`${req.method} ${req.url}`);

  // Route handling example
  const url = new URL(req.url);

  if (url.pathname === "/") {
    return new Response("Hello World from Andromeda!");
  } else if (url.pathname === "/json") {
    return new Response(
      JSON.stringify({ message: "Hello from JSON endpoint" }),
      {
        headers: { "Content-Type": "application/json" },
      },
    );
  } else if (url.pathname === "/about") {
    return new Response("About Andromeda HTTP Server", {
      status: 200,
      headers: { "Content-Type": "text/plain" },
    });
  } else {
    return new Response("Not Found", { status: 404 });
  }
});

console.log("HTTP server is running on http://127.0.0.1:8080");
console.log("Try these endpoints:");
console.log("  - http://127.0.0.1:8080/");
console.log("  - http://127.0.0.1:8080/json");
console.log("  - http://127.0.0.1:8080/about");
console.log("  - http://127.0.0.1:8080/notfound");
