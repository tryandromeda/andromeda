// deno-lint-ignore-file no-explicit-any
let passed = 0;
let total = 0;

function check(name: string, cond: boolean) {
  total += 1;
  if (cond) {
    passed += 1;
  } else {
    throw new Error(`FAIL ${name}`);
  }
}

{
  const sab = new SharedArrayBuffer(16);
  (sab as any).expando = "x";
  const clone = structuredClone(sab);
  check("basic: instance", clone instanceof SharedArrayBuffer);
  check("basic: new wrapper", clone !== sab);
  check("basic: byteLength", clone.byteLength === 16);
  check("basic: no expando", (clone as any).expando === undefined);

  const a = new Int32Array(sab);
  const b = new Int32Array(clone);
  a[0] = 42;
  check("basic: write through original visible in clone", b[0] === 42);
  b[1] = 7;
  check("basic: write through clone visible in original", a[1] === 7);
}

{
  const sab = new SharedArrayBuffer(16);
  const view = new Uint8Array(sab, 4, 8);
  const out = structuredClone({ a: sab, view });
  check("graph: buffer cloned", out.a instanceof SharedArrayBuffer);
  check("graph: view byteOffset", out.view.byteOffset === 4);
  check("graph: view length", out.view.length === 8);
  check("graph: view.buffer identity", out.view.buffer === out.a);
  const original = new Uint8Array(sab);
  out.view[0] = 99; // writes at byte offset 4
  check("graph: shared through view", original[4] === 99);
}

{
  const sab = new SharedArrayBuffer(16);
  const ta = structuredClone(new Int32Array(sab));
  check("bare view: length", ta.length === 4);
  const orig = new Int32Array(sab);
  ta[2] = 1234;
  check("bare view: shared", orig[2] === 1234);

  const dv = structuredClone(new DataView(sab, 2, 4));
  check("bare DataView: byteOffset", dv.byteOffset === 2);
  check("bare DataView: byteLength", dv.byteLength === 4);
  const bytes = new Uint8Array(sab);
  bytes[2] = 0xAB;
  check("bare DataView: shared", dv.getUint8(0) === 0xAB);
}

{
  const sab = new SharedArrayBuffer(8);
  const out = structuredClone([sab, sab]);
  check("dup: same wrapper", out[0] === out[1]);
  check("dup: new wrapper", out[0] !== sab);
}

{
  const z1 = new SharedArrayBuffer(0);
  const z2 = new SharedArrayBuffer(0);
  const out = structuredClone([z1, z2]);
  check("zero: byteLength", out[0].byteLength === 0);
  check("zero: distinct objects stay distinct", out[0] !== out[1]);
}

{
  const sab = new SharedArrayBuffer(8, { maxByteLength: 32 });
  const clone = structuredClone(sab);
  check("growable: flag", clone.growable === true);
  check("growable: maxByteLength", clone.maxByteLength === 32);
  sab.grow(16);
  check("growable: grow visible through clone", clone.byteLength === 16);
}

{
  const sab = new SharedArrayBuffer(8);
  let threw = false;
  let code = 0;
  try {
    structuredClone(sab, { transfer: [sab as any] });
  } catch (e) {
    threw = e instanceof DOMException && e.name === "DataCloneError";
    code = (e as any).code;
  }
  check("transfer: SAB in transfer list throws DataCloneError", threw);
  check("transfer: DataCloneError code", code === 25);
}

{
  const sab = new SharedArrayBuffer(8);
  const impostor = { buffer: sab, byteOffset: 0, byteLength: 8, extra: 1 };
  const out = structuredClone(impostor);
  check("impostor: stays a plain object", out.extra === 1);
  check("impostor: buffer cloned as SAB", out.buffer instanceof SharedArrayBuffer);
  check("impostor: byteOffset survives", out.byteOffset === 0);
}

{
  const ab = new ArrayBuffer(4);
  new Uint8Array(ab)[0] = 5;
  const sab = new SharedArrayBuffer(4);
  const out = structuredClone({ a: ab, b: sab }, { transfer: [ab] });
  check("mixed: AB arrives", new Uint8Array(out.a)[0] === 5);
  check("mixed: SAB is shared", out.b instanceof SharedArrayBuffer);
  new Uint8Array(out.b)[0] = 9;
  check("mixed: SAB shares block", new Uint8Array(sab)[0] === 9);
}

function messagePortRoundTrip(): Promise<void> {
  return new Promise((resolve, reject) => {
    const { port1, port2 } = new MessageChannel();
    const sab = new SharedArrayBuffer(8);
    port2.onmessage = (event: MessageEvent) => {
      try {
        const got = event.data as SharedArrayBuffer;
        check("port: instance", got instanceof SharedArrayBuffer);
        check("port: new wrapper", got !== sab);
        new Int32Array(got)[0] = 11;
        check("port: shared", new Int32Array(sab)[0] === 11);
        port1.close();
        resolve();
      } catch (e) {
        reject(e);
      }
    };
    port1.postMessage(sab);
  });
}

function broadcastChannelRoundTrip(): Promise<void> {
  return new Promise((resolve, reject) => {
    const rx = new BroadcastChannel("sab-test");
    const tx = new BroadcastChannel("sab-test");
    const sab = new SharedArrayBuffer(8);
    rx.onmessage = (event: MessageEvent) => {
      try {
        const got = event.data as SharedArrayBuffer;
        check("broadcast: instance", got instanceof SharedArrayBuffer);
        check("broadcast: new wrapper", got !== sab);
        new Int32Array(got)[0] = 21;
        check("broadcast: shared", new Int32Array(sab)[0] === 21);
        rx.close();
        tx.close();
        resolve();
      } catch (e) {
        rx.close();
        tx.close();
        reject(e);
      }
    };
    tx.postMessage(sab);
  });
}

messagePortRoundTrip()
  .then(broadcastChannelRoundTrip)
  .then(() => {
    console.log(`structured_clone_sab: ${passed}/${total} passed`);
  })
  .catch((e) => {
    console.log(`FAIL async: ${e}`);
    throw e;
  });
