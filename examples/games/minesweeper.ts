const COLS = 16;
const ROWS = 16;
const MINES = 40;
const CELL = 34;
const GAP = 2;
const PADDING = 14;
const HEADER_H = 72;
const WIDTH = COLS * CELL + PADDING * 2;
const HEIGHT = ROWS * CELL + HEADER_H + PADDING;
const BEST_KEY = "andromeda.minesweeper.best";

type Phase = "playing" | "won" | "lost";

interface Cell {
  mine: boolean;
  revealed: boolean;
  flagged: boolean;
  adj: number;
}

interface GameState {
  grid: Cell[];
  phase: Phase;
  flags: number;
  firstClick: boolean;
  startMs: number;
  elapsed: number;
  explodedIdx: number;
  best: number;
}

// Dark palette. Grouped so the design intent is obvious.
const COLORS = {
  bg: "#0b0d12",
  panel: "#13161d",
  panelBorder: "#1f2430",
  cellHidden: "#1c2029",
  cellHiddenHover: "#252a36",
  cellRevealed: "#0f1217",
  cellExploded: "#3a1518",
  cellRevealedBorder: "rgba(255,255,255,0.03)",
  text: "#e7e9ee",
  textMuted: "#8b93a7",
  flag: "#f87171",
  flagPole: "#cbd5e1",
  mine: "#e5e7eb",
  mineHighlight: "#fca5a5",
  accent: "#60a5fa",
  green: "#22c55e",
  red: "#ef4444",
};

// Tailwind-ish number colors, tuned for a dark background.
const NUM_COLORS = [
  "",
  "#60a5fa", // 1 — sky
  "#4ade80", // 2 — green
  "#f87171", // 3 — rose
  "#c084fc", // 4 — violet
  "#fbbf24", // 5 — amber
  "#2dd4bf", // 6 — teal
  "#f9a8d4", // 7 — pink
  "#cbd5e1", // 8 — slate
];

function loadBest(): number {
  const raw = localStorage.getItem(BEST_KEY);
  if (!raw) return 0;
  const n = Number(raw);
  return Number.isFinite(n) && n > 0 ? n : 0;
}

function saveBest(seconds: number) {
  localStorage.setItem(BEST_KEY, String(seconds));
}

function emptyGrid(): Cell[] {
  const out: Cell[] = [];
  for (let i = 0; i < COLS * ROWS; i++) {
    out.push({ mine: false, revealed: false, flagged: false, adj: 0 });
  }
  return out;
}

function idxOf(x: number, y: number): number {
  return y * COLS + x;
}

function inBounds(x: number, y: number): boolean {
  return x >= 0 && x < COLS && y >= 0 && y < ROWS;
}

function forEachNeighbor(
  x: number,
  y: number,
  fn: (nx: number, ny: number) => void,
) {
  for (let dy = -1; dy <= 1; dy++) {
    for (let dx = -1; dx <= 1; dx++) {
      if (dx === 0 && dy === 0) continue;
      const nx = x + dx;
      const ny = y + dy;
      if (inBounds(nx, ny)) fn(nx, ny);
    }
  }
}

// First click is always safe: the clicked cell and its 8 neighbors never
// contain mines. Matches classic Windows-style minesweeper.
function placeMines(grid: Cell[], safeX: number, safeY: number) {
  const forbidden = new Set<number>();
  forbidden.add(idxOf(safeX, safeY));
  forEachNeighbor(safeX, safeY, (nx, ny) => forbidden.add(idxOf(nx, ny)));

  let placed = 0;
  while (placed < MINES) {
    const i = Math.floor(Math.random() * grid.length);
    if (forbidden.has(i) || grid[i].mine) continue;
    grid[i].mine = true;
    placed++;
  }

  for (let y = 0; y < ROWS; y++) {
    for (let x = 0; x < COLS; x++) {
      if (grid[idxOf(x, y)].mine) continue;
      let n = 0;
      forEachNeighbor(x, y, (nx, ny) => {
        if (grid[idxOf(nx, ny)].mine) n++;
      });
      grid[idxOf(x, y)].adj = n;
    }
  }
}

function floodReveal(grid: Cell[], x: number, y: number) {
  const stack = [[x, y]];
  while (stack.length > 0) {
    const [cx, cy] = stack.pop()!;
    const c = grid[idxOf(cx, cy)];
    if (c.revealed || c.flagged) continue;
    c.revealed = true;
    if (c.adj === 0 && !c.mine) {
      forEachNeighbor(cx, cy, (nx, ny) => {
        const n = grid[idxOf(nx, ny)];
        if (!n.revealed && !n.flagged) stack.push([nx, ny]);
      });
    }
  }
}

