// Test that ES modules work with runtime extensions
import { add, PI } from "./math.ts";

// Use console API provided by runtime
console.log("Testing ES modules with runtime extensions:");
console.log(`Math.PI = ${PI}`);
console.log(`2 + 3 = ${add(2, 3)}`);

// Test timer API provided by runtime
console.time("test");
setTimeout(() => {
  console.timeEnd("test");
  console.log("Timer callback executed");
}, 100);
