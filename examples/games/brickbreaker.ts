// Brick Breaker — paddle + ball + multi-hit bricks across procedurally
// generated levels. Header tracks score, lives, level, best. Ball sticks
// to the paddle on launch and after each death; space launches it. Mouse
// also drives the paddle. Particles fly on brick break, a subtle trail
// follows the ball, and physics is dt-based so it doesn't drift on
// slower frames.

const WIDTH = 720;
const HEIGHT = 540;
const HEADER_H = 56;
const PADDING = 16;
const PLAY_TOP = HEADER_H;
const PLAY_BOTTOM = HEIGHT - 14;

const PADDLE_W = 92;
const PADDLE_H = 12;
const PADDLE_Y = HEIGHT - 50;
const PADDLE_SPEED = 760; // px/sec

const BALL_R = 7;
const BALL_INIT_SPEED = 500; // px/sec
const BALL_SPEEDUP_PER_HIT = 1.018;
const BALL_MAX_SPEED = 1050;
const BALL_TRAIL_LEN = 10;

const BRICK_COLS = 12;
const BRICK_W = 48;
const BRICK_H = 18;
const BRICK_GAP = 4;
const BRICK_OFFSET_Y = HEADER_H + 40;
const BRICK_OFFSET_X = (WIDTH - (BRICK_COLS * (BRICK_W + BRICK_GAP) - BRICK_GAP)) / 2;

const STARTING_LIVES = 3;
const BEST_KEY = "andromeda.brickbreaker.best";

const COLORS = {
  bg: "#0b0f17",
  panel: "#13161d",
  panelBorder: "#1f2532",
  text: "#e7e9ee",
  textMuted: "#8b93a7",
  paddle: "#e7e9ee",
  paddleEdge: "#9ca3af",
  ball: "#fbbf24",
  ballGlow: "rgba(251,191,36,0.35)",
  red: "#ef4444",
  green: "#22c55e",
  particle: ["#f87171", "#fbbf24", "#facc15", "#34d399", "#60a5fa", "#a78bfa"],
};

const ROW_PALETTE = [
  // Each row's [hp1, hp2, hp3] colors so multi-hit bricks darken on hit.
  ["#ef4444", "#b91c1c", "#7f1d1d"],
  ["#f97316", "#c2410c", "#7c2d12"],
  ["#fbbf24", "#b45309", "#78350f"],
  ["#22c55e", "#15803d", "#14532d"],
  ["#3b82f6", "#1d4ed8", "#1e3a8a"],
  ["#a855f7", "#7e22ce", "#581c87"],
  ["#ec4899", "#be185d", "#831843"],
];

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
  rowKind: number; // 0..ROW_PALETTE.length-1
  hp: number; // remaining hits
  maxHp: number;
}
interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  life: number;
  color: string;
}

type Phase = "ready" | "playing" | "lost" | "won";

interface State {
  paddle: Paddle;
  ball: Ball;
  trail: { x: number; y: number }[];
  bricks: Brick[];
  particles: Particle[];
  level: number;
  score: number;
  best: number;
  lives: number;
  phase: Phase;
  ballAttached: boolean; // ball stuck to paddle waiting for launch
  flashTimer: number;
  pointerX: number | null;
  leftDown: boolean;
  rightDown: boolean;
}

function loadBest(): number {
  const n = Number(localStorage.getItem(BEST_KEY));
  return Number.isFinite(n) && n >= 0 ? Math.floor(n) : 0;
}
function saveBest(n: number) {
  localStorage.setItem(BEST_KEY, String(n));
}

function rowsForLevel(level: number): number {
  return Math.min(7, 4 + Math.floor((level - 1) / 2));
}

function buildBricks(level: number): Brick[] {
  const rows = rowsForLevel(level);
  const out: Brick[] = [];
  for (let row = 0; row < rows; row++) {
    // Top rows are tougher: row 0/1 always 1hp; deeper rows scale up with level.
    let hp = 1;
    if (row >= rows - 2) hp = 1;
    else if (row >= rows - 4) hp = level >= 2 ? 2 : 1;
    else hp = level >= 3 ? 3 : level >= 2 ? 2 : 1;
    for (let col = 0; col < BRICK_COLS; col++) {
      out.push({
        x: BRICK_OFFSET_X + col * (BRICK_W + BRICK_GAP),
        y: BRICK_OFFSET_Y + row * (BRICK_H + BRICK_GAP),
        w: BRICK_W,
        h: BRICK_H,
        rowKind: row % ROW_PALETTE.length,
        hp,
        maxHp: hp,
      });
    }
  }
  return out;
}

