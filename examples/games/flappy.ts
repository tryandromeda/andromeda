const WIDTH = 480;
const HEIGHT = 640;
const HEADER_H = 56;
const GROUND_H = 64;
const PLAY_TOP = HEADER_H;
const PLAY_BOTTOM = HEIGHT - GROUND_H;

const BIRD_X = 130;
const BIRD_R = 14;
const GRAVITY = 1500;
const FLAP_VY = -480;
const MAX_VY = 700;

const PIPE_W = 64;
const PIPE_GAP = 150;
const PIPE_SPEED = 200;
const PIPE_SPACING = 220;

const BEST_KEY = "andromeda.flappy.best";

const COLORS = {
  skyTop: "#4ec0e8",
  skyMid: "#8fd6f0",
  skyLow: "#cfeef7",
  hillFar: "#79c98a",
  hillNear: "#5cb073",
  cloud: "#ffffff",
  ground: "#dec07a",
  groundDark: "#a98a4a",
  grass: "#7ec850",
  grassDark: "#5ea63d",
  pipe: "#5fbf3c",
  pipeLight: "#86d35a",
  pipeDark: "#2f8512",
  pipeStroke: "#1d4f08",
  bird: "#fde047",
  birdBelly: "#fef3a8",
  birdStroke: "#a16207",
  beak: "#fb923c",
  beakStroke: "#c2410c",
  text: "#0b1320",
  textMuted: "rgba(11,19,32,0.6)",
  panel: "rgba(255,255,255,0.9)",
};

type Phase = "ready" | "playing" | "dead";

interface Pipe {
  x: number;
  gapY: number;
  passed: boolean;
}

interface Cloud {
  x: number;
  y: number;
  scale: number;
  speed: number;
}

interface State {
  phase: Phase;
  score: number;
  best: number;
  bird: { y: number; vy: number; angle: number; wingPhase: number };
  pipes: Pipe[];
  clouds: Cloud[];
  spawn: number;
  groundOffset: number;
  hillScroll: number;
  flashGold: number;
}

const HILL_FAR_STEP = 160;
const HILL_FAR_HEIGHT = 70;
const HILL_NEAR_STEP = 110;
const HILL_NEAR_HEIGHT = 44;

function loadBest(): number {
  const n = Number(localStorage.getItem(BEST_KEY));
  return Number.isFinite(n) && n >= 0 ? Math.floor(n) : 0;
}
function saveBest(n: number) {
  localStorage.setItem(BEST_KEY, String(n));
}

function makeClouds(): Cloud[] {
  const out: Cloud[] = [];
  for (let i = 0; i < 5; i++) {
    out.push({
      x: (WIDTH / 5) * i + Math.random() * 40,
      y: HEADER_H + 20 + Math.random() * 140,
      scale: 0.8 + Math.random() * 0.4,
      speed: 12 + Math.random() * 8,
    });
  }
  return out;
}

function fresh(best: number): State {
  return {
    phase: "ready",
    score: 0,
    best,
    bird: { y: HEIGHT / 2, vy: 0, angle: 0, wingPhase: 0 },
    pipes: [],
    clouds: makeClouds(),
    spawn: 0,
    groundOffset: 0,
    hillScroll: 0,
    flashGold: 0,
  };
}

const win = Andromeda.createWindow({
  title: "Andromeda Flap",
  width: WIDTH,
  height: HEIGHT,
});
const canvas = new OffscreenCanvas(WIDTH, HEIGHT);
const ctx = canvas.getContext("2d")!;
let state = fresh(loadBest());
let last = Date.now();

function flap() {
  if (state.phase === "dead") {
    state = fresh(state.best);
    return;
  }
  if (state.phase === "ready") state.phase = "playing";
  state.bird.vy = FLAP_VY;
  state.bird.wingPhase = 0;
}

win.addEventListener("keydown", (e: any) => {
  const c: string = e.detail.code;
  if (c === "Escape") return win.close();
  if (c === "Space" || c === "ArrowUp" || c === "KeyW") {
    if (!e.detail.repeat) flap();
  }
  if (c === "KeyR") state = fresh(state.best);
});
win.addEventListener("mousedown", (e: any) => {
  if (e.detail.button === 0) flap();
});

