// MessageChannel: entangled ports deliver messages to each other.
const channel = new MessageChannel();

let port1Received = 0;
let port2Received = 0;
let phase = 0;

channel.port1.onmessage = (event: MessageEvent) => {
  port1Received += 1;
  if (event.data !== "to port1") {
    throw new Error(`port1 expected "to port1", got ${event.data}`);
  }
  checkPhase();
};

channel.port2.onmessage = (event: MessageEvent) => {
  port2Received += 1;
  if (event.data !== "to port2") {
    throw new Error(`port2 expected "to port2", got ${event.data}`);
  }
  checkPhase();
};

function checkPhase() {
  if (port1Received === 1 && port2Received === 1 && phase === 0) {
    phase = 1;
    // close port1; further posts from port2 should be dropped.
    channel.port1.close();
    channel.port2.postMessage("dropped");
    // Use queueMicrotask to verify nothing arrives.
    queueMicrotask(() => {
      queueMicrotask(() => {
        if (port1Received !== 1) {
          throw new Error(
            `expected port1 to stop receiving after close, got ${port1Received}`,
          );
        }
        console.log("message_channel: ok");
      });
    });
  }
}

channel.port1.postMessage("to port2");
channel.port2.postMessage("to port1");
