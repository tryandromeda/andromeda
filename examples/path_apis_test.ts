// Test script for new Canvas path APIs

const canvas = new OffscreenCanvas(400, 400);
const ctx = canvas.getContext("2d");

if (!ctx) {
    throw new Error("Failed to get 2D context");
}

// Test quadraticCurveTo
console.log("Testing quadraticCurveTo...");
ctx.beginPath();
ctx.moveTo(50, 50);
ctx.quadraticCurveTo(100, 20, 150, 50);
ctx.stroke();

// Test ellipse
console.log("Testing ellipse...");
ctx.beginPath();
ctx.ellipse(200, 100, 50, 30, Math.PI / 4, 0, 2 * Math.PI);
ctx.stroke();

// Test roundRect
console.log("Testing roundRect...");
ctx.beginPath();
ctx.roundRect(50, 150, 100, 80, 10);
ctx.stroke();

// Test existing APIs still work
console.log("Testing existing APIs...");
ctx.beginPath();
ctx.moveTo(200, 200);
ctx.lineTo(300, 200);
ctx.lineTo(300, 300);
ctx.closePath();
ctx.fill();

console.log("All path API tests completed!");

canvas.saveAsPng("path_apis.demo.png");
