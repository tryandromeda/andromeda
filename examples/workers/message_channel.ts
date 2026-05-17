// MessageChannel: two entangled ports in the same realm.
const channel = new MessageChannel();

channel.port1.onmessage = (event: MessageEvent) => {
  console.log("port1 received:", event.data);
};

channel.port2.onmessage = (event: MessageEvent) => {
  console.log("port2 received:", event.data);
  channel.port2.postMessage("ack from port2");
};

channel.port1.postMessage({ hello: "port2" });
channel.port2.postMessage({ hello: "port1" });
