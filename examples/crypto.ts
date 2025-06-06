const uuid = crypto.randomUUID();
console.log("Generated UUID:", uuid);
console.log("UUID length:", uuid.length);
console.log(
  "UUID has dashes at correct positions:",
  uuid.charAt(8) === "-" && uuid.charAt(13) === "-" &&
    uuid.charAt(18) === "-" && uuid.charAt(23) === "-",
);

// Test getRandomValues
console.log("\nTesting getRandomValues...");
const buffer = new Uint8Array(16);
const result = crypto.getRandomValues(buffer);
console.log("Buffer filled:", buffer);
console.log("Returned same array:", result === buffer);
console.log("Buffer has non-zero values:", buffer.some((x) => x !== 0)); // Test digest
console.log("\nTesting crypto.subtle.digest...");
// Since TextEncoder is not available, we'll create a Uint8Array manually
const testData = new Uint8Array([
  72,
  101,
  108,
  108,
  111,
  44,
  32,
  87,
  111,
  114,
  108,
  100,
  33,
]); // "Hello, World!" as bytes
crypto.subtle.digest("SHA-256", testData).then((hash) => {
  console.log("SHA-256 hash:", hash);
  console.log("Hash type:", typeof hash);
}).catch((err) => {
  console.error("Digest error:", err);
});

// Test generateKey
console.log("\nTesting crypto.subtle.generateKey...");
crypto.subtle.generateKey("AES-GCM", true, ["encrypt", "decrypt"]).then(
  (key) => {
    console.log("Generated key:", key);
    console.log("Key type:", typeof key);
  },
).catch((err) => {
  console.error("GenerateKey error:", err);
});
