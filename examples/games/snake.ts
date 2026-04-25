const CELL = 24;
const COLS = 28;
const ROWS = 20;
const PADDING = 14;
const HEADER_H = 72;
const WIDTH = COLS * CELL + PADDING * 2;
const HEIGHT = ROWS * CELL + HEADER_H + PADDING;
const BEST_KEY = "andromeda.snake.best";

const TICKS_PER_MOVE_START = 5;
const TICKS_PER_MOVE_MIN = 2;
const INPUT_BUFFER = 3;

type Dir = "up" | "down" | "left" | "right";
type Phase = "playing" | "game-over";

interface Cell {
  x: number;
  y: number;
}

interface GameState {
  snake: Cell[];
  dir: Dir;
  inputQueue: Dir[];
  food: Cell;
  score: number;
  best: number;
  ticksPerMove: number;
  tick: number;
  phase: Phase;
  foodPulse: number;
}

const COLORS = {
  bg: "#0b0d12",
  panel: "#13161d",
  panelBorder: "#1f2430",
  grid: "rgba(255,255,255,0.025)",
  board: "#0f1217",
  boardBorder: "#1f2430",
  snakeHead: "#86efac",
  snakeBody: "#4ade80",
  food: "#f472b6",
  foodGlow: "rgba(244, 114, 182, 0.28)",
  text: "#e7e9ee",
  textMuted: "#8b93a7",
  accent: "#60a5fa",
  green: "#22c55e",
  red: "#ef4444",
};

function loadBest(): number {
  const raw = localStorage.getItem(BEST_KEY);
  if (!raw) return 0;
  const n = Number(raw);
  return Number.isFinite(n) && n >= 0 ? n : 0;
}

function saveBest(best: number) {
  localStorage.setItem(BEST_KEY, String(best));
}

function randomFood(snake: Cell[]): Cell {
  while (true) {
    const f = {
      x: Math.floor(Math.random() * COLS),
      y: Math.floor(Math.random() * ROWS),
    };
    if (!snake.some((s) => s.x === f.x && s.y === f.y)) return f;
  }
}

function initialState(best: number): GameState {
  const snake: Cell[] = [
    { x: 6, y: ROWS >> 1 },
    { x: 5, y: ROWS >> 1 },
    { x: 4, y: ROWS >> 1 },
  ];
  return {
    snake,
    dir: "right",
    inputQueue: [],
    food: randomFood(snake),
    score: 0,
    best,
    ticksPerMove: TICKS_PER_MOVE_START,
    tick: 0,
    phase: "playing",
    foodPulse: 0,
  };
}

// Prevent 180° reversal (would instantly self-collide).
function opposite(a: Dir, b: Dir): boolean {
  return (
    (a === "up" && b === "down") ||
    (a === "down" && b === "up") ||
    (a === "left" && b === "right") ||
    (a === "right" && b === "left")
  );
}

const win = Andromeda.createWindow({
  title: "Andromeda Snake",
  width: WIDTH,
  height: HEIGHT,
});
console.log(`window ${win.rid} (${win.rawHandle().system})`);
const canvas = new OffscreenCanvas(WIDTH, HEIGHT);
const ctx = canvas.getContext("2d")!;

let state: GameState = initialState(loadBest());
let resetHover = false;

win.addEventListener("keydown", (e: any) => {
  const code: string = e.detail.code;
  if (code === "Escape") {
    win.close();
    return;
  }
  if (code === "Space" && state.phase === "game-over") {
    state = initialState(state.best);
    return;
  }
  if (code === "KeyR") {
    state = initialState(state.best);
    return;
  }
  let next: Dir | null = null;
  if (code === "ArrowUp" || code === "KeyW") next = "up";
  else if (code === "ArrowDown" || code === "KeyS") next = "down";
  else if (code === "ArrowLeft" || code === "KeyA") next = "left";
  else if (code === "ArrowRight" || code === "KeyD") next = "right";
  if (!next) return;
  // Validate against the last buffered direction (not just the current
  // one) so chained taps like right→up→left queue cleanly without any
  // leg being rejected or dropped to a 180° self-kill.
  const last = state.inputQueue.length > 0
    ? state.inputQueue[state.inputQueue.length - 1]
    : state.dir;
  if (next === last || opposite(last, next)) return;
  if (state.inputQueue.length < INPUT_BUFFER) {
    state.inputQueue.push(next);
  }
});

function resetButtonRect(): { x: number; y: number; w: number; h: number } {
  const w = 120;
  const h = 36;
  return { x: (WIDTH - w) / 2, y: (HEADER_H - h) / 2, w, h };
}

function pointInRect(
  x: number,
  y: number,
  r: { x: number; y: number; w: number; h: number },
): boolean {
  return x >= r.x && x <= r.x + r.w && y >= r.y && y <= r.y + r.h;
}

