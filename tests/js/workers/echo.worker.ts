// Echoes whatever the parent posts.
self.onmessage = (event: MessageEvent) => {
  self.postMessage(event.data);
};
