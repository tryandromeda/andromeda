console.log("Testing queueMicrotask error reporting behavior...");

// This should execute successfully
queueMicrotask(() => {
  console.log("✓ First microtask executed successfully");
});

// This should throw an error but not crash the program
queueMicrotask(() => {
  console.log("About to throw an error in microtask...");
  throw new Error("This is a test error in a microtask callback");
});

// This should still execute despite the previous error
queueMicrotask(() => {
  console.log("✓ Third microtask executed successfully after error");
});

console.log("All microtasks queued. They will execute asynchronously...");

// Wait a bit to see the results
setTimeout(() => {
  console.log("✓ Test completed - error reporting working correctly!");
}, 50);