win.addEventListener("mousemove", (e: any) => {
  resetHover = pointInRect(e.detail.x, e.detail.y, resetButtonRect());
});

win.addEventListener("mousedown", (e: any) => {
  if (
    e.detail.button === 0 &&
    pointInRect(e.detail.x, e.detail.y, resetButtonRect())
  ) {
    state = initialState(state.best);
  }
});

function step() {
  const queued = state.inputQueue.shift();
  if (queued) state.dir = queued;
  const head = state.snake[0];
  const next: Cell = { x: head.x, y: head.y };
  if (state.dir === "up") next.y--;
  else if (state.dir === "down") next.y++;
  else if (state.dir === "left") next.x--;
  else if (state.dir === "right") next.x++;

  if (next.x < 0 || next.x >= COLS || next.y < 0 || next.y >= ROWS) {
    state.phase = "game-over";
    return;
  }
  if (state.snake.some((s) => s.x === next.x && s.y === next.y)) {
    state.phase = "game-over";
    return;
  }

  state.snake.unshift(next);
  if (next.x === state.food.x && next.y === state.food.y) {
    state.score += 10;
    if (state.score > state.best) {
      state.best = state.score;
      saveBest(state.best);
    }
    state.food = randomFood(state.snake);
    if (state.ticksPerMove > TICKS_PER_MOVE_MIN) state.ticksPerMove--;
  } else {
    state.snake.pop();
  }
}

function update() {
  state.foodPulse += 0.08;
  if (state.phase !== "playing") return;
  state.tick++;
  if (state.tick >= state.ticksPerMove) {
    state.tick = 0;
    step();
  }
}

function roundedRect(
  x: number,
  y: number,
  w: number,
  h: number,
  r: number,
  fill?: string,
  stroke?: string,
  strokeWidth = 1,
) {
  ctx.beginPath();
  ctx.roundRect(x, y, w, h, r);
  if (fill) {
    ctx.fillStyle = fill;
    ctx.fill();
  }
  if (stroke) {
    ctx.strokeStyle = stroke;
    ctx.lineWidth = strokeWidth;
    ctx.stroke();
  }
}

function drawHeader() {
  ctx.fillStyle = COLORS.panel;
  ctx.fillRect(0, 0, WIDTH, HEADER_H);
  ctx.fillStyle = COLORS.panelBorder;
  ctx.fillRect(0, HEADER_H - 1, WIDTH, 1);

  // Score pill (left).
  const leftPill = { x: PADDING, y: (HEADER_H - 36) / 2, w: 132, h: 36 };
  roundedRect(
    leftPill.x,
    leftPill.y,
    leftPill.w,
    leftPill.h,
    10,
    COLORS.board,
    COLORS.panelBorder,
    1,
  );
  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "600 10px sans-serif";
  ctx.textAlign = "left";
  ctx.textBaseline = "middle";
  ctx.fillText("SCORE", leftPill.x + 14, leftPill.y + 12);
  ctx.fillStyle = COLORS.text;
  ctx.font = "700 18px ui-monospace, monospace";
  ctx.textAlign = "right";
  ctx.fillText(
    String(state.score).padStart(3, "0"),
    leftPill.x + leftPill.w - 14,
    leftPill.y + 24,
  );

  // Reset button (center).
  const r = resetButtonRect();
  const bg = state.phase === "game-over"
    ? "#4c0519"
    : resetHover
    ? "#252a36"
    : "#1c2029";
  const fg = state.phase === "game-over" ? "#fecaca" : COLORS.text;
  roundedRect(r.x, r.y, r.w, r.h, 10, bg, COLORS.panelBorder, 1);
  ctx.fillStyle = fg;
  ctx.font = "600 13px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(
    state.phase === "game-over" ? "PLAY AGAIN" : "NEW GAME",
    r.x + r.w / 2,
    r.y + r.h / 2,
  );

  // Best pill (right).
  const rightPill = {
    x: WIDTH - PADDING - 132,
    y: (HEADER_H - 36) / 2,
    w: 132,
    h: 36,
  };
  roundedRect(
    rightPill.x,
    rightPill.y,
    rightPill.w,
    rightPill.h,
    10,
    COLORS.board,
    COLORS.panelBorder,
    1,
  );
  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "600 10px sans-serif";
  ctx.textAlign = "left";
  ctx.textBaseline = "middle";
  ctx.fillText("BEST", rightPill.x + 14, rightPill.y + 12);
  ctx.fillStyle = state.best > 0 && state.score === state.best
    ? COLORS.green
    : COLORS.text;
  ctx.font = "700 18px ui-monospace, monospace";
  ctx.textAlign = "right";
  ctx.fillText(
    String(state.best).padStart(3, "0"),
    rightPill.x + rightPill.w - 14,
    rightPill.y + 24,
  );
}