function checkWin(grid: Cell[]): boolean {
  for (const c of grid) {
    if (!c.mine && !c.revealed) return false;
  }
  return true;
}

function initialState(best: number): GameState {
  return {
    grid: emptyGrid(),
    phase: "playing",
    flags: 0,
    firstClick: true,
    startMs: 0,
    elapsed: 0,
    explodedIdx: -1,
    best,
  };
}

const win = Andromeda.createWindow({
  title: "Andromeda Minesweeper",
  width: WIDTH,
  height: HEIGHT,
});
console.log(`window ${win.rid} (${win.rawHandle().system})`);
const canvas = new OffscreenCanvas(WIDTH, HEIGHT);
const ctx = canvas.getContext("2d")!;

let state: GameState = initialState(loadBest());
let hoverX = -1;
let hoverY = -1;
let resetHover = false;

function reveal(x: number, y: number) {
  if (state.phase !== "playing") return;
  if (state.firstClick) {
    placeMines(state.grid, x, y);
    state.firstClick = false;
    state.startMs = Date.now();
  }
  const c = state.grid[idxOf(x, y)];
  if (c.revealed || c.flagged) return;
  if (c.mine) {
    c.revealed = true;
    state.explodedIdx = idxOf(x, y);
    state.phase = "lost";
    state.elapsed = Math.floor((Date.now() - state.startMs) / 1000);
    for (const g of state.grid) if (g.mine) g.revealed = true;
    return;
  }
  floodReveal(state.grid, x, y);
  if (checkWin(state.grid)) {
    state.phase = "won";
    state.elapsed = Math.floor((Date.now() - state.startMs) / 1000);
    if (state.best === 0 || state.elapsed < state.best) {
      state.best = state.elapsed;
      saveBest(state.best);
    }
  }
}

function chord(x: number, y: number) {
  const c = state.grid[idxOf(x, y)];
  if (!c.revealed || c.adj === 0) return;
  let flags = 0;
  forEachNeighbor(x, y, (nx, ny) => {
    if (state.grid[idxOf(nx, ny)].flagged) flags++;
  });
  if (flags !== c.adj) return;
  forEachNeighbor(x, y, (nx, ny) => {
    const n = state.grid[idxOf(nx, ny)];
    if (!n.flagged && !n.revealed) reveal(nx, ny);
  });
}

function toggleFlag(x: number, y: number) {
  if (state.phase !== "playing") return;
  const c = state.grid[idxOf(x, y)];
  if (c.revealed) return;
  c.flagged = !c.flagged;
  state.flags += c.flagged ? 1 : -1;
}

function cellFromPos(px: number, py: number): { x: number; y: number } | null {
  const gx = px - PADDING;
  const gy = py - HEADER_H;
  if (gx < 0 || gy < 0) return null;
  const x = Math.floor(gx / CELL);
  const y = Math.floor(gy / CELL);
  if (!inBounds(x, y)) return null;
  return { x, y };
}