function fresh(best: number, score = 0, lives = STARTING_LIVES, level = 1): State {
  return {
    paddle: { x: (WIDTH - PADDLE_W) / 2, y: PADDLE_Y, w: PADDLE_W, h: PADDLE_H },
    ball: { x: WIDTH / 2, y: PADDLE_Y - BALL_R - 2, vx: 0, vy: 0, r: BALL_R },
    trail: [],
    bricks: buildBricks(level),
    particles: [],
    level,
    score,
    best,
    lives,
    phase: "ready",
    ballAttached: true,
    flashTimer: 0,
    pointerX: null,
    leftDown: false,
    rightDown: false,
  };
}

function ballIntersectsRect(ball: Ball, rx: number, ry: number, rw: number, rh: number): boolean {
  const cx = Math.max(rx, Math.min(ball.x, rx + rw));
  const cy = Math.max(ry, Math.min(ball.y, ry + rh));
  const dx = ball.x - cx;
  const dy = ball.y - cy;
  return dx * dx + dy * dy <= ball.r * ball.r;
}

function speedUp(ball: Ball, factor: number) {
  ball.vx *= factor;
  ball.vy *= factor;
  const speed = Math.hypot(ball.vx, ball.vy);
  if (speed > BALL_MAX_SPEED) {
    const s = BALL_MAX_SPEED / speed;
    ball.vx *= s;
    ball.vy *= s;
  }
}

function spawnBrickParticles(b: Brick) {
  const cx = b.x + b.w / 2;
  const cy = b.y + b.h / 2;
  for (let i = 0; i < 10; i++) {
    const a = Math.random() * Math.PI * 2;
    const sp = 80 + Math.random() * 140;
    state.particles.push({
      x: cx,
      y: cy,
      vx: Math.cos(a) * sp,
      vy: Math.sin(a) * sp,
      life: 0.4 + Math.random() * 0.3,
      color: COLORS.particle[Math.floor(Math.random() * COLORS.particle.length)],
    });
  }
}

const win = Andromeda.createWindow({ title: "Andromeda Brick Breaker", width: WIDTH, height: HEIGHT });
const canvas = new OffscreenCanvas(WIDTH, HEIGHT);
const ctx = canvas.getContext("2d")!;
let state = fresh(loadBest());

function launchBall() {
  if (!state.ballAttached) return;
  state.ballAttached = false;
  // Slight angle off vertical so the first launch isn't a snooze.
  const a = (Math.random() - 0.5) * 0.6 - Math.PI / 2; // ~upward
  state.ball.vx = Math.cos(a) * BALL_INIT_SPEED;
  state.ball.vy = Math.sin(a) * BALL_INIT_SPEED;
}

win.addEventListener("keydown", (e: any) => {
  const c: string = e.detail.code;
  if (c === "Escape") {
    commitBest();
    return win.close();
  }
  if (c === "KeyR") {
    commitBest();
    state = fresh(state.best);
    return;
  }
  if (state.phase === "lost" || state.phase === "won") {
    if (c === "Space" || c === "Enter") state = fresh(state.best);
    return;
  }
  if (c === "ArrowLeft" || c === "KeyA") state.leftDown = true;
  if (c === "ArrowRight" || c === "KeyD") state.rightDown = true;
  if ((c === "Space" || c === "ArrowUp" || c === "KeyW") && state.phase === "ready") {
    state.phase = "playing";
    launchBall();
  } else if ((c === "Space" || c === "ArrowUp" || c === "KeyW") && state.ballAttached) {
    launchBall();
  }
});
win.addEventListener("keyup", (e: any) => {
  const c: string = e.detail.code;
  if (c === "ArrowLeft" || c === "KeyA") state.leftDown = false;
  if (c === "ArrowRight" || c === "KeyD") state.rightDown = false;
});
win.addEventListener("mousemove", (e: any) => {
  state.pointerX = e.detail.x;
});
win.addEventListener("mousedown", (e: any) => {
  if (e.detail.button !== 0) return;
  if (state.phase === "lost" || state.phase === "won") {
    state = fresh(state.best);
    return;
  }
  if (state.phase === "ready") state.phase = "playing";
  if (state.ballAttached) launchBall();
});

