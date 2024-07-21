/// <reference path="../src/runtime/runtime.d.ts" />

console.log("Hello, from Andromeda!");

assertEquals(1, 1, "1 should be equal to 1");
assert(1 === 1, "1 should be equal to 1");

// Andromeda.writeTextFileSync("./modified.txt", "Hello, from Andromeda!");

Andromeda.exit(0);