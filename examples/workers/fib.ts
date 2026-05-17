const worker = new Worker(
  new URL("./fib.worker.ts", import.meta.url),
  { type: "module" },
);

worker.onmessage = (event: MessageEvent) => {
  console.log(`fib(${event.data.n}) = ${event.data.result}`);
  worker.terminate();
};

worker.postMessage(20);

// Parent loop continues while worker computes — proves we're really off-thread.
let ticks = 0;
const heartbeat = setInterval(() => {
  ticks += 1;
  console.log(`parent still alive (tick ${ticks})`);
  if (ticks >= 5) clearInterval(heartbeat);
}, 50);
