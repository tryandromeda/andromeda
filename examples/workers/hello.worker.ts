// Worker side: echo back whatever the parent posts.
self.onmessage = (event: MessageEvent) => {
  self.postMessage(`worker received: ${JSON.stringify(event.data)}`);
};
