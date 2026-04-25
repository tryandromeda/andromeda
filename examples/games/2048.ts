const SIZE = 4;
const CELL = 110;
const GAP = 12;
const BOARD_PAD = 12;
const BOARD_W = SIZE * CELL + (SIZE + 1) * GAP;
const WIDTH = BOARD_W + BOARD_PAD * 2;
const HEIGHT = WIDTH + 90;
const BOARD_OFFSET_X = BOARD_PAD;
const BOARD_OFFSET_Y = 80;

const ANIM_FRAMES = 8;
const BEST_KEY = "andromeda.2048.best";

function loadBest(): number {
  const raw = localStorage.getItem(BEST_KEY);
  if (!raw) return 0;
  const n = Number(raw);
  return Number.isFinite(n) && n >= 0 ? n : 0;
}

function saveBest(best: number) {
  localStorage.setItem(BEST_KEY, String(best));
}

type Board = number[]; // flat SIZE*SIZE
type Phase = "playing" | "won" | "game-over";

interface GameState {
  board: Board;
  score: number;
  best: number;
  phase: Phase;
  wonAcknowledged: boolean;
  animFrame: number;
}

const TILE_COLORS: Record<number, { bg: string; fg: string }> = {
  0: { bg: "#cdc1b4", fg: "#776e65" },
  2: { bg: "#eee4da", fg: "#776e65" },
  4: { bg: "#ede0c8", fg: "#776e65" },
  8: { bg: "#f2b179", fg: "#f9f6f2" },
  16: { bg: "#f59563", fg: "#f9f6f2" },
  32: { bg: "#f67c5f", fg: "#f9f6f2" },
  64: { bg: "#f65e3b", fg: "#f9f6f2" },
  128: { bg: "#edcf72", fg: "#f9f6f2" },
  256: { bg: "#edcc61", fg: "#f9f6f2" },
  512: { bg: "#edc850", fg: "#f9f6f2" },
  1024: { bg: "#edc53f", fg: "#f9f6f2" },
  2048: { bg: "#edc22e", fg: "#f9f6f2" },
};

function tileColors(v: number): { bg: string; fg: string } {
  return TILE_COLORS[v] ?? { bg: "#3c3a32", fg: "#f9f6f2" };
}

function emptyBoard(): Board {
  return new Array(SIZE * SIZE).fill(0);
}

function addRandomTile(b: Board) {
  const empty: number[] = [];
  for (let i = 0; i < b.length; i++) if (b[i] === 0) empty.push(i);
  if (empty.length === 0) return;
  const idx = empty[Math.floor(Math.random() * empty.length)];
  b[idx] = Math.random() < 0.9 ? 2 : 4;
}

function cloneBoard(b: Board): Board {
  return b.slice();
}

function boardsEqual(a: Board, b: Board): boolean {
  for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return false;
  return true;
}

function slideRow(row: number[]): { row: number[]; gained: number } {
  const filtered = row.filter((v) => v !== 0);
  let gained = 0;
  for (let i = 0; i < filtered.length - 1; i++) {
    if (filtered[i] === filtered[i + 1]) {
      filtered[i] *= 2;
      gained += filtered[i];
      filtered.splice(i + 1, 1);
    }
  }
  while (filtered.length < SIZE) filtered.push(0);
  return { row: filtered, gained };
}

function getRow(b: Board, y: number): number[] {
  const out: number[] = [];
  for (let x = 0; x < SIZE; x++) out.push(b[y * SIZE + x]);
  return out;
}

function setRow(b: Board, y: number, row: number[]) {
  for (let x = 0; x < SIZE; x++) b[y * SIZE + x] = row[x];
}

function getCol(b: Board, x: number): number[] {
  const out: number[] = [];
  for (let y = 0; y < SIZE; y++) out.push(b[y * SIZE + x]);
  return out;
}

function setCol(b: Board, x: number, col: number[]) {
  for (let y = 0; y < SIZE; y++) b[y * SIZE + x] = col[y];
}

type Dir = "left" | "right" | "up" | "down";

