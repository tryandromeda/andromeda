const W = 900;
const H = 600;
const canvas = new OffscreenCanvas(W, H);
const ctx = canvas.getContext("2d")!;

ctx.fillStyle = "#0b1020";
ctx.fillRect(0, 0, W, H);

const star5 = new Path2D();

const rO = 1;
const rI = 0.4;
for (let i = 0; i < 10; i++) {
  const r = i % 2 === 0 ? rO : rI;
  const a = (i * Math.PI) / 5 - Math.PI / 2;
  const x = r * Math.cos(a);
  const y = r * Math.sin(a);
  if (i === 0) star5.moveTo(x, y);
  else star5.lineTo(x, y);
}
star5.closePath();

const rand = Math.random;

const drawStar = (
  cx: number,
  cy: number,
  size: number,
  rotation: number,
  color: string,
) => {
  const p = new Path2D();
  p.addPath(
    star5,
    new DOMMatrix().translate(cx, cy).rotate(rotation).scale(size, size),
  );
  ctx.fillStyle = color;
  ctx.fill(p);
};

const logoCx = 450;
const logoCy = 300;
const exclusion = 220;

for (let i = 0; i < 180; i++) {
  const x = rand() * W;
  const y = rand() * H;
  const dx = x - logoCx;
  const dy = y - logoCy;
  if (dx * dx + dy * dy < exclusion * exclusion) continue;
  const size = 1 + rand() * 3.5;
  const brightness = 0.3 + rand() * 0.7;
  const color = `rgba(255, 255, 255, ${brightness.toFixed(3)})`;
  drawStar(x, y, size, rand() * 360, color);
}

drawStar(120, 120, 9, 15, "#f8b500");
drawStar(780, 480, 7, -12, "#f59e0b");
drawStar(820, 100, 5, 30, "#fbbf24");
drawStar(90, 500, 6, -25, "#f8b500");

ctx.save();
const scale = 2;
ctx.translate(logoCx - 101.5 * scale, logoCy - 101.5 * scale);
ctx.scale(scale, scale);

ctx.fillStyle = "#252525";
ctx.beginPath();
ctx.arc(102, 101, 100, 0, Math.PI * 2);
ctx.fill();

const yellowStar = new Path2D(
  "M101.7 49.4448L113.433 85.5527H151.399L120.683 107.869L132.416 " +
    "143.977L101.7 121.661L70.9852 143.977L82.7174 107.869L52.0021 " +
    "85.5527H89.9683L101.7 49.4448Z",
);
ctx.fillStyle = "#FFB800";
ctx.fill(yellowStar);

const amberStar = new Path2D(
  "M71.7277 58.8951L102.049 81.7437L133.149 59.9672L120.788 95.8649" +
    "L151.109 118.714L113.149 118.051L100.788 153.949L89.6883 117.641" +
    "L51.7279 116.979L82.828 95.2023Z",
);
ctx.fillStyle = "#FF9900";
ctx.fill(amberStar);

ctx.strokeStyle = "#FF9900";
ctx.lineWidth = 9;
ctx.lineCap = "round";

const ringCircumference = 2 * Math.PI * 84;
const segments = 8;
const visible = ringCircumference / (segments * 2.8);
const gap = ringCircumference / segments - visible;
ctx.setLineDash([visible, gap]);
ctx.beginPath();
ctx.arc(102, 101, 84, 0, Math.PI * 2);
ctx.stroke();
ctx.setLineDash([]);

ctx.restore();

ctx.fillStyle = "#e2e8f0";
ctx.font = "bold 24px sans-serif";
ctx.textAlign = "center";
ctx.fillText("Andromeda", logoCx, H - 60);

ctx.fillStyle = "#94a3b8";
ctx.font = "12px sans-serif";
ctx.fillText("a javascript & typescript runtime", logoCx, H - 36);

const url = canvas.toDataURL("image/png");
const b64 = url.slice(url.indexOf(",") + 1);
const binary = atob(b64);
const bytes = new Uint8Array(binary.length);
for (let i = 0; i < binary.length; i++) {
  bytes[i] = binary.charCodeAt(i);
}
await Andromeda.writeFile("canvas_advanced.demo.png", bytes);
console.log(`Wrote ${bytes.length} bytes to canvas_advanced.demo.png`);
