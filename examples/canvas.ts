const canvas = createCanvas(800, 600);

console.log(canvas.getWidth()); // 800
console.log(canvas.getHeight()); // 600
const ctx = canvas.getContext("2d");

ctx?.fillRect(0, 0, 800, 600);
ctx?.clearRect(0, 0, 800, 600);
ctx?.arc(400, 300, 100, 0, Math.PI * 2);