function spawnPipe() {
  const margin = 60;
  const gapY = margin +
    Math.random() * (PLAY_BOTTOM - PLAY_TOP - PIPE_GAP - margin * 2);
  state.pipes.push({ x: WIDTH + 20, gapY, passed: false });
}

function update() {
  const now = Date.now();
  const dt = Math.min(0.05, (now - last) / 1000);
  last = now;

  state.groundOffset = (state.groundOffset + PIPE_SPEED * dt) % 32;
  state.hillScroll += PIPE_SPEED * dt;
  if (state.flashGold > 0) state.flashGold -= dt;

  for (const c of state.clouds) {
    c.x -= c.speed * dt;
    if (c.x + 80 * c.scale < 0) {
      c.x = WIDTH + 20;
      c.y = HEADER_H + 20 + Math.random() * 140;
    }
  }

  state.bird.wingPhase = (state.bird.wingPhase + dt * 18) % (Math.PI * 2);

  if (state.phase === "ready") {
    state.bird.y = HEIGHT / 2 + Math.sin(now / 250) * 8;
    return;
  }

  if (state.phase === "playing") {
    state.bird.vy = Math.min(MAX_VY, state.bird.vy + GRAVITY * dt);
    state.bird.y += state.bird.vy * dt;
    state.bird.angle = Math.max(-0.5, Math.min(1.2, state.bird.vy / 500));

    state.spawn -= PIPE_SPEED * dt;
    if (
      state.pipes.length === 0 ||
      WIDTH - state.pipes[state.pipes.length - 1].x >= PIPE_SPACING
    ) {
      spawnPipe();
    }

    for (const p of state.pipes) {
      p.x -= PIPE_SPEED * dt;
      if (!p.passed && p.x + PIPE_W < BIRD_X) {
        p.passed = true;
        state.score++;
        if (state.score > state.best) {
          state.best = state.score;
          saveBest(state.best);
          state.flashGold = 0.6;
        }
      }
    }
    state.pipes = state.pipes.filter((p) => p.x + PIPE_W > -10);

    if (state.bird.y + BIRD_R > PLAY_BOTTOM) {
      state.bird.y = PLAY_BOTTOM - BIRD_R;
      state.phase = "dead";
    }
    if (state.bird.y - BIRD_R < PLAY_TOP) {
      state.bird.y = PLAY_TOP + BIRD_R;
      state.bird.vy = 0;
    }

    for (const p of state.pipes) {
      const inX = BIRD_X + BIRD_R > p.x && BIRD_X - BIRD_R < p.x + PIPE_W;
      if (!inX) continue;
      const top = PLAY_TOP + p.gapY;
      const bot = top + PIPE_GAP;
      if (state.bird.y - BIRD_R < top || state.bird.y + BIRD_R > bot) {
        state.phase = "dead";
        break;
      }
    }
  } else if (state.phase === "dead") {
    if (state.bird.y < PLAY_BOTTOM - BIRD_R) {
      state.bird.vy = Math.min(MAX_VY, state.bird.vy + GRAVITY * dt);
      state.bird.y = Math.min(
        PLAY_BOTTOM - BIRD_R,
        state.bird.y + state.bird.vy * dt,
      );
      state.bird.angle = Math.min(1.4, state.bird.angle + 4 * dt);
    }
  }
}

function drawSky() {
  const grad = ctx.createLinearGradient(0, 0, 0, PLAY_BOTTOM);
  grad.addColorStop(0, COLORS.skyTop);
  grad.addColorStop(0.6, COLORS.skyMid);
  grad.addColorStop(1, COLORS.skyLow);
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, WIDTH, PLAY_BOTTOM);
}

