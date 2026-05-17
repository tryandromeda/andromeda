// This example demonstrates how to use the `parallel` option to run multiple workers in parallel.
const workerName = (globalThis as any).name || "main";

Andromeda.serve(
  (_req: Request) => {
    return new Response(
      JSON.stringify({ handledBy: workerName }) + "\n",
      { headers: { "content-type": "application/json" } },
    );
  },
  {
    port: 8080,
    hostname: "127.0.0.1",
    parallel: 4,
    entry: import.meta.url,
    onListen: ({ hostname, port }) => {
      console.log(`[${workerName}] listening on http://${hostname}:${port}`);
    },
  },
);
