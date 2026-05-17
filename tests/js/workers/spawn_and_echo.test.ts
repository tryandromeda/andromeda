// Verify primitive and complex-object round-trip through a worker.
const worker = new Worker(
  new URL("./echo.worker.ts", import.meta.url),
  { type: "module" },
);

const tests: { name: string; value: unknown; check: (got: unknown) => boolean }[] = [
  { name: "number", value: 42, check: (g) => g === 42 },
  { name: "string", value: "hello", check: (g) => g === "hello" },
  { name: "null", value: null, check: (g) => g === null },
  {
    name: "object",
    value: { a: 1, b: [2, 3], c: { d: "x" } },
    check: (g) =>
      JSON.stringify(g) === JSON.stringify({ a: 1, b: [2, 3], c: { d: "x" } }),
  },
  {
    name: "Date",
    value: new Date(1700000000000),
    check: (g) => g instanceof Date && (g as Date).getTime() === 1700000000000,
  },
  {
    name: "Map",
    value: new Map([["a", 1], ["b", 2]]),
    check: (g) =>
      g instanceof Map && (g as Map<string, number>).get("a") === 1 &&
      (g as Map<string, number>).get("b") === 2,
  },
  {
    name: "Set",
    value: new Set([1, 2, 3]),
    check: (g) =>
      g instanceof Set && (g as Set<number>).has(1) && (g as Set<number>).has(2) &&
      (g as Set<number>).has(3),
  },
];

let index = 0;
let passed = 0;

function next() {
  if (index >= tests.length) {
    worker.terminate();
    if (passed !== tests.length) {
      throw new Error(`worker round-trip failed: ${passed}/${tests.length}`);
    }
    console.log(`spawn_and_echo: ${passed}/${tests.length} passed`);
    return;
  }
  const t = tests[index++];
  worker.onmessage = (event: MessageEvent) => {
    if (t.check(event.data)) {
      passed += 1;
    } else {
      console.log(`FAIL ${t.name}: got`, event.data);
    }
    next();
  };
  worker.postMessage(t.value);
}

worker.onerror = (event: ErrorEvent) => {
  worker.terminate();
  throw new Error(`worker reported error: ${event.message}`);
};

next();
