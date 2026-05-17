// terminate() stops the worker and prevents further deliveries.
const worker = new Worker(
  new URL("./terminate.worker.ts", import.meta.url),
  { type: "module" },
);

const start = Date.now();
let received = 0;

worker.onmessage = (event: MessageEvent) => {
  received += 1;
  if (event.data === "ack") {
    worker.terminate();
    // Send another message *after* terminate — it should be silently dropped.
    worker.postMessage("after-terminate");
    setTimeout(() => {
      if (received !== 1) {
        throw new Error(
          `expected exactly 1 delivery, got ${received} (terminate did not stop deliveries)`,
        );
      }
      const elapsed = Date.now() - start;
      if (elapsed > 2000) {
        throw new Error(`terminate took too long: ${elapsed}ms`);
      }
      console.log(`terminate: ok (elapsed ${elapsed}ms)`);
    }, 500);
  }
};

worker.postMessage("ping");
