// JSON request/response example
// Similar to Supabase Edge Functions examples

console.log("🚀 Starting JSON request server...\n");

interface RequestPayload {
  name: string;
}

Andromeda.serve(async (req) => {
  const url = new URL(req.url);
  const method = req.method;

  console.log(`${method} ${url.pathname}`);

  if (method === "POST" && url.pathname === "/greet") {
    try {
      const body = await req.text();
      const { name }: RequestPayload = JSON.parse(body);

      const data = {
        message: `Hello ${name}!`,
        timestamp: Date.now(),
      };

      return new Response(JSON.stringify(data), {
        status: 200,
        headers: {
          "Content-Type": "application/json",
        },
      });
    } catch (error) {
      return new Response(
        JSON.stringify({
          error: "Invalid JSON",
        }),
        {
          status: 400,
          headers: {
            "Content-Type": "application/json",
          },
        },
      );
    }
  }

  return new Response(
    JSON.stringify({
      message:
        'Send POST request to /greet with JSON body: {"name": "your-name"}',
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
console.log(
  '  Try: curl -X POST http://127.0.0.1:8080/greet -d \'{"name":"Alice"}\'',
);