function resetButtonRect(): { x: number; y: number; w: number; h: number } {
  const w = 140;
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

win.addEventListener("keydown", (e: any) => {
  const code: string = e.detail.code;
  if (code === "Escape") {
    win.close();
    return;
  }
  if (code === "KeyR" || (code === "Space" && state.phase !== "playing")) {
    state = initialState(state.best);
  }
});

win.addEventListener("mousemove", (e: any) => {
  const c = cellFromPos(e.detail.x, e.detail.y);
  hoverX = c?.x ?? -1;
  hoverY = c?.y ?? -1;
  resetHover = pointInRect(e.detail.x, e.detail.y, resetButtonRect());
});

win.addEventListener("mousedown", (e: any) => {
  if (pointInRect(e.detail.x, e.detail.y, resetButtonRect())) {
    state = initialState(state.best);
    return;
  }
  const c = cellFromPos(e.detail.x, e.detail.y);
  if (!c) return;
  if (e.detail.button === 2) {
    toggleFlag(c.x, c.y);
  } else if (e.detail.button === 1) {
    chord(c.x, c.y);
  } else if (e.detail.button === 0) {
    const cell = state.grid[idxOf(c.x, c.y)];
    if (cell.revealed && cell.adj > 0) chord(c.x, c.y);
    else reveal(c.x, c.y);
  }
});

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

  // Flag counter pill (left).
  const leftPill = { x: PADDING, y: (HEADER_H - 36) / 2, w: 96, h: 36 };
  roundedRect(
    leftPill.x,
    leftPill.y,
    leftPill.w,
    leftPill.h,
    10,
    COLORS.cellHidden,
    COLORS.panelBorder,
    1,
  );
  // Flag mini-icon.
  const flagX = leftPill.x + 14;
  const flagY = leftPill.y + leftPill.h / 2;
  ctx.fillStyle = COLORS.flag;
  ctx.beginPath();
  ctx.moveTo(flagX, flagY - 8);
  ctx.lineTo(flagX + 12, flagY - 4);
  ctx.lineTo(flagX, flagY);
  ctx.closePath();
  ctx.fill();
  ctx.fillStyle = COLORS.flagPole;
  ctx.fillRect(flagX - 1, flagY - 10, 2, 18);
  // Count.
  ctx.fillStyle = COLORS.text;
  ctx.font = "600 18px ui-monospace, monospace";
  ctx.textAlign = "right";
  ctx.textBaseline = "middle";
  const remaining = MINES - state.flags;
  ctx.fillText(
    String(remaining).padStart(3, "0"),
    leftPill.x + leftPill.w - 14,
    leftPill.y + leftPill.h / 2,
  );

  // Reset button (center).
  const r = resetButtonRect();
  const resetBg = state.phase === "won"
    ? "#14532d"
    : state.phase === "lost"
    ? "#4c0519"
    : resetHover
    ? COLORS.cellHiddenHover
    : COLORS.cellHidden;
  const resetFg = state.phase === "won"
    ? "#86efac"
    : state.phase === "lost"
    ? "#fecaca"
    : COLORS.text;
  roundedRect(r.x, r.y, r.w, r.h, 10, resetBg, COLORS.panelBorder, 1);
  ctx.fillStyle = resetFg;
  ctx.font = "600 13px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  const label = state.phase === "won"
    ? "YOU WIN — PLAY AGAIN"
    : state.phase === "lost"
    ? "BOOM — TRY AGAIN"
    : "NEW GAME";
  ctx.fillText(label, r.x + r.w / 2, r.y + r.h / 2);

  // Timer pill (right).
  const rightPill = {
    x: WIDTH - PADDING - 96,
    y: (HEADER_H - 36) / 2,
    w: 96,
    h: 36,
  };
  roundedRect(
    rightPill.x,
    rightPill.y,
    rightPill.w,
    rightPill.h,
    10,
    COLORS.cellHidden,
    COLORS.panelBorder,
    1,
  );
  // Clock icon.
  const clockX = rightPill.x + 20;
  const clockY = rightPill.y + rightPill.h / 2;
  ctx.strokeStyle = COLORS.textMuted;
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  ctx.arc(clockX, clockY, 8, 0, Math.PI * 2);
  ctx.stroke();
  ctx.beginPath();
  ctx.moveTo(clockX, clockY);
  ctx.lineTo(clockX, clockY - 5);
  ctx.moveTo(clockX, clockY);
  ctx.lineTo(clockX + 4, clockY);
  ctx.stroke();

  let elapsed = state.elapsed;
  if (state.phase === "playing" && !state.firstClick) {
    elapsed = Math.floor((Date.now() - state.startMs) / 1000);
  }
  ctx.fillStyle = COLORS.text;
  ctx.font = "600 18px ui-monospace, monospace";
  ctx.textAlign = "right";
  ctx.textBaseline = "middle";
  ctx.fillText(
    String(elapsed).padStart(3, "0"),
    rightPill.x + rightPill.w - 14,
    rightPill.y + rightPill.h / 2,
  );
}

function drawMineIcon(cx: number, cy: number, highlighted: boolean) {
  ctx.fillStyle = highlighted ? COLORS.mineHighlight : COLORS.mine;
  ctx.beginPath();
  ctx.arc(cx, cy, 6, 0, Math.PI * 2);
  ctx.fill();
  ctx.strokeStyle = highlighted ? COLORS.mineHighlight : COLORS.mine;
  ctx.lineWidth = 1.8;
  ctx.lineCap = "round";
  for (let i = 0; i < 8; i++) {
    const ang = (i * Math.PI) / 4;
    ctx.beginPath();
    ctx.moveTo(cx + Math.cos(ang) * 7, cy + Math.sin(ang) * 7);
    ctx.lineTo(cx + Math.cos(ang) * 11, cy + Math.sin(ang) * 11);
    ctx.stroke();
  }
  // Tiny highlight dot.
  ctx.fillStyle = "rgba(255,255,255,0.4)";
  ctx.beginPath();
  ctx.arc(cx - 2, cy - 2, 1.5, 0, Math.PI * 2);
  ctx.fill();
}

