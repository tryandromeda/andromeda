const canvas = createCanvas(600, 400);
const ctx = canvas.getContext("2d");

if (!ctx) {
    console.error("Failed to get 2D context");
    throw new Error("Canvas context not available");
}

ctx.fillStyle = "#ff0000"; // Red
ctx.fillRect(50, 50, 80, 80);
ctx.fillStyle = "#00ff00"; // Green
ctx.fillRect(140, 50, 80, 80);

ctx.fillStyle = "#0000ff"; // Blue
ctx.fillRect(230, 50, 80, 80);

ctx.fillStyle = "rgb(255, 165, 0)"; // Orange
ctx.fillRect(50, 140, 80, 80);

ctx.fillStyle = "rgb(128, 0, 128)"; // Purple
ctx.fillRect(140, 140, 80, 80);

ctx.fillStyle = "rgb(255, 192, 203)"; // Pink
ctx.fillRect(230, 140, 80, 80);

ctx.fillStyle = "rgba(255, 0, 0, 0.7)"; // Semi-transparent red
ctx.fillRect(50, 230, 80, 80);

ctx.fillStyle = "rgba(0, 255, 0, 0.5)"; // Semi-transparent green
ctx.fillRect(140, 230, 80, 80);

ctx.fillStyle = "rgba(0, 0, 255, 0.3)"; // Semi-transparent blue
ctx.fillRect(230, 230, 80, 80);

ctx.fillStyle = "red";
ctx.fillRect(320, 50, 80, 80);

ctx.fillStyle = "green";
ctx.fillRect(410, 50, 80, 80);

ctx.fillStyle = "blue";
ctx.fillRect(500, 50, 80, 80);

ctx.fillStyle = "yellow";
ctx.fillRect(320, 140, 80, 80);

ctx.fillStyle = "cyan";
ctx.fillRect(410, 140, 80, 80);

ctx.fillStyle = "magenta";
ctx.fillRect(500, 140, 80, 80);

ctx.fillStyle = "black";
ctx.fillRect(320, 230, 80, 80);

ctx.fillStyle = "white";
ctx.fillRect(410, 230, 80, 80);

ctx.fillStyle = "gray";
ctx.fillRect(500, 230, 80, 80);

ctx.fillStyle = "rgba(255, 255, 0, 0.6)"; // Semi-transparent yellow
ctx.fillRect(75, 320, 100, 60);

ctx.fillStyle = "rgba(255, 0, 255, 0.6)"; // Semi-transparent magenta
ctx.fillRect(125, 320, 100, 60);

ctx.fillStyle = "rgba(0, 255, 255, 0.6)"; // Semi-transparent cyan
ctx.fillRect(175, 320, 100, 60);

const saved = canvas.saveAsPng("test.demo.png");
console.log(`Canvas save result: ${saved}`);
