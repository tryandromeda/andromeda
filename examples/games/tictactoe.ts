const WIDTH = 440;
const HEIGHT = 560;
const PADDING = 14;
const HEADER_H = 72;
const BOARD_SIZE = WIDTH - PADDING * 2;
const CELL = BOARD_SIZE / 3;
const STATS_KEY = "andromeda.tictactoe.stats";

const COLORS = {
  bg: "#0b0d12",
  panel: "#13161d",
  panelBorder: "#1f2430",
  board: "#0f1217",
  cellHover: "#1c2029",
  grid: "#252a36",
  text: "#e7e9ee",
  textMuted: "#8b93a7",
  x: "#60a5fa",
  o: "#f472b6",
  winLine: "#fde047",
  green: "#22c55e",
  red: "#ef4444",
};

type Player = "X" | "O";
type Phase = "playing" | "won" | "draw";

interface Stats {
  wins: number;
  losses: number;
  draws: number;
}

interface GameState {
  board: (Player | null)[];
  turn: Player;
  phase: Phase;
  winner: Player | null;
  winLine: number[] | null;
  stats: Stats;
  aiThinking: number;
}

function loadStats(): Stats {
  try {
    const raw = localStorage.getItem(STATS_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (parsed && typeof parsed === "object") {
        return {
          wins: Number(parsed.wins) || 0,
          losses: Number(parsed.losses) || 0,
          draws: Number(parsed.draws) || 0,
        };
      }
    }
  } catch (_) {
    // ignore
  }
  return { wins: 0, losses: 0, draws: 0 };
}
function saveStats(s: Stats) {
  localStorage.setItem(STATS_KEY, JSON.stringify(s));
}

const LINES = [
  [0, 1, 2],
  [3, 4, 5],
  [6, 7, 8],
  [0, 3, 6],
  [1, 4, 7],
  [2, 5, 8],
  [0, 4, 8],
  [2, 4, 6],
];

function findWin(
  board: (Player | null)[],
): { player: Player; line: number[] } | null {
  for (const l of LINES) {
    const [a, b, c] = l;
    if (board[a] && board[a] === board[b] && board[b] === board[c]) {
      return { player: board[a]!, line: l };
    }
  }
  return null;
}

function isDraw(board: (Player | null)[]): boolean {
  return board.every((c) => c !== null);
}

function minimax(
  board: (Player | null)[],
  turn: Player,
  depth: number,
): number {
  const win = findWin(board);
  if (win) return win.player === "O" ? 10 - depth : depth - 10;
  if (isDraw(board)) return 0;
  let best = turn === "O" ? -Infinity : Infinity;
  for (let i = 0; i < 9; i++) {
    if (board[i] !== null) continue;
    board[i] = turn;
    const score = minimax(board, turn === "O" ? "X" : "O", depth + 1);
    board[i] = null;
    best = turn === "O" ? Math.max(best, score) : Math.min(best, score);
  }
  return best;
}

function aiMove(board: (Player | null)[]): number {
  let bestScore = -Infinity;
  let bestMove = -1;
  for (let i = 0; i < 9; i++) {
    if (board[i] !== null) continue;
    board[i] = "O";
    const score = minimax(board, "X", 0);
    board[i] = null;
    if (score > bestScore) {
      bestScore = score;
      bestMove = i;
    }
  }
  return bestMove;
}

function initialState(stats: Stats): GameState {
  return {
    board: new Array(9).fill(null),
    turn: "X",
    phase: "playing",
    winner: null,
    winLine: null,
    stats,
    aiThinking: 0,
  };
}

const win = Andromeda.createWindow({
  title: "Andromeda Tic-Tac-Toe",
  width: WIDTH,
  height: HEIGHT,
});
const canvas = new OffscreenCanvas(WIDTH, HEIGHT);
const ctx = canvas.getContext("2d")!;

let state: GameState = initialState(loadStats());
let hoverIdx = -1;
let resetHover = false;