function drawHills(
  color: string,
  baseY: number,
  height: number,
  step: number,
  scrollMul: number,
) {
  const offset = ((state.hillScroll * scrollMul) % step + step) % step;
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.moveTo(-step, baseY + 1);
  for (let x = -step - offset; x <= WIDTH + step; x += step) {
    const cx = x + step / 2;
    ctx.quadraticCurveTo(cx, baseY - height * 2, x + step, baseY);
  }
  ctx.lineTo(WIDTH + step, baseY + 1);
  ctx.closePath();
  ctx.fill();
}

function drawHillsBackground() {
  drawHills(
    COLORS.hillFar,
    PLAY_BOTTOM,
    HILL_FAR_HEIGHT,
    HILL_FAR_STEP,
    0.12,
  );
  drawHills(
    COLORS.hillNear,
    PLAY_BOTTOM,
    HILL_NEAR_HEIGHT,
    HILL_NEAR_STEP,
    0.3,
  );
}

function drawCloud(x: number, y: number, scale: number) {
  ctx.save();
  ctx.translate(x, y);
  ctx.scale(scale, scale);
  ctx.fillStyle = COLORS.cloud;
  ctx.beginPath();
  ctx.arc(14, 14, 13, 0, Math.PI * 2);
  ctx.arc(30, 9, 16, 0, Math.PI * 2);
  ctx.arc(48, 14, 13, 0, Math.PI * 2);
  ctx.arc(36, 20, 14, 0, Math.PI * 2);
  ctx.arc(20, 20, 11, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();
}

function drawClouds() {
  for (const c of state.clouds) drawCloud(c.x, c.y, c.scale);
}

function drawGround() {
  // Dirt base
  ctx.fillStyle = COLORS.ground;
  ctx.fillRect(0, PLAY_BOTTOM + 10, WIDTH, GROUND_H - 10);

  // Subtle scrolling stripes
  ctx.fillStyle = COLORS.groundDark;
  for (let x = -state.groundOffset; x < WIDTH; x += 32) {
    ctx.fillRect(x, PLAY_BOTTOM + 30, 16, 3);
  }

  // Grass strip
  ctx.fillStyle = COLORS.grass;
  ctx.fillRect(0, PLAY_BOTTOM, WIDTH, 10);

  // Grass tufts
  ctx.fillStyle = COLORS.grassDark;
  for (let x = -state.groundOffset; x < WIDTH; x += 16) {
    ctx.fillRect(x, PLAY_BOTTOM - 2, 3, 2);
  }
}

function drawPipe(p: Pipe) {
  const top = PLAY_TOP + p.gapY;
  const bot = top + PIPE_GAP;

  drawPipeColumn(p.x, PLAY_TOP, top - PLAY_TOP - 22);
  drawPipeColumn(p.x, bot + 22, PLAY_BOTTOM - (bot + 22));

  drawPipeCap(p.x, top - 24);
  drawPipeCap(p.x, bot);
}

function drawPipeColumn(x: number, y: number, h: number) {
  if (h <= 0) return;
  ctx.fillStyle = COLORS.pipe;
  ctx.fillRect(x, y, PIPE_W, h);
  // Highlight stripe
  ctx.fillStyle = COLORS.pipeLight;
  ctx.fillRect(x + 8, y, 6, h);
  // Dark edge
  ctx.fillStyle = COLORS.pipeDark;
  ctx.fillRect(x + PIPE_W - 6, y, 4, h);
  // Outline
  ctx.strokeStyle = COLORS.pipeStroke;
  ctx.lineWidth = 1.5;
  ctx.strokeRect(x + 0.5, y, PIPE_W - 1, h);
}

function drawPipeCap(x: number, y: number) {
  const cw = PIPE_W + 8;
  const ch = 24;
  const cx = x - 4;
  ctx.fillStyle = COLORS.pipe;
  ctx.fillRect(cx, y, cw, ch);
  ctx.fillStyle = COLORS.pipeLight;
  ctx.fillRect(cx + 6, y + 4, 6, ch - 8);
  ctx.fillStyle = COLORS.pipeDark;
  ctx.fillRect(cx + cw - 6, y, 4, ch);
  ctx.strokeStyle = COLORS.pipeStroke;
  ctx.lineWidth = 1.5;
  ctx.strokeRect(cx + 0.5, y + 0.5, cw - 1, ch - 1);
}

function drawBird() {
  ctx.save();
  ctx.translate(BIRD_X, state.bird.y);
  ctx.rotate(state.bird.angle);

  // Body
  ctx.fillStyle = COLORS.bird;
  ctx.beginPath();
  ctx.arc(0, 0, BIRD_R, 0, Math.PI * 2);
  ctx.fill();

  // Belly highlight
  ctx.fillStyle = COLORS.birdBelly;
  ctx.beginPath();
  ctx.ellipse(-1, 5, BIRD_R - 5, 4, 0, 0, Math.PI * 2);
  ctx.fill();

  // Outline
  ctx.strokeStyle = COLORS.birdStroke;
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  ctx.arc(0, 0, BIRD_R, 0, Math.PI * 2);
  ctx.stroke();

  // Wing — animated
  const wingY = Math.sin(state.bird.wingPhase) * 5;
  ctx.fillStyle = "#ffffff";
  ctx.beginPath();
  ctx.ellipse(-2, 2 + wingY, 7, 4, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.stroke();

  // Eye
  ctx.fillStyle = "#ffffff";
  ctx.beginPath();
  ctx.arc(5, -4, 4, 0, Math.PI * 2);
  ctx.fill();
  ctx.stroke();
  ctx.fillStyle = "#000";
  ctx.beginPath();
  ctx.arc(6, -4, 2, 0, Math.PI * 2);
  ctx.fill();

  // Beak
  ctx.fillStyle = COLORS.beak;
  ctx.beginPath();
  ctx.moveTo(BIRD_R - 2, -3);
  ctx.lineTo(BIRD_R + 9, -1);
  ctx.lineTo(BIRD_R + 9, 3);
  ctx.lineTo(BIRD_R - 2, 5);
  ctx.closePath();
  ctx.fill();
  ctx.strokeStyle = COLORS.beakStroke;
  ctx.lineWidth = 1;
  ctx.stroke();

  ctx.restore();
}

function drawHeader() {
  ctx.fillStyle = "rgba(11,19,32,0.55)";
  ctx.fillRect(0, 0, WIDTH, HEADER_H);
  ctx.fillStyle = "#e7e9ee";
  ctx.font = "600 11px sans-serif";
  ctx.textAlign = "left";
  ctx.textBaseline = "alphabetic";
  ctx.fillText("SCORE", 16, 20);
  ctx.fillText("BEST", WIDTH - 80, 20);
  ctx.fillStyle = state.flashGold > 0 ? "#fde047" : "#ffffff";
  ctx.font = "700 24px ui-monospace, monospace";
  ctx.fillText(String(state.score).padStart(3, "0"), 16, 44);
  ctx.fillStyle = "#ffffff";
  ctx.fillText(String(state.best).padStart(3, "0"), WIDTH - 80, 44);
}

function drawCenterCard(title: string, subtitle: string) {
  const w = 280;
  const h = 140;
  const x = (WIDTH - w) / 2;
  const y = (HEIGHT - h) / 2 - 20;
  ctx.fillStyle = COLORS.panel;
  ctx.beginPath();
  ctx.roundRect(x, y, w, h, 14);
  ctx.fill();
  ctx.fillStyle = COLORS.text;
  ctx.font = "700 24px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(title, x + w / 2, y + 50);
  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "13px sans-serif";
  ctx.fillText(subtitle, x + w / 2, y + 92);
}

function render() {
  drawSky();
  drawClouds();
  drawHillsBackground();
  for (const p of state.pipes) drawPipe(p);
  drawGround();
  drawBird();
  drawHeader();

  if (state.phase === "ready") {
    drawCenterCard("Tap to start", "space / click / ↑ to flap");
  } else if (state.phase === "dead") {
    const newBest = state.score === state.best && state.best > 0;
    drawCenterCard(
      newBest ? "New best!" : "Game Over",
      `${state.score} · best ${state.best} · click to retry`,
    );
  }
}

await Andromeda.Window.mainloop(() => {
  update();
  render();
  win.presentCanvas(canvas);
});