function drawFlagIcon(cx: number, cy: number) {
  ctx.fillStyle = COLORS.flagPole;
  ctx.fillRect(cx - 1, cy - 9, 2, 17);
  ctx.fillStyle = COLORS.flag;
  ctx.beginPath();
  ctx.moveTo(cx, cy - 9);
  ctx.lineTo(cx + 11, cy - 5);
  ctx.lineTo(cx, cy - 1);
  ctx.closePath();
  ctx.fill();
}

function drawCell(x: number, y: number, cell: Cell) {
  const px = PADDING + x * CELL + GAP / 2;
  const py = HEADER_H + y * CELL + GAP / 2;
  const size = CELL - GAP;
  const cx = px + size / 2;
  const cy = py + size / 2;

  if (!cell.revealed) {
    const isHover = hoverX === x && hoverY === y && state.phase === "playing";
    roundedRect(
      px,
      py,
      size,
      size,
      6,
      isHover ? COLORS.cellHiddenHover : COLORS.cellHidden,
    );
    // Subtle top highlight for depth.
    ctx.fillStyle = "rgba(255,255,255,0.03)";
    ctx.beginPath();
    ctx.roundRect(px + 1, py + 1, size - 2, 6, 5);
    ctx.fill();

    if (cell.flagged) drawFlagIcon(cx, cy);
  } else {
    const isExploded = idxOf(x, y) === state.explodedIdx;
    roundedRect(
      px,
      py,
      size,
      size,
      6,
      isExploded ? COLORS.cellExploded : COLORS.cellRevealed,
      COLORS.cellRevealedBorder,
      1,
    );

    if (cell.mine) {
      drawMineIcon(cx, cy, isExploded);
    } else if (cell.adj > 0) {
      ctx.fillStyle = NUM_COLORS[cell.adj];
      ctx.font = "700 17px sans-serif";
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText(String(cell.adj), cx, cy + 1);
    }
  }
}

function drawBoard() {
  for (let y = 0; y < ROWS; y++) {
    for (let x = 0; x < COLS; x++) {
      drawCell(x, y, state.grid[idxOf(x, y)]);
    }
  }
}

function drawOverlay() {
  if (state.phase === "playing") return;

  const overlayY = HEADER_H + (ROWS * CELL) / 2 - 70;
  const overlayH = 140;

  // Dimmed backdrop over the board area only.
  ctx.fillStyle = "rgba(5, 7, 11, 0.72)";
  ctx.fillRect(PADDING, HEADER_H, COLS * CELL, ROWS * CELL);

  // Card.
  const cardW = 280;
  const cardH = overlayH;
  const cardX = (WIDTH - cardW) / 2;
  const cardY = overlayY;
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

  // Accent dot.
  ctx.fillStyle = state.phase === "won" ? COLORS.green : COLORS.red;
  ctx.beginPath();
  ctx.arc(cardX + cardW / 2, cardY + 28, 4, 0, Math.PI * 2);
  ctx.fill();

  // Heading.
  ctx.fillStyle = COLORS.text;
  ctx.font = "700 24px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(
    state.phase === "won" ? "You swept it" : "You hit a mine",
    cardX + cardW / 2,
    cardY + 58,
  );

  // Stat line.
  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "14px sans-serif";
  const bestStr = state.best > 0 ? `best ${state.best}s` : "no best yet";
  ctx.fillText(
    `${state.elapsed}s · ${bestStr}`,
    cardX + cardW / 2,
    cardY + 88,
  );

  // Hint.
  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "12px sans-serif";
  ctx.fillText(
    "press R or click NEW GAME above",
    cardX + cardW / 2,
    cardY + 114,
  );
}

function drawHint() {
  if (!state.firstClick || state.phase !== "playing") return;
  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "12px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(
    "left-click reveal · right-click flag · middle-click chord",
    WIDTH / 2,
    HEIGHT - PADDING / 2 - 2,
  );
}

function render() {
  ctx.fillStyle = COLORS.bg;
  ctx.fillRect(0, 0, WIDTH, HEIGHT);

  drawHeader();
  drawBoard();
  drawOverlay();
  drawHint();
}

// let frameCount = 0;
// const fpsStart = Date.now();
await Andromeda.Window.mainloop(() => {
  // frameCount++;
  render();
  win.presentCanvas(canvas);
  // if (frameCount % 300 === 0) {
  //   const elapsed = (Date.now() - fpsStart) / 1000;
  //   console.log(
  //     `minesweeper: frame ${frameCount} — avg fps ${
  //       (frameCount / elapsed).toFixed(1)
  //     }`,
  //   );
  // }
});
