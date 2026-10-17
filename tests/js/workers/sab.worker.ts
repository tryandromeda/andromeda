// deno-lint-ignore-file no-explicit-any
self.onmessage = (event: MessageEvent) => {
  const msg = event.data as any;
  switch (msg.cmd) {
    case "fill": {
      const ia = new Int32Array(msg.sab);
      for (let i = 0; i < ia.length; i++) ia[i] = (i + 1) * 10;
      self.postMessage({ cmd: "filled" });
      break;
    }
    case "create": {
      const sab = new SharedArrayBuffer(msg.byteLength);
      const ia = new Int32Array(sab);
      ia[0] = 7;
      self.postMessage({ cmd: "created", sab });
      break;
    }
    case "view": {
      const view = msg.view as Int32Array;
      view[0] = 555;
      self.postMessage({
        cmd: "viewdone",
        byteOffset: view.byteOffset,
        length: view.length,
      });
      break;
    }
    case "grow": {
      (msg.sab as SharedArrayBuffer).grow(msg.to);
      self.postMessage({ cmd: "grown" });
      break;
    }
    case "dup": {
      self.postMessage({
        cmd: "dupresult",
        same: msg.a === msg.b,
        instance: msg.a instanceof SharedArrayBuffer,
      });
      break;
    }
    case "zero": {
      self.postMessage({
        cmd: "zeroresult",
        byteLength: (msg.sab as SharedArrayBuffer).byteLength,
      });
      break;
    }
    case "wait": {
      const ia = new Int32Array(msg.sab);
      self.postMessage({ cmd: "waiting" });
      const result = Atomics.wait(ia, 0, 0, 5000);
      self.postMessage({ cmd: "woken", result, value: Atomics.load(ia, 0) });
      break;
    }
    case "messageerror": {
      (globalThis as any).__andromeda__.op_worker_post_messageerror_to_parent(
        "deliberate messageerror",
      );
      break;
    }
    case "corrupt": {
      (globalThis as any).__andromeda__.op_worker_post_to_parent(
        JSON.stringify({
          root: { type: "SharedArrayBuffer", id: 0, sharedIndex: 0 },
          transferList: 0,
        }),
      );
      break;
    }
    default:
      throw new Error(`unknown command: ${msg && msg.cmd}`);
  }
};
