// Test importing ES modules
import main, { greet, VERSION } from "./test_esm.ts";

console.log(greet("World"));
console.log(`Version: ${VERSION}`);
main();