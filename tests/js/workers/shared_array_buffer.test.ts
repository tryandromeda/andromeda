// deno-lint-ignore-file no-explicit-any
const worker = new Worker(
  new URL("./sab.worker.ts", import.meta.url),
  { type: "module" },
);

let passed = 0;
let total = 0;

function check(name: string, cond: boolean) {
  total += 1;
  if (cond) {
    passed += 1;
  } else {
    console.log(`FAIL ${name}`);
  }
}

function send(msg: unknown): Promise<any> {
  return new Promise((resolve) => {
    worker.onmessage = (event: MessageEvent) => resolve(event.data);
    worker.postMessage(msg);
  });
}

async function main() {
  {
    const sab = new SharedArrayBuffer(16);
    const reply = await send({ cmd: "fill", sab });
    check("fill: ack", reply.cmd === "filled");
    const ia = new Int32Array(sab);
    check(
      "fill: writes visible in main",
      ia[0] === 10 && ia[1] === 20 && ia[2] === 30 && ia[3] === 40,
    );
  }

  {
    const reply = await send({ cmd: "create", byteLength: 8 });
    check("create: instance", reply.sab instanceof SharedArrayBuffer);
    const ia = new Int32Array(reply.sab);
    check("create: worker write visible", ia[0] === 7);
    ia[1] = 8;
  }

  {
    const sab = new SharedArrayBuffer(32);
    const view = new Int32Array(sab, 8, 4);
    const reply = await send({ cmd: "view", view });
    check("view: byteOffset", reply.byteOffset === 8);
    check("view: length", reply.length === 4);
    check("view: shared", new Int32Array(sab, 8, 4)[0] === 555);
  }

  {
    const sab = new SharedArrayBuffer(8, { maxByteLength: 64 });
    const reply = await send({ cmd: "grow", sab, to: 32 });
    check("grow: ack", reply.cmd === "grown");
    check("grow: visible in main", sab.byteLength === 32);
    check("grow: still growable", sab.growable === true);
  }

  {
    const sab = new SharedArrayBuffer(8);
    const reply = await send({ cmd: "dup", a: sab, b: sab });
    check("dup: same identity", reply.same === true);
    check("dup: instance", reply.instance === true);
  }

  {
    const sab = new SharedArrayBuffer(0);
    const reply = await send({ cmd: "zero", sab });
    check("zero: byteLength", reply.byteLength === 0);
  }
  {
    const sab = new SharedArrayBuffer(8);
    const ia = new Int32Array(sab);
    const woken = new Promise<any>((resolve) => {
      worker.onmessage = (event: MessageEvent) => {
        const msg = event.data as any;
        if (msg.cmd === "waiting") {
          Atomics.store(ia, 0, 1);
          Atomics.notify(ia, 0);
        } else if (msg.cmd === "woken") {
          resolve(msg);
        }
      };
    });
    worker.postMessage({ cmd: "wait", sab });
    const msg = await woken;
    check(
      "atomics: worker woke",
      msg.result === "ok" || msg.result === "not-equal",
    );
    check("atomics: value", msg.value === 1);
  }

  {
    const me = new Promise<any>((resolve) => {
      worker.onmessageerror = (event: MessageEvent) => resolve(event);
    });
    worker.postMessage({ cmd: "messageerror" });
    const event = await me;
    check("messageerror: data is null", (event as any).data === null);
    worker.onmessageerror = null;
  }

  {
    const me = new Promise<any>((resolve) => {
      worker.onmessageerror = (event: MessageEvent) => resolve(event);
    });
    worker.postMessage({ cmd: "corrupt" });
    const event = await me;
    check("corrupt: messageerror data is null", (event as any).data === null);
    worker.onmessageerror = null;
    const sab = new SharedArrayBuffer(16);
    const reply = await send({ cmd: "fill", sab });
    check("corrupt: runtime continues", reply.cmd === "filled");
    check("corrupt: memory still shares", new Int32Array(sab)[0] === 10);
  }

  {
    const sab = new SharedArrayBuffer(8);
    worker.postMessage({ cmd: "fill", sab });
    worker.terminate();
    const ia = new Int32Array(sab);
    ia[0] = 1;
    check("terminate: original SAB usable", ia[0] === 1);
  }

  if (passed !== total) {
    throw new Error(`shared_array_buffer: ${passed}/${total} passed`);
  }
  console.log(`shared_array_buffer: ${passed}/${total} passed`);
}

worker.onerror = (event: ErrorEvent) => {
  worker.terminate();
  throw new Error(`worker reported error: ${event.message}`);
};

main();
