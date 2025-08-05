const cache = await caches.open("test-cache-simple");
console.log("✓ Opened cache successfully");

const hasCache = await caches.has("test-cache-simple");
console.log("✓ Cache exists:", hasCache);

await cache.add("https://api.example.com/simple");
console.log("✓ Added URL to cache: https://api.example.com/simple");
const matchResult = await cache.match("https://api.example.com/simple");
console.log("✓ Cache match result:", matchResult !== undefined);
console.log("  Match type:", typeof matchResult);
console.log("  Match value:", matchResult);

await cache.add("https://api.example.com/data1");
await cache.add("https://api.example.com/data2");
await cache.add("https://api.example.com/data3");

const match1 = await cache.match("https://api.example.com/data1");
const match2 = await cache.match("https://api.example.com/data2");
const match3 = await cache.match("https://api.example.com/data3");
const nonMatch = await cache.match("https://api.example.com/nonexistent");

console.log("✓ Match data1:", match1 !== undefined);
console.log("✓ Match data2:", match2 !== undefined);
console.log("✓ Match data3:", match3 !== undefined);
console.log("✓ Non-existent match:", nonMatch === undefined);

const keys = await cache.keys();
console.log(
    "✓ Cache keys count:",
    Array.isArray(keys) ? keys.length : "not an array",
);
console.log("  Keys type:", typeof keys);
const deleted = await cache.delete("https://api.example.com/data1");
console.log("✓ Delete result:", deleted);

const deletedMatch = await cache.match("https://api.example.com/data1");
console.log("✓ Deleted URL no longer matches:", deletedMatch === undefined);
await caches.open("another-cache");
const cacheNames = await caches.keys();
console.log("✓ Cache names:", cacheNames);
const deletedCache = await caches.delete("test-cache-simple");
const deletedAnother = await caches.delete("another-cache");
console.log("✓ Deleted test-cache-simple:", deletedCache);
console.log("✓ Deleted another-cache:", deletedAnother);
