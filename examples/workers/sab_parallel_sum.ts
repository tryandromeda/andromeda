
const N = 1_000_000;
const WORKERS = 4;

const data = new Int32Array(new SharedArrayBuffer(N * 4));
for (let i = 0; i < N; i++) data[i] = i % 10;

// One result slot per worker.
const results = new Int32Array(new SharedArrayBuffer(WORKERS * 4));

let expected = 0;
for (let i = 0; i < N; i++) expected += i % 10;
console.log(`summing ${N} integers across ${WORKERS} workers…`);

const chunk = Math.ceil(N / WORKERS);
let done = 0;

for (let w = 0; w < WORKERS; w++) {
  const worker = new Worker(
    new URL("./sab_parallel_sum.worker.ts", import.meta.url),
    { type: "module" },
  );
  worker.onmessage = () => {
    worker.terminate();
    done += 1;
    if (done === WORKERS) {
      let total = 0;
      for (let i = 0; i < WORKERS; i++) {
        console.log(`worker ${i} partial sum: ${Atomics.load(results, i)}`);
        total += Atomics.load(results, i);
      }
      console.log(`total: ${total} (expected ${expected})`, total === expected ? "✓" : "✗");
    }
  };
  worker.postMessage({
    data: data.buffer,
    results: results.buffer,
    index: w,
    start: w * chunk,
    end: Math.min((w + 1) * chunk, N),
  });
}
