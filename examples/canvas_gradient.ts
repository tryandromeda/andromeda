const canvas = new OffscreenCanvas(400, 400);
const ctx = canvas.getContext("2d");

if (!ctx) {
  console.error("Failed to get 2D context");
  throw new Error("Canvas context not available");
}

const linearGradient = ctx.createLinearGradient(0, 0, 200, 200)
linearGradient.addColorStop(0, "red")
linearGradient.addColorStop(1, "blue")

ctx.fillStyle = linearGradient
ctx.fillRect(0, 0, 200, 200)

const radialGradient1 = ctx.createRadialGradient(250, 100, 50, 300, 100, 100)
radialGradient1.addColorStop(0, "red")
radialGradient1.addColorStop(1, "blue")

ctx.fillStyle = radialGradient1
ctx.fillRect(200, 0, 200, 200)

const radialGradient2 = ctx.createRadialGradient(100, 300, 50, 100, 300, 100)
radialGradient2.addColorStop(0, "red")
radialGradient2.addColorStop(1, "blue")

ctx.fillStyle = radialGradient2
ctx.fillRect(0, 200, 200, 200)

const conicGradient = ctx.createConicGradient(0, 300, 300)
conicGradient.addColorStop(0, "red")
conicGradient.addColorStop(1, "blue")

ctx.fillStyle = conicGradient
ctx.fillRect(200, 200, 200, 200)

const saved = canvas.saveAsPng("test.demo.png");
console.log(`Canvas save result: ${saved}`);