function drawBoard() {
  const bx = PADDING;
  const by = HEADER_H;
  const bw = COLS * CELL;
  const bh = ROWS * CELL;

  roundedRect(bx, by, bw, bh, 8, COLORS.board, COLORS.boardBorder, 1);

  // Subtle grid.
  ctx.strokeStyle = COLORS.grid;
  ctx.lineWidth = 1;
  ctx.beginPath();
  for (let x = 1; x < COLS; x++) {
    ctx.moveTo(bx + x * CELL, by + 4);
    ctx.lineTo(bx + x * CELL, by + bh - 4);
  }
  for (let y = 1; y < ROWS; y++) {
    ctx.moveTo(bx + 4, by + y * CELL);
    ctx.lineTo(bx + bw - 4, by + y * CELL);
  }
  ctx.stroke();
}

function drawFood() {
  const bx = PADDING;
  const by = HEADER_H;
  const cx = bx + state.food.x * CELL + CELL / 2;
  const cy = by + state.food.y * CELL + CELL / 2;
  const r = CELL / 2 - 4;
  const pulse = 1 + Math.sin(state.foodPulse) * 0.08;

  const glow = ctx.createRadialGradient(cx, cy, 0, cx, cy, r * 2.6);
  glow.addColorStop(0, COLORS.foodGlow);
  glow.addColorStop(1, "rgba(244, 114, 182, 0)");
  ctx.fillStyle = glow;
  ctx.beginPath();
  ctx.arc(cx, cy, r * 2.6, 0, Math.PI * 2);
  ctx.fill();

  ctx.fillStyle = COLORS.food;
  ctx.beginPath();
  ctx.arc(cx, cy, r * pulse, 0, Math.PI * 2);
  ctx.fill();
}

function drawSnake() {
  const bx = PADDING;
  const by = HEADER_H;
  const inset = 2;
  const radius = 6;
  state.snake.forEach((seg, i) => {
    const px = bx + seg.x * CELL + inset;
    const py = by + seg.y * CELL + inset;
    const size = CELL - inset * 2;
    roundedRect(
      px,
      py,
      size,
      size,
      radius,
      i === 0 ? COLORS.snakeHead : COLORS.snakeBody,
    );
  });
}

function drawOverlay() {
  if (state.phase !== "game-over") return;

  const bx = PADDING;
  const by = HEADER_H;
  const bw = COLS * CELL;
  const bh = ROWS * CELL;

  ctx.fillStyle = "rgba(5, 7, 11, 0.72)";
  ctx.fillRect(bx, by, bw, bh);

  const cardW = 280;
  const cardH = 150;
  const cardX = (WIDTH - cardW) / 2;
  const cardY = by + bh / 2 - cardH / 2;
  roundedRect(
    cardX,
    cardY,
    cardW,
    cardH,
    14,
    COLORS.panel,
    COLORS.panelBorder,
    1,
  );

  // Accent dot (green if new best, red otherwise).
  const isNewBest = state.best > 0 && state.score === state.best;
  ctx.fillStyle = isNewBest ? COLORS.green : COLORS.red;
  ctx.beginPath();
  ctx.arc(cardX + cardW / 2, cardY + 26, 4, 0, Math.PI * 2);
  ctx.fill();

  ctx.fillStyle = COLORS.text;
  ctx.font = "700 24px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(
    isNewBest ? "New best!" : "Game over",
    cardX + cardW / 2,
    cardY + 58,
  );

  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "14px sans-serif";
  const bestStr = state.best > 0 ? `best ${state.best}` : "no best yet";
  ctx.fillText(
    `${state.score} · ${bestStr}`,
    cardX + cardW / 2,
    cardY + 92,
  );

  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "12px sans-serif";
  ctx.fillText(
    "press Space or click PLAY AGAIN",
    cardX + cardW / 2,
    cardY + 120,
  );
}

function drawHint() {
  if (state.phase !== "playing" || state.score > 0) return;
  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "12px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(
    "arrows / WASD to steer · R to reset · Esc to quit",
    WIDTH / 2,
    HEIGHT - PADDING / 2 - 2,
  );
}

function render() {
  ctx.fillStyle = COLORS.bg;
  ctx.fillRect(0, 0, WIDTH, HEIGHT);

  drawHeader();
  drawBoard();
  drawFood();
  drawSnake();
  drawOverlay();
  drawHint();
}

let frameCount = 0;
const fpsStart = Date.now();
await Andromeda.Window.mainloop(() => {
  frameCount++;
  update();
  render();
  win.presentCanvas(canvas);
  if (frameCount % 300 === 0) {
    const elapsed = (Date.now() - fpsStart) / 1000;
    console.log(
      `snake: frame ${frameCount} — avg fps ${(frameCount / elapsed).toFixed(1)}`,
    );
  }
});
