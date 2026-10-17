const sab = new SharedArrayBuffer(16);
const view = new Int32Array(sab);

const worker = new Worker(
  new URL("./sab_shared_memory.worker.ts", import.meta.url),
  { type: "module" },
);

view[0] = 42;

worker.onmessage = () => {
  console.log("parent reads what the worker wrote:", view[1], view[2], view[3]);

  view[0] = 1000;
  console.log("shared memory works: no bytes were copied!");
  worker.terminate();
};

console.log("parent wrote 42 into slot 0, posting the buffer to the worker…");
worker.postMessage(sab);