// Fixed-timestep physics. Accumulator collects real elapsed time and
// pays it down in `STEP` chunks; cosmetic things (particles, flashTimer)
// move on the raw frame dt. This is the single biggest smoothness fix:
// motion no longer hiccups when a frame happens to be 24ms instead of
// 16ms because every chunk simulates exactly the same amount of time.
const STEP = 1 / 120;
const MAX_FRAME_DT = 1 / 30;
// How fast the paddle catches up to the mouse target. e^(-rate*dt) of
// remaining distance is left after dt seconds. ~32 -> 99% in 145ms.
const MOUSE_FOLLOW_RATE = 32;

let last = Date.now();
let physAccum = 0;
let bestSaveDirty = false;

function commitBest() {
  if (!bestSaveDirty) return;
  saveBest(state.best);
  bestSaveDirty = false;
}

function update() {
  const now = Date.now();
  const frameDt = Math.min(MAX_FRAME_DT, (now - last) / 1000);
  last = now;

  // Cosmetic timers run on the raw frame dt -- they don't need physics
  // determinism and we don't want them to stop animating during a hitch.
  state.flashTimer = Math.max(0, state.flashTimer - frameDt);
  for (const p of state.particles) {
    p.x += p.vx * frameDt;
    p.y += p.vy * frameDt;
    p.vy += 320 * frameDt;
    p.life -= frameDt;
  }
  state.particles = state.particles.filter((p) => p.life > 0);

  if (state.phase !== "playing") {
    physAccum = 0;
    return;
  }

  physAccum += frameDt;
  while (physAccum >= STEP) {
    physAccum -= STEP;
    physicsStep(STEP);
    if (state.phase !== "playing") {
      physAccum = 0;
      break;
    }
  }
}

function physicsStep(dt: number) {
  // Paddle: keyboard takes precedence; mouse smoothly trails the pointer.
  const dir = (state.rightDown ? 1 : 0) - (state.leftDown ? 1 : 0);
  if (dir !== 0) {
    state.paddle.x += dir * PADDLE_SPEED * dt;
  } else if (state.pointerX !== null) {
    const target = state.pointerX - state.paddle.w / 2;
    const k = 1 - Math.exp(-MOUSE_FOLLOW_RATE * dt);
    state.paddle.x += (target - state.paddle.x) * k;
  }
  state.paddle.x = Math.max(PADDING, Math.min(WIDTH - PADDING - state.paddle.w, state.paddle.x));

  if (state.ballAttached) {
    state.ball.x = state.paddle.x + state.paddle.w / 2;
    state.ball.y = state.paddle.y - state.ball.r - 2;
    return;
  }

  // Trail samples *inside* the physics step, so trail spacing stays
  // consistent regardless of how many steps the frame happens to need.
  state.trail.unshift({ x: state.ball.x, y: state.ball.y });
  if (state.trail.length > BALL_TRAIL_LEN) state.trail.length = BALL_TRAIL_LEN;

  if (!stepBall(dt)) {
    // Ball fell off the bottom.
    state.lives--;
    state.flashTimer = 0.4;
    if (state.score > state.best) {
      state.best = state.score;
      bestSaveDirty = true;
    }
    if (state.lives <= 0) {
      state.phase = "lost";
      commitBest();
    } else {
      state.ballAttached = true;
      state.ball.vx = 0;
      state.ball.vy = 0;
      state.trail.length = 0;
    }
    return;
  }

  if (state.bricks.every((b) => b.hp <= 0)) {
    if (state.score > state.best) {
      state.best = state.score;
      bestSaveDirty = true;
    }
    commitBest();
    state = fresh(state.best, state.score, state.lives, state.level + 1);
  }
}

