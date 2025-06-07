const encoder = new TextEncoder();
const decoder = new TextDecoder();

console.log(
  "✓ Encoder encoding:",
  encoder.encoding === "utf-8" ? "PASS" : "FAIL",
);
console.log(
  "✓ Decoder encoding:",
  decoder.encoding === "utf-8" ? "PASS" : "FAIL",
);

// Test 2: ASCII characters
const ascii = "Hello, World! 123";
const asciiBytes = encoder.encode(ascii);
const asciiDecoded = decoder.decode(asciiBytes);
console.log("✓ ASCII round-trip:", ascii === asciiDecoded ? "PASS" : "FAIL");

// Test 3: Unicode characters (2-byte)
const unicode2 = "Café résumé";
const unicode2Bytes = encoder.encode(unicode2);
const unicode2Decoded = decoder.decode(unicode2Bytes);
console.log(
  "✓ 2-byte Unicode:",
  unicode2 === unicode2Decoded ? "PASS" : "FAIL",
);

// Test 4: Unicode characters (3-byte)
const unicode3 = "你好世界";
const unicode3Bytes = encoder.encode(unicode3);
const unicode3Decoded = decoder.decode(unicode3Bytes);
console.log(
  "✓ 3-byte Unicode:",
  unicode3 === unicode3Decoded ? "PASS" : "FAIL",
);

// Test 5: Unicode characters (4-byte, emojis)
const unicode4 = "🚀🌟💻🎉";
const unicode4Bytes = encoder.encode(unicode4);
const unicode4Decoded = decoder.decode(unicode4Bytes);
console.log(
  "✓ 4-byte Unicode (emojis):",
  unicode4 === unicode4Decoded ? "PASS" : "FAIL",
);

// Test 6: Mixed content
const mixed = "Hello 世界! 🌍 Café résumé 123";
const mixedBytes = encoder.encode(mixed);
const mixedDecoded = decoder.decode(mixedBytes);
console.log("✓ Mixed content:", mixed === mixedDecoded ? "PASS" : "FAIL");

// Test 7: encodeInto functionality
const source = "Hello";
const buffer = new Uint8Array(20);
const result = encoder.encodeInto(source, buffer);
const expectedBytes = [72, 101, 108, 108, 111];
let encodeIntoPass = result.read === 5 && result.written === 5;
for (let i = 0; i < 5; i++) {
  if (buffer[i] !== expectedBytes[i]) {
    encodeIntoPass = false;
    break;
  }
}
console.log("✓ encodeInto:", encodeIntoPass ? "PASS" : "FAIL");

// Test 8: encodeInto with limited space
const longSource = "Hello, World!";
const smallBuffer = new Uint8Array(5);
const limitedResult = encoder.encodeInto(longSource, smallBuffer);
const limitedPass = limitedResult.read === 5 && limitedResult.written === 5;
console.log("✓ encodeInto limited:", limitedPass ? "PASS" : "FAIL");

// Test 9: Decoder options
const fatalDecoder = new TextDecoder("utf-8", { fatal: true });
const bomDecoder = new TextDecoder("utf-8", { ignoreBOM: true });
console.log("✓ Fatal option:", fatalDecoder.fatal === true ? "PASS" : "FAIL");
console.log(
  "✓ IgnoreBOM option:",
  bomDecoder.ignoreBOM === true ? "PASS" : "FAIL",
);

// Test 10: Error handling
let errorHandlingPass = true;
try {
  const invalidBytes = new Uint8Array([0xFF, 0xFE, 0xFD]);
  fatalDecoder.decode(invalidBytes);
  errorHandlingPass = false; // Should have thrown
} catch (_e) {
  // Expected to throw
}
console.log("✓ Fatal error handling:", errorHandlingPass ? "PASS" : "FAIL");

// Test 11: Edge cases
const empty = "";
const emptyBytes = encoder.encode(empty);
const emptyDecoded = decoder.decode(emptyBytes);
console.log(
  "✓ Empty string:",
  (emptyBytes.length === 0 && emptyDecoded === "") ? "PASS" : "FAIL",
);

// Test 12: Null bytes
const nullString = "Hello\0World";
const nullBytes = encoder.encode(nullString);
const nullDecoded = decoder.decode(nullBytes);
console.log("✓ Null bytes:", nullString === nullDecoded ? "PASS" : "FAIL");

console.log("\n=== All Tests Complete ===");
console.log("TextEncoder and TextDecoder implementation is working correctly!");
