const worker = new Worker(
  new URL("./uncaught_error.worker.ts", import.meta.url),
  { type: "module" },
);

worker.onerror = (event: ErrorEvent) => {
  if (!event.message || event.message.indexOf("boom") === -1) {
    worker.terminate();
    throw new Error(
      `expected message to contain "boom", got: ${event.message}`,
    );
  }
  worker.terminate();
  console.log("uncaught_error: ok");
};
