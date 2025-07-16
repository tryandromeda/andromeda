const canvas = new OffscreenCanvas(500, 500);
const ctx = canvas.getContext("2d");

if (!ctx) {
  console.error("Failed to get 2D context");
  throw new Error("Canvas context not available");
}

const gradient = ctx.createLinearGradient(0, 0, 500, 500)
gradient.addColorStop(0, "red")
gradient.addColorStop(1, "blue")

ctx.fillStyle = gradient
ctx.fillRect(0, 0, 500, 500)

const saved = canvas.saveAsPng("test.demo.png");
console.log(`Canvas save result: ${saved}`);