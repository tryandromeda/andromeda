const WIDTH = 640;
const HEIGHT = 480;

const PADDLE_W = 90;
const PADDLE_H = 12;
const PADDLE_Y = HEIGHT - 40;
const PADDLE_SPEED = 7;

const BALL_R = 7;
const BALL_INIT_VX = 4.2;
const BALL_INIT_VY = -4.2;
const BALL_SPEED_UP = 1.03;
const BALL_MAX_SPEED = 9;

const BRICK_COLS = 10;
const BRICK_ROWS = 5;
const BRICK_W = 56;
const BRICK_H = 18;
const BRICK_GAP = 4;
const BRICK_OFFSET_X =
  (WIDTH - (BRICK_COLS * (BRICK_W + BRICK_GAP) - BRICK_GAP)) / 2;
const BRICK_OFFSET_Y = 50;
const ROW_COLORS = ["#ff3355", "#ff9533", "#ffd633", "#33cc66", "#3388ff"];

interface Paddle {
  x: number;
  y: number;
  w: number;
  h: number;
}
interface Ball {
  x: number;
  y: number;
  vx: number;
  vy: number;
  r: number;
}
interface Brick {
  x: number;
  y: number;
  w: number;
  h: number;
  color: string;
  alive: boolean;
}
type Phase = "playing" | "game-over" | "you-win";

interface GameState {
  paddle: Paddle;
  ball: Ball;
  bricks: Brick[];
  score: number;
  phase: Phase;
}

function buildBricks(): Brick[] {
  const out: Brick[] = [];
  for (let row = 0; row < BRICK_ROWS; row++) {
    for (let col = 0; col < BRICK_COLS; col++) {
      out.push({
        x: BRICK_OFFSET_X + col * (BRICK_W + BRICK_GAP),
        y: BRICK_OFFSET_Y + row * (BRICK_H + BRICK_GAP),
        w: BRICK_W,
        h: BRICK_H,
        color: ROW_COLORS[row] ?? "#aaaaaa",
        alive: true,
      });
    }
  }
  return out;
}

function initialState(): GameState {
  return {
    paddle: {
      x: (WIDTH - PADDLE_W) / 2,
      y: PADDLE_Y,
      w: PADDLE_W,
      h: PADDLE_H,
    },
    ball: {
      x: WIDTH / 2,
      y: PADDLE_Y - BALL_R - 2,
      vx: BALL_INIT_VX,
      vy: BALL_INIT_VY,
      r: BALL_R,
    },
    bricks: buildBricks(),
    score: 0,
    phase: "playing",
  };
}

// Closest-point-on-rect distance test — lets ball-vs-brick behave like a
// proper circle/AABB collision (nice corner bounces, no tunneling at modest
// speeds).
function ballIntersectsRect(
  ball: Ball,
  rx: number,
  ry: number,
  rw: number,
  rh: number,
): boolean {
  const cx = Math.max(rx, Math.min(ball.x, rx + rw));
  const cy = Math.max(ry, Math.min(ball.y, ry + rh));
  const dx = ball.x - cx;
  const dy = ball.y - cy;
  return dx * dx + dy * dy <= ball.r * ball.r;
}

function clampSpeed(ball: Ball) {
  const speed = Math.hypot(ball.vx, ball.vy);
  if (speed > BALL_MAX_SPEED) {
    const s = BALL_MAX_SPEED / speed;
    ball.vx *= s;
    ball.vy *= s;
  }
}

const win = Andromeda.createWindow({
  title: "Andromeda Brick Breaker",
  width: WIDTH,
  height: HEIGHT,
});
console.log(`window ${win.rid} (${win.rawHandle().system})`);
const canvas = new OffscreenCanvas(WIDTH, HEIGHT);
const ctx = canvas.getContext("2d")!;

const keys = new Set<string>();
let state: GameState = initialState();

win.addEventListener("keydown", (e: any) => {
  const code: string = e.detail.code;
  if (code === "Escape") {
    win.close();
    return;
  }
  keys.add(code);
  if (code === "Space" && state.phase !== "playing") {
    state = initialState();
  }
});

win.addEventListener("keyup", (e: any) => {
  keys.delete(e.detail.code);
});

win.addEventListener("close", () => {
  console.log("breakout: window close requested");
});

// -----------------------------------------------------------------------
// Update