function move(
  b: Board,
  dir: Dir,
): { board: Board; moved: boolean; gained: number } {
  const next = cloneBoard(b);
  let gained = 0;
  if (dir === "left") {
    for (let y = 0; y < SIZE; y++) {
      const r = slideRow(getRow(next, y));
      setRow(next, y, r.row);
      gained += r.gained;
    }
  } else if (dir === "right") {
    for (let y = 0; y < SIZE; y++) {
      const r = slideRow(getRow(next, y).reverse());
      setRow(next, y, r.row.reverse());
      gained += r.gained;
    }
  } else if (dir === "up") {
    for (let x = 0; x < SIZE; x++) {
      const r = slideRow(getCol(next, x));
      setCol(next, x, r.row);
      gained += r.gained;
    }
  } else {
    for (let x = 0; x < SIZE; x++) {
      const r = slideRow(getCol(next, x).reverse());
      setCol(next, x, r.row.reverse());
      gained += r.gained;
    }
  }
  return { board: next, moved: !boardsEqual(b, next), gained };
}

function anyMoveAvailable(b: Board): boolean {
  if (b.some((v) => v === 0)) return true;
  for (let y = 0; y < SIZE; y++) {
    for (let x = 0; x < SIZE; x++) {
      const v = b[y * SIZE + x];
      if (x + 1 < SIZE && b[y * SIZE + x + 1] === v) return true;
      if (y + 1 < SIZE && b[(y + 1) * SIZE + x] === v) return true;
    }
  }
  return false;
}

function initialState(best: number): GameState {
  const board = emptyBoard();
  addRandomTile(board);
  addRandomTile(board);
  return {
    board,
    score: 0,
    best,
    phase: "playing",
    wonAcknowledged: false,
    animFrame: 0,
  };
}

const win = Andromeda.createWindow({
  title: "Andromeda 2048",
  width: WIDTH,
  height: HEIGHT,
});
console.log(`window ${win.rid} (${win.rawHandle().system})`);
const canvas = new OffscreenCanvas(WIDTH, HEIGHT);
const ctx = canvas.getContext("2d")!;

let state: GameState = initialState(loadBest());

function attemptMove(dir: Dir) {
  if (state.phase === "game-over") return;
  if (state.phase === "won" && !state.wonAcknowledged) return;
  const result = move(state.board, dir);
  if (!result.moved) return;
  state.board = result.board;
  state.score += result.gained;
  if (state.score > state.best) {
    state.best = state.score;
    saveBest(state.best);
  }
  addRandomTile(state.board);
  state.animFrame = ANIM_FRAMES;
  if (!state.wonAcknowledged && state.board.some((v) => v === 2048)) {
    state.phase = "won";
  } else if (!anyMoveAvailable(state.board)) {
    state.phase = "game-over";
  }
}

win.addEventListener("keydown", (e: any) => {
  const code: string = e.detail.code;
  if (code === "Escape") {
    win.close();
    return;
  }
  if (code === "KeyR") {
    state = initialState(state.best);
    return;
  }
  if (state.phase === "won" && code === "Space") {
    state.wonAcknowledged = true;
    state.phase = "playing";
    return;
  }
  if (state.phase === "game-over" && code === "Space") {
    state = initialState(state.best);
    return;
  }
  if (code === "ArrowLeft" || code === "KeyA") attemptMove("left");
  else if (code === "ArrowRight" || code === "KeyD") attemptMove("right");
  else if (code === "ArrowUp" || code === "KeyW") attemptMove("up");
  else if (code === "ArrowDown" || code === "KeyS") attemptMove("down");
});

function drawTile(x: number, y: number, v: number) {
  const px = BOARD_OFFSET_X + GAP + x * (CELL + GAP);
  const py = BOARD_OFFSET_Y + GAP + y * (CELL + GAP);
  const { bg, fg } = tileColors(v);

  // Pop animation on newly-moved tiles (subtle — whole board uses the frame).
  let scale = 1;
  if (v !== 0 && state.animFrame > 0) {
    const t = state.animFrame / ANIM_FRAMES;
    scale = 1 - 0.08 * t;
  }
  const pad = (CELL * (1 - scale)) / 2;

  ctx.fillStyle = bg;
  ctx.beginPath();
  if (ctx.roundRect) {
    ctx.roundRect(px + pad, py + pad, CELL * scale, CELL * scale, 8);
    ctx.fill();
  } else {
    ctx.fillRect(px + pad, py + pad, CELL * scale, CELL * scale);
  }

  if (v === 0) return;

  ctx.fillStyle = fg;
  const text = String(v);
  const size = text.length >= 4 ? 30 : text.length === 3 ? 38 : 48;
  ctx.font = `bold ${size}px sans-serif`;
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(text, px + CELL / 2, py + CELL / 2 + 2);
}

