const sab = new SharedArrayBuffer(8);
const flag = new Int32Array(sab); // flag[0] = signal slot, flag[1] = answer

const worker = new Worker(
  new URL("./sab_atomics.worker.ts", import.meta.url),
  { type: "module" },
);

worker.onmessage = (event: MessageEvent) => {
  const msg = event.data as { state: string };
  if (msg.state === "waiting") {
    console.log("worker is parked in Atomics.wait — waking it up…");
    Atomics.store(flag, 0, 1);
    Atomics.notify(flag, 0);
  } else if (msg.state === "woken") {
    console.log("worker woke up and computed:", Atomics.load(flag, 1));
    worker.terminate();
  }
};

worker.postMessage(sab);
