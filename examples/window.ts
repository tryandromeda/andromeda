const WIDTH = 640;
const HEIGHT = 480;

const win = Andromeda.createWindow({
  title: "Andromeda Canvas Window",
  width: WIDTH,
  height: HEIGHT,
});
console.log("opened window", win.rid, win.rawHandle().system);

const canvas = new OffscreenCanvas(WIDTH, HEIGHT);
const ctx = canvas.getContext("2d")!;

win.addEventListener("resize", (e: any) => {
  console.log(
    `resize → ${e.detail.width}x${e.detail.height} @${e.detail.scaleFactor}x`,
  );
});

win.addEventListener("keydown", (e: any) => {
  if (e.detail.code === "Escape") {
    console.log("escape pressed, closing");
    win.close();
  }
});

win.addEventListener("close", () => {
  console.log("window close requested");
});

let frame = 0;
await Andromeda.Window.mainloop(() => {
  frame++;
  const t = frame / 60;

  ctx.fillStyle = `rgb(${(20 + 20 * Math.sin(t)) | 0}, ${(20 + 20 * Math.sin(t + 1)) | 0}, 40)`;
  ctx.fillRect(0, 0, WIDTH, HEIGHT);

  for (let i = 0; i < 3; i++) {
    const cx = WIDTH * (0.25 + i * 0.25);
    const cy = HEIGHT * 0.5 + Math.sin(t + i) * 60;
    const size = 80 + Math.sin(t * 2 + i) * 20;
    const r = 128 + Math.sin(t + i * 2) * 127;
    const g = 128 + Math.sin(t + i * 2 + 2) * 127;
    const b = 128 + Math.sin(t + i * 2 + 4) * 127;
    ctx.fillStyle = `rgb(${r | 0}, ${g | 0}, ${b | 0})`;
    ctx.fillRect(cx - size / 2, cy - size / 2, size, size);
  }

  try {
    win.presentCanvas(canvas);
  } catch (e) {
    console.log("present err:", String(e));
    win.close();
    return;
  }

  if (frame === 1 || frame % 60 === 0) {
    console.log(`tick ${frame}`);
  }
});

console.log("mainloop exited cleanly");
