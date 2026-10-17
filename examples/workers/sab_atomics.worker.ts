self.onmessage = (event: MessageEvent) => {
  const flag = new Int32Array(event.data as SharedArrayBuffer);
  self.postMessage({ state: "waiting" });
  const result = Atomics.wait(flag, 0, 0, 5000);

  console.log("worker Atomics.wait returned:", result);

  Atomics.store(flag, 1, Atomics.load(flag, 0) * 21);
  self.postMessage({ state: "woken" });
};
