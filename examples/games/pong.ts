const WIDTH = 760;
const HEIGHT = 480;
const PADDING = 14;
const HEADER_H = 64;
const FIELD_Y = HEADER_H;
const FIELD_H = HEIGHT - HEADER_H - PADDING;
const BEST_KEY = "andromeda.pong.best";

const PADDLE_W = 12;
const PADDLE_H = 80;
const PADDLE_MARGIN = 28;
const PADDLE_SPEED = 7;
const AI_SPEED = 5.4;

const BALL_R = 7;
const BALL_SPEED = 5.2;
const BALL_MAX_SPEED = 12;
const BALL_SPEED_UP = 1.04;

const WIN_SCORE = 7;

const COLORS = {
  bg: "#0b0d12",
  panel: "#13161d",
  panelBorder: "#1f2430",
  field: "#0f1217",
  center: "rgba(255,255,255,0.08)",
  text: "#e7e9ee",
  textMuted: "#8b93a7",
  player: "#60a5fa",
  ai: "#f472b6",
  ball: "#fde047",
  ballGlow: "rgba(253, 224, 71, 0.3)",
  green: "#22c55e",
  red: "#ef4444",
};

type Phase = "serve" | "playing" | "game-over";

interface GameState {
  playerY: number;
  aiY: number;
  ball: { x: number; y: number; vx: number; vy: number };
  playerScore: number;
  aiScore: number;
  phase: Phase;
  serveDir: 1 | -1;
  serveTimer: number;
  winner: "player" | "ai" | null;
  bestWinTime: number;
  rallyFrames: number;
  matchStart: number;
}

function loadBest(): number {
  const n = Number(localStorage.getItem(BEST_KEY));
  return Number.isFinite(n) && n > 0 ? n : 0;
}
function saveBest(n: number) {
  localStorage.setItem(BEST_KEY, String(n));
}

function serveBall(dir: 1 | -1) {
  const angle = (Math.random() - 0.5) * 0.7;
  return {
    x: WIDTH / 2,
    y: FIELD_Y + FIELD_H / 2,
    vx: Math.cos(angle) * BALL_SPEED * dir,
    vy: Math.sin(angle) * BALL_SPEED,
  };
}

function initialState(bestWinTime: number): GameState {
  return {
    playerY: FIELD_Y + (FIELD_H - PADDLE_H) / 2,
    aiY: FIELD_Y + (FIELD_H - PADDLE_H) / 2,
    ball: serveBall(Math.random() < 0.5 ? 1 : -1),
    playerScore: 0,
    aiScore: 0,
    phase: "serve",
    serveDir: 1,
    serveTimer: 60,
    winner: null,
    bestWinTime,
    rallyFrames: 0,
    matchStart: Date.now(),
  };
}

const win = Andromeda.createWindow({
  title: "Andromeda Pong",
  width: WIDTH,
  height: HEIGHT,
});
const canvas = new OffscreenCanvas(WIDTH, HEIGHT);
const ctx = canvas.getContext("2d")!;

const keys = new Set<string>();
let state: GameState = initialState(loadBest());

win.addEventListener("keydown", (e: any) => {
  const code: string = e.detail.code;
  if (code === "Escape") {
    win.close();
    return;
  }
  keys.add(code);
  if (code === "Space") {
    if (state.phase === "game-over") state = initialState(state.bestWinTime);
    else if (state.phase === "serve") state.phase = "playing";
  }
});

win.addEventListener("keyup", (e: any) => keys.delete(e.detail.code));

function ballHitsPaddle(bx: number, by: number, px: number, py: number) {
  const cx = Math.max(px, Math.min(bx, px + PADDLE_W));
  const cy = Math.max(py, Math.min(by, py + PADDLE_H));
  const dx = bx - cx;
  const dy = by - cy;
  return dx * dx + dy * dy <= BALL_R * BALL_R;
}

function clampSpeed(b: { vx: number; vy: number }) {
  const s = Math.hypot(b.vx, b.vy);
  if (s > BALL_MAX_SPEED) {
    b.vx *= BALL_MAX_SPEED / s;
    b.vy *= BALL_MAX_SPEED / s;
  }
}

