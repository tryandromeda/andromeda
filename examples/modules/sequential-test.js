// Simple sequential test to avoid module loading conflicts
console.log("🚀 Testing ES6 modules sequentially...\n");

async function runTests() {
  try {
    // Test 1: Basic math module
    console.log("📦 Test 1: Math module");
    const mathModule = await import("./math.js");
    console.log("   ✅ math.add(2, 3) =", mathModule.add(2, 3));
    console.log("   ✅ math.PI =", mathModule.PI);
    console.log("   ✅ Default export square(4) =", mathModule.default(4));

    // Test 2: Default export module
    console.log("\n🎯 Test 2: Default export");
    const defaultModule = await import("./default-export.js");
    console.log("   ✅ Default greeting:", defaultModule.default("Andromeda"));
    console.log("   ✅ Version:", defaultModule.version);

    // Test 3: TypeScript module
    console.log("\n🔀 Test 3: TypeScript module");
    const tsModule = await import("./mixed-exports.ts");
    console.log("   ✅ API_URL:", tsModule.API_URL);
    console.log("   ✅ fetchData:", tsModule.fetchData("users"));

    // Test 4: JSON module
    console.log("\n📄 Test 4: JSON module");
    const jsonModule = await import("./config.json");
    console.log("   ✅ Package name:", jsonModule.default.name);
    console.log("   ✅ Features:", jsonModule.default.features);

    console.log("\n🎉 All tests completed successfully!");
  } catch (error) {
    console.error("❌ Test failed:", error.message);
  }
}

// Use async/await pattern instead of multiple concurrent imports
runTests();