function update() {
  if (state.phase !== "playing") return;

  // Paddle.
  const left = keys.has("ArrowLeft") || keys.has("KeyA");
  const right = keys.has("ArrowRight") || keys.has("KeyD");
  if (left) state.paddle.x -= PADDLE_SPEED;
  if (right) state.paddle.x += PADDLE_SPEED;
  if (state.paddle.x < 0) state.paddle.x = 0;
  if (state.paddle.x + state.paddle.w > WIDTH)
    state.paddle.x = WIDTH - state.paddle.w;

  // Ball.
  const b = state.ball;
  b.x += b.vx;
  b.y += b.vy;

  // Walls.
  if (b.x - b.r < 0) {
    b.x = b.r;
    b.vx = Math.abs(b.vx);
  } else if (b.x + b.r > WIDTH) {
    b.x = WIDTH - b.r;
    b.vx = -Math.abs(b.vx);
  }
  if (b.y - b.r < 0) {
    b.y = b.r;
    b.vy = Math.abs(b.vy);
  }

  // Paddle collision — only when the ball is moving down, so a jittery
  // paddle on the underside doesn't re-trap the ball.
  if (
    b.vy > 0 &&
    ballIntersectsRect(
      b,
      state.paddle.x,
      state.paddle.y,
      state.paddle.w,
      state.paddle.h,
    )
  ) {
    b.y = state.paddle.y - b.r;
    b.vy = -Math.abs(b.vy);
    const rawHit =
      (b.x - (state.paddle.x + state.paddle.w / 2)) / (state.paddle.w / 2);
    const hit = Math.max(-1, Math.min(1, rawHit));
    const speed = Math.hypot(b.vx, b.vy);
    b.vx = hit * speed * 0.85;
    b.vy = -Math.sqrt(Math.max(0.001, speed * speed - b.vx * b.vx));
  }

  for (const brick of state.bricks) {
    if (!brick.alive) continue;
    if (!ballIntersectsRect(b, brick.x, brick.y, brick.w, brick.h)) continue;
    brick.alive = false;
    state.score += 10;
    const overlapX = Math.min(
      Math.abs(b.x + b.r - brick.x),
      Math.abs(brick.x + brick.w - (b.x - b.r)),
    );
    const overlapY = Math.min(
      Math.abs(b.y + b.r - brick.y),
      Math.abs(brick.y + brick.h - (b.y - b.r)),
    );
    if (overlapX < overlapY) {
      b.vx = -b.vx;
    } else {
      b.vy = -b.vy;
    }
    b.vx *= BALL_SPEED_UP;
    b.vy *= BALL_SPEED_UP;
    clampSpeed(b);
    break;
  }

  if (state.bricks.every((br) => !br.alive)) {
    state.phase = "you-win";
  } else if (b.y - b.r > HEIGHT) {
    state.phase = "game-over";
  }
}

function render() {
  ctx.fillStyle = "#111827";
  ctx.fillRect(0, 0, WIDTH, HEIGHT);

  for (const brick of state.bricks) {
    if (!brick.alive) continue;
    ctx.fillStyle = brick.color;
    ctx.fillRect(brick.x, brick.y, brick.w, brick.h);
  }

  ctx.fillStyle = "#f3f4f6";
  ctx.fillRect(state.paddle.x, state.paddle.y, state.paddle.w, state.paddle.h);

  ctx.fillStyle = "#fbbf24";
  ctx.beginPath();
  ctx.arc(state.ball.x, state.ball.y, state.ball.r, 0, Math.PI * 2);
  ctx.fill();

  ctx.fillStyle = "#e5e7eb";
  ctx.font = "18px sans-serif";
  ctx.textAlign = "left";
  ctx.fillText(`Score: ${state.score}`, 12, 28);

  if (state.phase !== "playing") {
    ctx.fillStyle = "rgba(0, 0, 0, 0.7)";
    ctx.fillRect(0, HEIGHT / 2 - 60, WIDTH, 120);
    ctx.fillStyle = "#ffffff";
    ctx.font = "36px sans-serif";
    ctx.textAlign = "center";
    const heading = state.phase === "you-win" ? "You Win!" : "Game Over";
    ctx.fillText(heading, WIDTH / 2, HEIGHT / 2 - 10);
    ctx.font = "18px sans-serif";
    ctx.fillText("press Space to restart", WIDTH / 2, HEIGHT / 2 + 28);
  }
}

let frameCount = 0;
const fpsStart = Date.now();
await Andromeda.Window.mainloop(() => {
  frameCount++;
  update();
  render();
  win.presentCanvas(canvas);
  if (frameCount % 120 === 0) {
    const elapsed = (Date.now() - fpsStart) / 1000;
    console.log(
      `breakout: frame ${frameCount} — avg fps ${(frameCount / elapsed).toFixed(1)}`,
    );
  }
});