function resetRect() {
  return {
    x: (WIDTH - 140) / 2,
    y: (HEADER_H - 36) / 2,
    w: 140,
    h: 36,
  };
}
function pointInRect(
  x: number,
  y: number,
  r: { x: number; y: number; w: number; h: number },
) {
  return x >= r.x && x <= r.x + r.w && y >= r.y && y <= r.y + r.h;
}
function cellIdxFromPos(px: number, py: number): number {
  const bx = PADDING;
  const by = HEADER_H;
  const gx = px - bx;
  const gy = py - by;
  if (gx < 0 || gy < 0 || gx >= BOARD_SIZE || gy >= BOARD_SIZE) return -1;
  return Math.floor(gy / CELL) * 3 + Math.floor(gx / CELL);
}

win.addEventListener("keydown", (e: any) => {
  if (e.detail.code === "Escape") win.close();
  if (e.detail.code === "KeyR") state = initialState(state.stats);
});

win.addEventListener("mousemove", (e: any) => {
  hoverIdx = cellIdxFromPos(e.detail.x, e.detail.y);
  resetHover = pointInRect(e.detail.x, e.detail.y, resetRect());
});

win.addEventListener("mousedown", (e: any) => {
  if (e.detail.button !== 0) return;
  if (pointInRect(e.detail.x, e.detail.y, resetRect())) {
    state = initialState(state.stats);
    return;
  }
  if (state.phase !== "playing" || state.turn !== "X") return;
  const idx = cellIdxFromPos(e.detail.x, e.detail.y);
  if (idx < 0 || state.board[idx] !== null) return;
  state.board[idx] = "X";
  resolveAfterMove();
});

function resolveAfterMove() {
  const win = findWin(state.board);
  if (win) {
    state.phase = "won";
    state.winner = win.player;
    state.winLine = win.line;
    if (win.player === "X") state.stats.wins++;
    else state.stats.losses++;
    saveStats(state.stats);
    return;
  }
  if (isDraw(state.board)) {
    state.phase = "draw";
    state.stats.draws++;
    saveStats(state.stats);
    return;
  }
  state.turn = state.turn === "X" ? "O" : "X";
  if (state.turn === "O") state.aiThinking = 20;
}

function update() {
  if (state.phase !== "playing" || state.turn !== "O") return;
  if (state.aiThinking > 0) {
    state.aiThinking--;
    return;
  }
  const move = aiMove(state.board);
  if (move < 0) return;
  state.board[move] = "O";
  resolveAfterMove();
}

function roundedRect(
  x: number,
  y: number,
  w: number,
  h: number,
  r: number,
  fill?: string,
  stroke?: string,
  sw = 1,
) {
  ctx.beginPath();
  ctx.roundRect(x, y, w, h, r);
  if (fill) {
    ctx.fillStyle = fill;
    ctx.fill();
  }
  if (stroke) {
    ctx.strokeStyle = stroke;
    ctx.lineWidth = sw;
    ctx.stroke();
  }
}

function drawHeader() {
  ctx.fillStyle = COLORS.panel;
  ctx.fillRect(0, 0, WIDTH, HEADER_H);
  ctx.fillStyle = COLORS.panelBorder;
  ctx.fillRect(0, HEADER_H - 1, WIDTH, 1);

  // Stats pill (left)
  const leftPill = { x: PADDING, y: (HEADER_H - 36) / 2, w: 80, h: 36 };
  roundedRect(
    leftPill.x,
    leftPill.y,
    leftPill.w,
    leftPill.h,
    10,
    COLORS.board,
    COLORS.panelBorder,
  );
  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "600 10px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText("W · D · L", leftPill.x + leftPill.w / 2, leftPill.y + 12);
  ctx.fillStyle = COLORS.text;
  ctx.font = "700 13px ui-monospace, monospace";
  ctx.fillText(
    `${state.stats.wins} · ${state.stats.draws} · ${state.stats.losses}`,
    leftPill.x + leftPill.w / 2,
    leftPill.y + 25,
  );

  // Reset button (center)
  const r = resetRect();
  const bg = resetHover ? "#252a36" : "#1c2029";
  roundedRect(r.x, r.y, r.w, r.h, 10, bg, COLORS.panelBorder);
  ctx.fillStyle = COLORS.text;
  ctx.font = "600 13px sans-serif";
  ctx.fillText("NEW GAME", r.x + r.w / 2, r.y + r.h / 2);

  // Turn indicator (right)
  const rightPill = {
    x: WIDTH - PADDING - 80,
    y: (HEADER_H - 36) / 2,
    w: 80,
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
  );
  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "600 10px sans-serif";
  ctx.fillText("TURN", rightPill.x + rightPill.w / 2, rightPill.y + 12);
  ctx.fillStyle = state.turn === "X" ? COLORS.x : COLORS.o;
  ctx.font = "700 16px sans-serif";
  ctx.fillText(
    state.phase === "playing"
      ? `${state.turn === "X" ? "YOU" : "AI"}`
      : state.phase === "draw"
      ? "DRAW"
      : state.winner === "X"
      ? "YOU"
      : "AI",
    rightPill.x + rightPill.w / 2,
    rightPill.y + 25,
  );
}

