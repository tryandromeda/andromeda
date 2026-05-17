// Compute Fibonacci off the main thread.
function fib(n: number): number {
  if (n < 2) return n;
  return fib(n - 1) + fib(n - 2);
}

self.onmessage = (event: MessageEvent) => {
  const n = event.data as number;
  const result = fib(n);
  self.postMessage({ n, result });
};
