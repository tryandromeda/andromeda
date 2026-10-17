self.onmessage = (event: MessageEvent) => {
  const sab = event.data as SharedArrayBuffer;
  const view = new Int32Array(sab);

  console.log("worker reads what the parent wrote:", view[0]);

  view[1] = 7;
  view[2] = 8;
  view[3] = 9;

  self.postMessage("done");
};
