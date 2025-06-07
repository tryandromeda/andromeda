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
console.log("Buffer has non-zero values:", buffer.some((x) => x !== 0));

// Test digest
console.log("\nTesting crypto.subtle.digest...");
// Use TextEncoder for proper string-to-bytes conversion
const encoder = new TextEncoder();
const testData = encoder.encode("Hello, World!");
crypto.subtle.digest("SHA-256", testData).then((hash) => {
  // NOTE: Current implementation returns hex string instead of ArrayBuffer
  // This is temporary non-W3C compliant behavior that will be fixed
  if (typeof hash === "string") {
    console.log("SHA-256 hash:", hash);
  } else if (hash instanceof ArrayBuffer) {
    // Convert ArrayBuffer to hex string using standard JavaScript
    const hexString = Array.from(new Uint8Array(hash))
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
    console.log("SHA-256 hash:", hexString);
  }
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

// Test standard TextEncoder/TextDecoder functionality
console.log("\nTesting TextEncoder/TextDecoder...");
const testString = "Hello, Crypto World! 🔐";
const encodedBytes = encoder.encode(testString);
console.log("Encoded bytes:", encodedBytes);
const decoder = new TextDecoder();
const decodedString = decoder.decode(encodedBytes);
console.log("Decoded string:", decodedString);
console.log("Round-trip successful:", decodedString === testString);

// Test hex conversion using standard JavaScript
const hexString = Array.from(encodedBytes)
  .map((b) => b.toString(16).padStart(2, "0"))
  .join("");
console.log("Hex representation:", hexString);
// Convert hex back to bytes
const backToBytes = new Uint8Array(
  hexString.match(/.{1,2}/g)?.map((byte) => parseInt(byte, 16)) || [],
);
console.log("Back to bytes:", backToBytes);
console.log(
  "Hex round-trip successful:",
  Array.from(backToBytes).every((byte, i) => byte === encodedBytes[i]),
);

// Test with different data for hashing
const messages = [
  "Short message",
  "A longer message with unicode: 你好 🌍",
  "Message with special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?",
];

console.log("\nTesting digest with various messages...");
for (const message of messages) {
  const messageBytes = encoder.encode(message);
  crypto.subtle.digest("SHA-256", messageBytes).then((hash) => {
    // NOTE: Current implementation returns hex string instead of ArrayBuffer
    let hexHash;
    if (typeof hash === "string") {
      hexHash = hash;
    } else if (hash instanceof ArrayBuffer) {
      hexHash = Array.from(new Uint8Array(hash))
        .map((b) => b.toString(16).padStart(2, "0"))
        .join("");
    } else {
      hexHash = "unknown";
    }
    console.log(`Message: "${message}"`);
    console.log(`SHA-256: ${hexHash}`);
  }).catch((err) => {
    console.error(`Error hashing "${message}":`, err);
  });
}
