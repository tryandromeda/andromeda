// Example demonstrating advanced Canvas 2D features with GPU acceleration
// Create a canvas
const canvas = new OffscreenCanvas(800, 600);
const ctx = canvas.getContext("2d");

if (!ctx) {
    console.error("Failed to get 2D context");
    throw new Error("Canvas context not available");
}

// Clear the canvas with a background
ctx.fillStyle = "#2c3e50";
ctx.fillRect(0, 0, 800, 600);

// Draw some filled rectangles
ctx.fillStyle = "#e74c3c";
ctx.fillRect(50, 50, 100, 80);

ctx.fillStyle = "#3498db";
ctx.fillRect(200, 50, 100, 80);

// Draw shapes using paths
ctx.beginPath();
ctx.fillStyle = "#2ecc71";
ctx.moveTo(400, 50);
ctx.lineTo(450, 150);
ctx.lineTo(350, 150);
ctx.closePath();
ctx.fill();

// Draw a circle using arc
ctx.beginPath();
ctx.fillStyle = "#f39c12";
ctx.arc(600, 100, 40, 0, 2 * Math.PI);
ctx.fill();

// Draw stroke examples
ctx.lineWidth = 3;
ctx.strokeStyle = "#8e44ad";

// Draw a stroked rectangle path
ctx.beginPath();
ctx.rect(50, 200, 100, 80);
ctx.stroke();

// Draw a stroked path
ctx.beginPath();
ctx.moveTo(200, 200);
ctx.lineTo(250, 240);
ctx.lineTo(300, 200);
ctx.lineTo(300, 280);
ctx.lineTo(200, 280);
ctx.stroke();


// Draw a stroked circle
ctx.beginPath();
ctx.arc(450, 240, 40, 0, 2 * Math.PI);
ctx.stroke();

// Draw Bezier curves
ctx.lineWidth = 2;
ctx.strokeStyle = "#e67e22";
ctx.beginPath();
ctx.moveTo(50, 350);
ctx.bezierCurveTo(150, 300, 250, 400, 350, 350);
ctx.stroke();

// Complex shape with multiple paths
ctx.fillStyle = "#9b59b6";
ctx.beginPath();
ctx.arc(500, 400, 30, 0, Math.PI);
ctx.moveTo(530, 400);
ctx.arc(500, 450, 30, 0, Math.PI);
ctx.fill();

// Create a more complex path with arcs and lines
ctx.strokeStyle = "#34495e";
ctx.lineWidth = 4;
ctx.beginPath();
ctx.moveTo(600, 350);
ctx.arc(650, 375, 25, 0, Math.PI);
ctx.lineTo(700, 400);
ctx.arc(700, 450, 25, Math.PI, 0);
ctx.lineTo(625, 475);
ctx.closePath();
ctx.stroke();

canvas.saveAsPng("advanced_canvas.demo.png");