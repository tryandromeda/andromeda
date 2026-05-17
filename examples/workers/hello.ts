// Parent side: spawn a worker, exchange a message, exit when done.
const worker = new Worker(
  new URL("./hello.worker.ts", import.meta.url),
  { type: "module" },
);

worker.onmessage = (event: MessageEvent) => {
  console.log("parent received:", event.data);
  worker.terminate();
};

worker.onerror = (event: ErrorEvent) => {
  console.log("worker error:", event.message);
  worker.terminate();
};

worker.postMessage({ greeting: "hello from parent", value: 42 });