// Returns false if the ball was lost off the bottom this step (so the outer
// loop can stop substepping and let the lives logic run).
function stepBall(dt: number): boolean {
  const b = state.ball;
  b.x += b.vx * dt;
  b.y += b.vy * dt;

  if (b.x - b.r < PADDING) {
    b.x = PADDING + b.r;
    b.vx = Math.abs(b.vx);
  } else if (b.x + b.r > WIDTH - PADDING) {
    b.x = WIDTH - PADDING - b.r;
    b.vx = -Math.abs(b.vx);
  }
  if (b.y - b.r < PLAY_TOP) {
    b.y = PLAY_TOP + b.r;
    b.vy = Math.abs(b.vy);
  }

  // Paddle bounce (only when descending).
  if (b.vy > 0 && ballIntersectsRect(b, state.paddle.x, state.paddle.y, state.paddle.w, state.paddle.h)) {
    b.y = state.paddle.y - b.r;
    const rel = (b.x - (state.paddle.x + state.paddle.w / 2)) / (state.paddle.w / 2);
    const clamped = Math.max(-1, Math.min(1, rel));
    // Map [-1, 1] to [-75°, +75°] off vertical, like Pong/Arkanoid.
    const a = clamped * (Math.PI / 180) * 75 - Math.PI / 2;
    const speed = Math.hypot(b.vx, b.vy);
    b.vx = Math.cos(a) * speed;
    b.vy = Math.sin(a) * speed;
    speedUp(b, BALL_SPEEDUP_PER_HIT);
  }

  // Bricks. Resolve at most one per substep to keep bounces predictable.
  for (const brick of state.bricks) {
    if (brick.hp <= 0) continue;
    if (!ballIntersectsRect(b, brick.x, brick.y, brick.w, brick.h)) continue;
    // Pick the shallower overlap axis to bounce on.
    const overlapX = Math.min(b.x + b.r - brick.x, brick.x + brick.w - (b.x - b.r));
    const overlapY = Math.min(b.y + b.r - brick.y, brick.y + brick.h - (b.y - b.r));
    if (overlapX < overlapY) b.vx = -b.vx;
    else b.vy = -b.vy;

    brick.hp--;
    state.score += 10 * brick.maxHp;
    speedUp(b, BALL_SPEEDUP_PER_HIT);
    if (brick.hp <= 0) {
      spawnBrickParticles(brick);
      state.score += 5 * (brick.maxHp - 1);
    }
    if (state.score > state.best) {
      state.best = state.score;
      // Defer the localStorage write until level/death; per-hit writes
      // were causing 1-2ms hitches on slower disks.
      bestSaveDirty = true;
    }
    break;
  }

  return b.y - b.r <= HEIGHT;
}

// --- Render ----------------------------------------------------------------

function drawHeader() {
  ctx.fillStyle = COLORS.panel;
  ctx.fillRect(0, 0, WIDTH, HEADER_H);
  ctx.fillStyle = COLORS.panelBorder;
  ctx.fillRect(0, HEADER_H - 1, WIDTH, 1);

  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "600 11px sans-serif";
  ctx.textAlign = "left";
  ctx.textBaseline = "middle";
  ctx.fillText("SCORE", PADDING, 18);
  ctx.fillText("LIVES", 220, 18);
  ctx.fillText("LEVEL", WIDTH / 2 + 30, 18);
  ctx.fillText("BEST", WIDTH - PADDING - 80, 18);

  ctx.fillStyle = COLORS.text;
  ctx.font = "700 22px ui-monospace, monospace";
  ctx.fillText(String(state.score).padStart(5, "0"), PADDING, 38);
  ctx.fillText(String(state.level).padStart(2, "0"), WIDTH / 2 + 30, 38);
  ctx.fillText(String(state.best).padStart(5, "0"), WIDTH - PADDING - 80, 38);

  // Tiny paddle icons for lives.
  for (let i = 0; i < state.lives; i++) {
    ctx.fillStyle = COLORS.paddle;
    ctx.fillRect(220 + i * 18, 32, 14, 4);
  }
}

function drawBricks() {
  for (const b of state.bricks) {
    if (b.hp <= 0) continue;
    const palette = ROW_PALETTE[b.rowKind];
    const hpIndex = Math.max(0, b.maxHp - b.hp); // 0 = full, higher = damaged
    const color = palette[Math.min(palette.length - 1, hpIndex)];
    ctx.fillStyle = color;
    ctx.fillRect(b.x, b.y, b.w, b.h);
    // Top highlight for some depth.
    ctx.fillStyle = "rgba(255,255,255,0.18)";
    ctx.fillRect(b.x, b.y, b.w, 2);
    ctx.fillStyle = "rgba(0,0,0,0.22)";
    ctx.fillRect(b.x, b.y + b.h - 2, b.w, 2);
    // HP indicator pip(s) on multi-hit bricks.
    if (b.maxHp > 1) {
      const remaining = b.hp;
      const pipW = 4;
      const pipGap = 2;
      const totalW = remaining * pipW + (remaining - 1) * pipGap;
      const pipX = b.x + b.w - totalW - 4;
      const pipY = b.y + b.h / 2 - 1;
      for (let i = 0; i < remaining; i++) {
        ctx.fillStyle = "rgba(255,255,255,0.85)";
        ctx.fillRect(pipX + i * (pipW + pipGap), pipY, pipW, 2);
      }
    }
  }
}

