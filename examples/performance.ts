function measureFn<T>(fn: () => T, name: string): T {
  performance.mark(`${name}-start`);
  const result = fn();
  performance.mark(`${name}-end`);
  performance.measure(name, `${name}-start`, `${name}-end`);

  const measure = performance.getEntriesByName(name)[0];
  console.log(`${measure.name}: ${measure.duration.toFixed(2)} ms`);

  return result;
}

for (let i = 0; i < 90; i++) {
  console.log(`Iteration ${i}: ${measureFn(() => i * i, `Square of ${i}`)}`);
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function sleeper() {
  await sleep(1000);
  console.log("Slept for 1000ms");
}

sleeper();