function update() {
  if (state.phase === "game-over") return;

  if (keys.has("ArrowUp") || keys.has("KeyW")) state.playerY -= PADDLE_SPEED;
  if (keys.has("ArrowDown") || keys.has("KeyS")) state.playerY += PADDLE_SPEED;
  state.playerY = Math.max(
    FIELD_Y,
    Math.min(FIELD_Y + FIELD_H - PADDLE_H, state.playerY),
  );

  const aiCenter = state.aiY + PADDLE_H / 2;
  const diff = state.ball.y - aiCenter;
  if (Math.abs(diff) > 6) state.aiY += Math.sign(diff) * AI_SPEED;
  state.aiY = Math.max(
    FIELD_Y,
    Math.min(FIELD_Y + FIELD_H - PADDLE_H, state.aiY),
  );

  if (state.phase === "serve") {
    state.serveTimer--;
    if (state.serveTimer <= 0) state.phase = "playing";
    return;
  }

  const b = state.ball;
  b.x += b.vx;
  b.y += b.vy;
  state.rallyFrames++;

  if (b.y - BALL_R < FIELD_Y) {
    b.y = FIELD_Y + BALL_R;
    b.vy = Math.abs(b.vy);
  } else if (b.y + BALL_R > FIELD_Y + FIELD_H) {
    b.y = FIELD_Y + FIELD_H - BALL_R;
    b.vy = -Math.abs(b.vy);
  }

  const playerX = PADDLE_MARGIN;
  const aiX = WIDTH - PADDLE_MARGIN - PADDLE_W;
  if (b.vx < 0 && ballHitsPaddle(b.x, b.y, playerX, state.playerY)) {
    b.x = playerX + PADDLE_W + BALL_R;
    b.vx = Math.abs(b.vx) * BALL_SPEED_UP;
    const hit = (b.y - (state.playerY + PADDLE_H / 2)) / (PADDLE_H / 2);
    b.vy = hit * BALL_SPEED * 1.2;
    clampSpeed(b);
  } else if (b.vx > 0 && ballHitsPaddle(b.x, b.y, aiX, state.aiY)) {
    b.x = aiX - BALL_R;
    b.vx = -Math.abs(b.vx) * BALL_SPEED_UP;
    const hit = (b.y - (state.aiY + PADDLE_H / 2)) / (PADDLE_H / 2);
    b.vy = hit * BALL_SPEED * 1.2;
    clampSpeed(b);
  }

  if (b.x + BALL_R < 0) {
    state.aiScore++;
    state.serveDir = 1;
    state.ball = serveBall(state.serveDir);
    state.phase = "serve";
    state.serveTimer = 60;
  } else if (b.x - BALL_R > WIDTH) {
    state.playerScore++;
    state.serveDir = -1;
    state.ball = serveBall(state.serveDir);
    state.phase = "serve";
    state.serveTimer = 60;
  }

  if (state.playerScore >= WIN_SCORE) {
    state.phase = "game-over";
    state.winner = "player";
    const matchTime = Math.floor((Date.now() - state.matchStart) / 1000);
    if (state.bestWinTime === 0 || matchTime < state.bestWinTime) {
      state.bestWinTime = matchTime;
      saveBest(state.bestWinTime);
    }
  } else if (state.aiScore >= WIN_SCORE) {
    state.phase = "game-over";
    state.winner = "ai";
  }
}

function roundedRect(x: number, y: number, w: number, h: number, r: number, fill?: string, stroke?: string) {
  ctx.beginPath();
  ctx.roundRect(x, y, w, h, r);
  if (fill) {
    ctx.fillStyle = fill;
    ctx.fill();
  }
  if (stroke) {
    ctx.strokeStyle = stroke;
    ctx.lineWidth = 1;
    ctx.stroke();
  }
}

function drawHeader() {
  ctx.fillStyle = COLORS.panel;
  ctx.fillRect(0, 0, WIDTH, HEADER_H);
  ctx.fillStyle = COLORS.panelBorder;
  ctx.fillRect(0, HEADER_H - 1, WIDTH, 1);

  const y = (HEADER_H - 40) / 2;

  // Player score pill
  const p1 = { x: PADDING, y, w: 120, h: 40 };
  roundedRect(p1.x, p1.y, p1.w, p1.h, 10, COLORS.field, COLORS.panelBorder);
  ctx.fillStyle = COLORS.player;
  ctx.font = "600 10px sans-serif";
  ctx.textAlign = "left";
  ctx.textBaseline = "middle";
  ctx.fillText("YOU", p1.x + 12, p1.y + 14);
  ctx.fillStyle = COLORS.text;
  ctx.font = "700 22px ui-monospace, monospace";
  ctx.textAlign = "right";
  ctx.fillText(String(state.playerScore), p1.x + p1.w - 12, p1.y + 26);

  // AI score pill
  const p2 = { x: WIDTH - PADDING - 120, y, w: 120, h: 40 };
  roundedRect(p2.x, p2.y, p2.w, p2.h, 10, COLORS.field, COLORS.panelBorder);
  ctx.fillStyle = COLORS.ai;
  ctx.font = "600 10px sans-serif";
  ctx.textAlign = "left";
  ctx.textBaseline = "middle";
  ctx.fillText("AI", p2.x + 12, p2.y + 14);
  ctx.fillStyle = COLORS.text;
  ctx.font = "700 22px ui-monospace, monospace";
  ctx.textAlign = "right";
  ctx.fillText(String(state.aiScore), p2.x + p2.w - 12, p2.y + 26);

  // Center best time
  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "600 11px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(
    state.bestWinTime > 0
      ? `FIRST TO ${WIN_SCORE} · BEST WIN ${state.bestWinTime}s`
      : `FIRST TO ${WIN_SCORE}`,
    WIDTH / 2,
    HEADER_H / 2,
  );
}