function render() {
  if (state.animFrame > 0) state.animFrame--;

  ctx.fillStyle = "#faf8ef";
  ctx.fillRect(0, 0, WIDTH, HEIGHT);

  // Title + score boxes.
  ctx.fillStyle = "#776e65";
  ctx.font = "bold 38px sans-serif";
  ctx.textAlign = "left";
  ctx.textBaseline = "alphabetic";
  ctx.fillText("2048", 14, 48);

  const boxW = 84;
  const boxH = 52;
  const scoreX = WIDTH - boxW * 2 - 18;
  const bestX = WIDTH - boxW - 10;

  ctx.fillStyle = "#bbada0";
  ctx.fillRect(scoreX, 12, boxW, boxH);
  ctx.fillRect(bestX, 12, boxW, boxH);
  ctx.fillStyle = "#eee4da";
  ctx.font = "11px sans-serif";
  ctx.textAlign = "center";
  ctx.fillText("SCORE", scoreX + boxW / 2, 28);
  ctx.fillText("BEST", bestX + boxW / 2, 28);
  ctx.fillStyle = "#ffffff";
  ctx.font = "bold 20px sans-serif";
  ctx.fillText(String(state.score), scoreX + boxW / 2, 54);
  ctx.fillText(String(state.best), bestX + boxW / 2, 54);

  // Board background.
  ctx.fillStyle = "#bbada0";
  ctx.fillRect(BOARD_OFFSET_X, BOARD_OFFSET_Y, BOARD_W, BOARD_W);

  for (let y = 0; y < SIZE; y++) {
    for (let x = 0; x < SIZE; x++) {
      drawTile(x, y, state.board[y * SIZE + x]);
    }
  }

  ctx.textBaseline = "alphabetic";
  ctx.fillStyle = "rgba(119, 110, 101, 0.7)";
  ctx.font = "12px sans-serif";
  ctx.textAlign = "center";
  ctx.fillText("arrows / WASD to move    R to reset", WIDTH / 2, HEIGHT - 14);

  if (state.phase === "game-over") {
    ctx.fillStyle = "rgba(238, 228, 218, 0.75)";
    ctx.fillRect(BOARD_OFFSET_X, BOARD_OFFSET_Y, BOARD_W, BOARD_W);
    ctx.fillStyle = "#776e65";
    ctx.font = "bold 48px sans-serif";
    ctx.fillText("Game Over", WIDTH / 2, BOARD_OFFSET_Y + BOARD_W / 2 - 10);
    ctx.font = "18px sans-serif";
    ctx.fillText(
      "press Space to restart",
      WIDTH / 2,
      BOARD_OFFSET_Y + BOARD_W / 2 + 24,
    );
  } else if (state.phase === "won") {
    ctx.fillStyle = "rgba(237, 194, 46, 0.75)";
    ctx.fillRect(BOARD_OFFSET_X, BOARD_OFFSET_Y, BOARD_W, BOARD_W);
    ctx.fillStyle = "#f9f6f2";
    ctx.font = "bold 48px sans-serif";
    ctx.fillText("You Win!", WIDTH / 2, BOARD_OFFSET_Y + BOARD_W / 2 - 10);
    ctx.font = "18px sans-serif";
    ctx.fillText(
      "Space to keep going",
      WIDTH / 2,
      BOARD_OFFSET_Y + BOARD_W / 2 + 24,
    );
  }
}

let frameCount = 0;
const fpsStart = Date.now();
await Andromeda.Window.mainloop(() => {
  frameCount++;
  render();
  win.presentCanvas(canvas);
  if (frameCount % 240 === 0) {
    const elapsed = (Date.now() - fpsStart) / 1000;
    console.log(
      `2048: frame ${frameCount} — avg fps ${(frameCount / elapsed).toFixed(1)}`,
    );
  }
});