function drawPaddle() {
  const p = state.paddle;
  ctx.fillStyle = COLORS.paddle;
  ctx.fillRect(p.x, p.y, p.w, p.h);
  ctx.fillStyle = "rgba(255,255,255,0.45)";
  ctx.fillRect(p.x + 2, p.y + 2, p.w - 4, 2);
  ctx.fillStyle = COLORS.paddleEdge;
  ctx.fillRect(p.x, p.y + p.h - 2, p.w, 2);
}

function drawBall() {
  // Trail.
  for (let i = 0; i < state.trail.length; i++) {
    const t = state.trail[i];
    const alpha = (1 - i / state.trail.length) * 0.45;
    ctx.fillStyle = `rgba(251,191,36,${alpha})`;
    const r = state.ball.r * (1 - i / (state.trail.length * 1.5));
    ctx.beginPath();
    ctx.arc(t.x, t.y, Math.max(1, r), 0, Math.PI * 2);
    ctx.fill();
  }
  // Glow.
  ctx.fillStyle = COLORS.ballGlow;
  ctx.beginPath();
  ctx.arc(state.ball.x, state.ball.y, state.ball.r + 4, 0, Math.PI * 2);
  ctx.fill();
  ctx.fillStyle = COLORS.ball;
  ctx.beginPath();
  ctx.arc(state.ball.x, state.ball.y, state.ball.r, 0, Math.PI * 2);
  ctx.fill();
}

function drawParticles() {
  for (const p of state.particles) {
    const a = Math.min(1, p.life * 2);
    ctx.fillStyle = p.color;
    ctx.globalAlpha = a;
    ctx.fillRect(p.x - 2, p.y - 2, 4, 4);
  }
  ctx.globalAlpha = 1;
}

function drawWalls() {
  ctx.fillStyle = COLORS.panelBorder;
  ctx.fillRect(0, PLAY_TOP - 1, PADDING, PLAY_BOTTOM - PLAY_TOP + 1);
  ctx.fillRect(WIDTH - PADDING, PLAY_TOP - 1, PADDING, PLAY_BOTTOM - PLAY_TOP + 1);
}

function render() {
  ctx.fillStyle = COLORS.bg;
  ctx.fillRect(0, 0, WIDTH, HEIGHT);

  drawWalls();
  drawBricks();
  drawPaddle();
  drawBall();
  drawParticles();
  drawHeader();

  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "11px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "alphabetic";
  ctx.fillText("←/→ or mouse to move · space to launch · R to reset · esc to quit", WIDTH / 2, HEIGHT - 6);

  if (state.phase === "ready") {
    ctx.fillStyle = "rgba(11,15,23,0.55)";
    ctx.fillRect(0, HEADER_H, WIDTH, HEIGHT - HEADER_H - 30);
    ctx.fillStyle = COLORS.text;
    ctx.font = "700 24px sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(`Level ${state.level}`, WIDTH / 2, HEIGHT / 2 - 18);
    ctx.fillStyle = COLORS.textMuted;
    ctx.font = "14px sans-serif";
    ctx.fillText("press space to launch", WIDTH / 2, HEIGHT / 2 + 10);
  }

  if (state.flashTimer > 0) {
    ctx.fillStyle = `rgba(239,68,68,${(state.flashTimer / 0.4) * 0.35})`;
    ctx.fillRect(0, HEADER_H, WIDTH, HEIGHT - HEADER_H);
  }

  if (state.phase === "lost" || state.phase === "won") {
    ctx.fillStyle = "rgba(11,15,23,0.78)";
    ctx.fillRect(0, HEADER_H, WIDTH, HEIGHT - HEADER_H);
    ctx.fillStyle = state.phase === "won" ? COLORS.green : COLORS.red;
    ctx.font = "700 32px sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(state.phase === "won" ? "YOU WIN" : "GAME OVER", WIDTH / 2, HEIGHT / 2 - 16);
    ctx.fillStyle = COLORS.text;
    ctx.font = "14px sans-serif";
    ctx.fillText(`score ${state.score} · best ${state.best}`, WIDTH / 2, HEIGHT / 2 + 14);
    ctx.fillStyle = COLORS.textMuted;
    ctx.fillText("space / click to play again", WIDTH / 2, HEIGHT / 2 + 36);
  }
}

await Andromeda.Window.mainloop(() => {
  update();
  render();
  win.presentCanvas(canvas);
});
