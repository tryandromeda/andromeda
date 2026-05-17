// Long-running worker; parent will terminate it.
setInterval(() => {
  /* keep alive */
}, 100);
self.onmessage = (_event: MessageEvent) => {
  self.postMessage("ack");
};
