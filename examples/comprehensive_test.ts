// Comprehensive test for canvas save/restore and property getters
const canvas = new OffscreenCanvas(400, 300);
const ctx = canvas.getContext("2d");

if (!ctx) {
    console.error("Failed to get 2D context");
    throw new Error("Canvas context not available");
}

console.log("Testing comprehensive save/restore with property getters...");

// Test default values
console.log("=== Default Values ===");
console.log("strokeStyle:", ctx.strokeStyle);
console.log("lineWidth:", ctx.lineWidth);

// Set initial state
console.log("\n=== Setting Initial State ===");
ctx.strokeStyle = "#ff0000";
ctx.lineWidth = 3.0;
ctx.fillStyle = "#00ff00";
ctx.globalAlpha = 0.8;

console.log("strokeStyle:", ctx.strokeStyle);
console.log("lineWidth:", ctx.lineWidth);
console.log("fillStyle:", ctx.fillStyle);
console.log("globalAlpha:", ctx.globalAlpha);

// Save and modify
console.log("\n=== Save and Modify ===");
ctx.save();
ctx.strokeStyle = "blue";
ctx.lineWidth = 7.5;
ctx.fillStyle = "yellow";
ctx.globalAlpha = 0.5;

console.log("After modifying:");
console.log("strokeStyle:", ctx.strokeStyle);
console.log("lineWidth:", ctx.lineWidth);
console.log("fillStyle:", ctx.fillStyle);
console.log("globalAlpha:", ctx.globalAlpha);

// Nested save and modify
console.log("\n=== Nested Save and Modify ===");
ctx.save();
ctx.strokeStyle = "#ff00ff";
ctx.lineWidth = 1.5;

console.log("After nested modification:");
console.log("strokeStyle:", ctx.strokeStyle);
console.log("lineWidth:", ctx.lineWidth);

// First restore
console.log("\n=== First Restore ===");
ctx.restore();
console.log("After first restore:");
console.log("strokeStyle:", ctx.strokeStyle);
console.log("lineWidth:", ctx.lineWidth);
console.log("fillStyle:", ctx.fillStyle);
console.log("globalAlpha:", ctx.globalAlpha);

// Second restore
console.log("\n=== Second Restore ===");
ctx.restore();
console.log("After second restore:");
console.log("strokeStyle:", ctx.strokeStyle);
console.log("lineWidth:", ctx.lineWidth);
console.log("fillStyle:", ctx.fillStyle);
console.log("globalAlpha:", ctx.globalAlpha);

console.log("\n=== Comprehensive test completed successfully! ===");