function drawField() {
  roundedRect(
    PADDING,
    FIELD_Y,
    WIDTH - PADDING * 2,
    FIELD_H,
    8,
    COLORS.field,
    COLORS.panelBorder,
  );
  // Center line — dashed
  ctx.fillStyle = COLORS.center;
  for (let y = FIELD_Y + 10; y < FIELD_Y + FIELD_H; y += 22) {
    ctx.fillRect(WIDTH / 2 - 1.5, y, 3, 12);
  }
}

function drawPaddles() {
  const playerX = PADDLE_MARGIN;
  const aiX = WIDTH - PADDLE_MARGIN - PADDLE_W;
  roundedRect(playerX, state.playerY, PADDLE_W, PADDLE_H, 6, COLORS.player);
  roundedRect(aiX, state.aiY, PADDLE_W, PADDLE_H, 6, COLORS.ai);
}

function drawBall() {
  const b = state.ball;
  const glow = ctx.createRadialGradient(b.x, b.y, 0, b.x, b.y, BALL_R * 3);
  glow.addColorStop(0, COLORS.ballGlow);
  glow.addColorStop(1, "rgba(253, 224, 71, 0)");
  ctx.fillStyle = glow;
  ctx.beginPath();
  ctx.arc(b.x, b.y, BALL_R * 3, 0, Math.PI * 2);
  ctx.fill();

  ctx.fillStyle = COLORS.ball;
  ctx.beginPath();
  ctx.arc(b.x, b.y, BALL_R, 0, Math.PI * 2);
  ctx.fill();
}

function drawOverlay() {
  if (state.phase !== "game-over") return;
  ctx.fillStyle = "rgba(5,7,11,0.75)";
  ctx.fillRect(PADDING, FIELD_Y, WIDTH - PADDING * 2, FIELD_H);

  const cardW = 340;
  const cardH = 160;
  const cardX = (WIDTH - cardW) / 2;
  const cardY = FIELD_Y + (FIELD_H - cardH) / 2;
  roundedRect(cardX, cardY, cardW, cardH, 14, COLORS.panel, COLORS.panelBorder);

  ctx.fillStyle = state.winner === "player" ? COLORS.green : COLORS.red;
  ctx.beginPath();
  ctx.arc(cardX + cardW / 2, cardY + 28, 4, 0, Math.PI * 2);
  ctx.fill();

  ctx.fillStyle = COLORS.text;
  ctx.font = "700 26px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(
    state.winner === "player" ? "You win" : "AI wins",
    cardX + cardW / 2,
    cardY + 62,
  );
  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "14px sans-serif";
  ctx.fillText(
    `${state.playerScore} — ${state.aiScore}`,
    cardX + cardW / 2,
    cardY + 96,
  );
  ctx.font = "12px sans-serif";
  ctx.fillText(
    "press Space to play again",
    cardX + cardW / 2,
    cardY + 126,
  );
}

function render() {
  ctx.fillStyle = COLORS.bg;
  ctx.fillRect(0, 0, WIDTH, HEIGHT);
  drawHeader();
  drawField();
  drawPaddles();
  drawBall();
  drawOverlay();

  if (state.phase === "serve") {
    ctx.fillStyle = COLORS.textMuted;
    ctx.font = "13px sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(
      "press Space to serve · W/S or ↑/↓ to move",
      WIDTH / 2,
      FIELD_Y + FIELD_H - 22,
    );
  }
}

await Andromeda.Window.mainloop(() => {
  update();
  render();
  win.presentCanvas(canvas);
});
