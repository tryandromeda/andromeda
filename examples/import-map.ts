import { basename, dirname, join } from "@std/path";

console.log("   join('a', 'b', 'c'):", join("a", "b", "c"));
console.log("   dirname('/path/to/file.ts'):", dirname("/path/to/file.ts"));
console.log("   basename('/path/to/file.ts'):", basename("/path/to/file.ts"));

console.log(
  "\n✅ Import map successfully resolved @std/path to ./modules/path.ts!",
);
