// Echo worker for the messaging example.
self.onmessage = (event: MessageEvent) => {
  self.postMessage(`worker received: ${JSON.stringify(event.data)}`);
};