function drawBoard() {
  const bx = PADDING;
  const by = HEADER_H;
  roundedRect(
    bx,
    by,
    BOARD_SIZE,
    BOARD_SIZE,
    12,
    COLORS.board,
    COLORS.panelBorder,
  );

  ctx.strokeStyle = COLORS.grid;
  ctx.lineWidth = 2;
  ctx.beginPath();
  for (let i = 1; i < 3; i++) {
    ctx.moveTo(bx + i * CELL, by + 14);
    ctx.lineTo(bx + i * CELL, by + BOARD_SIZE - 14);
    ctx.moveTo(bx + 14, by + i * CELL);
    ctx.lineTo(bx + BOARD_SIZE - 14, by + i * CELL);
  }
  ctx.stroke();

  for (let i = 0; i < 9; i++) {
    const col = i % 3;
    const row = Math.floor(i / 3);
    const cx = bx + col * CELL + CELL / 2;
    const cy = by + row * CELL + CELL / 2;
    if (
      state.phase === "playing" &&
      state.turn === "X" &&
      hoverIdx === i &&
      state.board[i] === null
    ) {
      ctx.fillStyle = COLORS.cellHover;
      roundedRect(
        bx + col * CELL + 10,
        by + row * CELL + 10,
        CELL - 20,
        CELL - 20,
        10,
        COLORS.cellHover,
      );
    }
    const cell = state.board[i];
    if (cell === "X") {
      ctx.strokeStyle = COLORS.x;
      ctx.lineWidth = 6;
      ctx.lineCap = "round";
      const r = CELL / 3.4;
      ctx.beginPath();
      ctx.moveTo(cx - r, cy - r);
      ctx.lineTo(cx + r, cy + r);
      ctx.moveTo(cx + r, cy - r);
      ctx.lineTo(cx - r, cy + r);
      ctx.stroke();
    } else if (cell === "O") {
      ctx.strokeStyle = COLORS.o;
      ctx.lineWidth = 6;
      ctx.beginPath();
      ctx.arc(cx, cy, CELL / 3.4, 0, Math.PI * 2);
      ctx.stroke();
    }
  }

  if (state.winLine) {
    const [a, , c] = state.winLine;
    const ax = bx + (a % 3) * CELL + CELL / 2;
    const ay = by + Math.floor(a / 3) * CELL + CELL / 2;
    const cx = bx + (c % 3) * CELL + CELL / 2;
    const cy = by + Math.floor(c / 3) * CELL + CELL / 2;
    ctx.strokeStyle = COLORS.winLine;
    ctx.lineWidth = 5;
    ctx.lineCap = "round";
    ctx.beginPath();
    ctx.moveTo(ax, ay);
    ctx.lineTo(cx, cy);
    ctx.stroke();
  }
}

function drawFooter() {
  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "12px sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(
    state.phase === "playing"
      ? "click an empty cell — you play X, AI is unbeatable"
      : state.phase === "draw"
      ? "Draw · click NEW GAME or press R"
      : state.winner === "X"
      ? "You win! (the AI must have let you)"
      : "AI wins — try again",
    WIDTH / 2,
    HEIGHT - PADDING - 12,
  );
}

function render() {
  ctx.fillStyle = COLORS.bg;
  ctx.fillRect(0, 0, WIDTH, HEIGHT);
  drawHeader();
  drawBoard();
  drawFooter();
}

await Andromeda.Window.mainloop(() => {
  update();
  render();
  win.presentCanvas(canvas);
});
