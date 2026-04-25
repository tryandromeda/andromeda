const WIDTH = 640;
const HEIGHT = 480;
const PADDING = 20;
const HEADER_H = 60;
const PLAY_TOP = HEADER_H;
const PLAY_BOTTOM = HEIGHT - 30;

const COLS = 11;
const ROWS = 5;
const ALIEN_W = 32;
const ALIEN_H = 22;
const ALIEN_HGAP = 14;
const ALIEN_VGAP = 12;
const SWARM_TOP = PLAY_TOP + 48;
const SWARM_LEFT = PADDING + 24;

const PLAYER_W = 44;
const PLAYER_H = 16;
const PLAYER_Y = PLAY_BOTTOM - PLAYER_H - 8;
const PLAYER_SPEED = 320;

const BULLET_W = 3;
const BULLET_H = 12;
const BULLET_SPEED = 520;
const ALIEN_BULLET_SPEED = 220;
const FIRE_COOLDOWN = 0.35;
const ALIEN_FIRE_RATE = 0.7; // average bullets/sec from swarm

const SHIELD_COUNT = 4;
const SHIELD_W = 64;
const SHIELD_H = 32;
const SHIELD_CELL = 4;
const SHIELD_Y = PLAYER_Y - 56;

const BEST_KEY = "andromeda.invaders.best";

const COLORS = {
  bg: "#05070b",
  panel: "#0e131c",
  panelBorder: "#1f2532",
  text: "#e7e9ee",
  textMuted: "#8b93a7",
  alienA: "#22c55e",
  alienB: "#60a5fa",
  alienC: "#f59e0b",
  player: "#e7e9ee",
  playerBullet: "#fef08a",
  alienBullet: "#f87171",
  shield: "#34d399",
  shieldHit: "#65a30d",
  red: "#ef4444",
  green: "#22c55e",
};

type Phase = "playing" | "won" | "lost";

interface Alien {
  col: number;
  row: number;
  alive: boolean;
}
interface Bullet {
  x: number;
  y: number;
  dy: number;
  fromPlayer: boolean;
}
interface Shield {
  x: number;
  y: number;
  cells: boolean[][]; // rows of width-cells
}
interface State {
  phase: Phase;
  score: number;
  best: number;
  lives: number;
  wave: number;
  playerX: number;
  leftDown: boolean;
  rightDown: boolean;
  fireCooldown: number;
  bullets: Bullet[];
  aliens: Alien[];
  swarmDir: 1 | -1;
  swarmDx: number; // accumulated x displacement
  swarmDy: number;
  swarmStepTimer: number;
  swarmStepInterval: number;
  alienFireTimer: number;
  shields: Shield[];
  flashTimer: number;
}

function loadBest(): number {
  const n = Number(localStorage.getItem(BEST_KEY));
  return Number.isFinite(n) && n >= 0 ? Math.floor(n) : 0;
}
function saveBest(n: number) {
  localStorage.setItem(BEST_KEY, String(n));
}

function newSwarm(): Alien[] {
  const a: Alien[] = [];
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      a.push({ col: c, row: r, alive: true });
    }
  }
  return a;
}

function newShield(x: number, y: number): Shield {
  const w = Math.floor(SHIELD_W / SHIELD_CELL);
  const h = Math.floor(SHIELD_H / SHIELD_CELL);
  const cells: boolean[][] = [];
  for (let yi = 0; yi < h; yi++) {
    const row: boolean[] = [];
    for (let xi = 0; xi < w; xi++) {
      // Notch a triangle out of the bottom-center for an arch silhouette.
      const cx = w / 2;
      const archDist = Math.hypot((xi - cx) / cx, (yi - h * 0.6) / (h * 0.5));
      const inArch = yi > h * 0.55 && Math.abs(xi - cx) < (h - yi) * 0.7;
      row.push(!inArch && archDist > 0.05);
    }
    cells.push(row);
  }
  return { x, y, cells };
}

function newShields(): Shield[] {
  const out: Shield[] = [];
  const total = SHIELD_COUNT;
  const span = WIDTH - PADDING * 2 - SHIELD_W;
  for (let i = 0; i < total; i++) {
    const x = PADDING + (span * i) / (total - 1);
    out.push(newShield(x, SHIELD_Y));
  }
  return out;
}

function fresh(best: number, wave = 1, score = 0, lives = 3): State {
  return {
    phase: "playing",
    score,
    best,
    lives,
    wave,
    playerX: WIDTH / 2 - PLAYER_W / 2,
    leftDown: false,
    rightDown: false,
    fireCooldown: 0,
    bullets: [],
    aliens: newSwarm(),
    swarmDir: 1,
    swarmDx: 0,
    swarmDy: 0,
    swarmStepTimer: 0,
    swarmStepInterval: Math.max(0.12, 0.7 - wave * 0.07),
    alienFireTimer: 0,
    shields: newShields(),
    flashTimer: 0,
  };
}

const win = Andromeda.createWindow({
  title: "Andromeda Invaders",
  width: WIDTH,
  height: HEIGHT,
});
const canvas = new OffscreenCanvas(WIDTH, HEIGHT);
const ctx = canvas.getContext("2d")!;
let state = fresh(loadBest());

win.addEventListener("keydown", (e: any) => {
  const c: string = e.detail.code;
  if (c === "Escape") return win.close();
  if (c === "KeyR") state = fresh(state.best, 1, 0, 3);
  if (state.phase !== "playing") {
    if (c === "Space" || c === "Enter") state = fresh(state.best, 1, 0, 3);
    return;
  }
  if (c === "ArrowLeft" || c === "KeyA") state.leftDown = true;
  if (c === "ArrowRight" || c === "KeyD") state.rightDown = true;
  if ((c === "Space" || c === "KeyW" || c === "ArrowUp") && !e.detail.repeat) {
    tryFire();
  }
});
win.addEventListener("keyup", (e: any) => {
  const c: string = e.detail.code;
  if (c === "ArrowLeft" || c === "KeyA") state.leftDown = false;
  if (c === "ArrowRight" || c === "KeyD") state.rightDown = false;
});

function tryFire() {
  if (state.fireCooldown > 0) return;
  state.fireCooldown = FIRE_COOLDOWN;
  state.bullets.push({
    x: state.playerX + PLAYER_W / 2 - BULLET_W / 2,
    y: PLAYER_Y - BULLET_H,
    dy: -BULLET_SPEED,
    fromPlayer: true,
  });
}

function alienXY(a: Alien): { x: number; y: number } {
  return {
    x: SWARM_LEFT + a.col * (ALIEN_W + ALIEN_HGAP) + state.swarmDx,
    y: SWARM_TOP + a.row * (ALIEN_H + ALIEN_VGAP) + state.swarmDy,
  };
}

function alienColor(row: number) {
  return row === 0 ? COLORS.alienC : row < 3 ? COLORS.alienB : COLORS.alienA;
}

function alienScore(row: number) {
  return row === 0 ? 30 : row < 3 ? 20 : 10;
}

function alive() {
  return state.aliens.filter((a) => a.alive);
}

function bottomMostByCol(): Alien[] {
  const map = new Map<number, Alien>();
  for (const a of alive()) {
    const cur = map.get(a.col);
    if (!cur || a.row > cur.row) map.set(a.col, a);
  }
  return [...map.values()];
}

function rectsHit(
  a: { x: number; y: number; w: number; h: number },
  b: { x: number; y: number; w: number; h: number },
) {
  return a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h &&
    a.y + a.h > b.y;
}

function damageShield(b: Bullet): boolean {
  for (const s of state.shields) {
    if (b.x + BULLET_W < s.x || b.x > s.x + SHIELD_W) continue;
    if (b.y + BULLET_H < s.y || b.y > s.y + SHIELD_H) continue;
    const xi = Math.floor((b.x + BULLET_W / 2 - s.x) / SHIELD_CELL);
    const yi = Math.floor(
      (b.y + (b.dy < 0 ? 0 : BULLET_H) - s.y) / SHIELD_CELL,
    );
    const inside = xi >= 0 && yi >= 0 && yi < s.cells.length &&
      xi < s.cells[0].length;
    if (!inside) continue;
    // Carve out a small splotch.
    let hit = false;
    for (let dy = -1; dy <= 1; dy++) {
      for (let dx = -1; dx <= 1; dx++) {
        const x = xi + dx;
        const y = yi + dy;
        if (x < 0 || y < 0 || y >= s.cells.length || x >= s.cells[0].length) {
          continue;
        }
        if (s.cells[y][x]) {
          s.cells[y][x] = false;
          hit = true;
        }
      }
    }
    if (hit) return true;
  }
  return false;
}

let last = Date.now();

function update() {
  const now = Date.now();
  const dt = Math.min(0.05, (now - last) / 1000);
  last = now;
  if (state.flashTimer > 0) {
    state.flashTimer = Math.max(0, state.flashTimer - dt);
  }
  if (state.phase !== "playing") return;

  // Player movement.
  const dir = (state.rightDown ? 1 : 0) - (state.leftDown ? 1 : 0);
  state.playerX = Math.max(
    PADDING,
    Math.min(
      WIDTH - PADDING - PLAYER_W,
      state.playerX + dir * PLAYER_SPEED * dt,
    ),
  );
  state.fireCooldown = Math.max(0, state.fireCooldown - dt);

  // Swarm step (discrete, faster as fewer aliens remain).
  const aliveCount = alive().length;
  state.swarmStepInterval = Math.max(
    0.08,
    0.55 * (aliveCount / (COLS * ROWS)) + 0.08 - state.wave * 0.02,
  );
  state.swarmStepTimer += dt;
  if (state.swarmStepTimer >= state.swarmStepInterval) {
    state.swarmStepTimer = 0;
    const step = 8 * state.swarmDir;
    state.swarmDx += step;
    // Wall check using current alien xs.
    let minX = Infinity, maxX = -Infinity;
    for (const a of alive()) {
      const { x } = alienXY(a);
      if (x < minX) minX = x;
      if (x + ALIEN_W > maxX) maxX = x + ALIEN_W;
    }
    if (minX < PADDING || maxX > WIDTH - PADDING) {
      state.swarmDx -= step;
      state.swarmDir = -state.swarmDir as 1 | -1;
      state.swarmDy += 14;
    }
  }

  // Aliens reach the player line — game over.
  const lowest = alive().reduce(
    (m, a) => Math.max(m, alienXY(a).y + ALIEN_H),
    0,
  );
  if (lowest >= PLAYER_Y) {
    state.phase = "lost";
    return;
  }

  // Aliens fire.
  state.alienFireTimer += dt;
  const fireInterval = 1 / (ALIEN_FIRE_RATE + state.wave * 0.15);
  while (state.alienFireTimer >= fireInterval) {
    state.alienFireTimer -= fireInterval;
    const bottom = bottomMostByCol();
    if (bottom.length > 0) {
      const a = bottom[Math.floor(Math.random() * bottom.length)];
      const { x, y } = alienXY(a);
      state.bullets.push({
        x: x + ALIEN_W / 2 - BULLET_W / 2,
        y: y + ALIEN_H,
        dy: ALIEN_BULLET_SPEED,
        fromPlayer: false,
      });
    }
  }

  // Bullets.
  for (const b of state.bullets) b.y += b.dy * dt;
  state.bullets = state.bullets.filter((b) =>
    b.y + BULLET_H > PLAY_TOP && b.y < PLAY_BOTTOM
  );

  // Player bullets vs aliens.
  for (const b of state.bullets) {
    if (!b.fromPlayer) continue;
    for (const a of state.aliens) {
      if (!a.alive) continue;
      const { x, y } = alienXY(a);
      if (
        rectsHit({ x: b.x, y: b.y, w: BULLET_W, h: BULLET_H }, {
          x,
          y,
          w: ALIEN_W,
          h: ALIEN_H,
        })
      ) {
        a.alive = false;
        b.y = -1000; // mark for removal
        state.score += alienScore(a.row);
      }
    }
  }

  // Bullets vs shields.
  state.bullets = state.bullets.filter((b) => !damageShield(b));

  // Alien bullets vs player.
  for (const b of state.bullets) {
    if (b.fromPlayer) continue;
    if (
      rectsHit({ x: b.x, y: b.y, w: BULLET_W, h: BULLET_H }, {
        x: state.playerX,
        y: PLAYER_Y,
        w: PLAYER_W,
        h: PLAYER_H,
      })
    ) {
      b.y = 1e6;
      state.lives--;
      state.flashTimer = 0.4;
      if (state.lives <= 0) state.phase = "lost";
    }
  }
  state.bullets = state.bullets.filter((b) =>
    b.y + BULLET_H > PLAY_TOP && b.y < PLAY_BOTTOM
  );

  // Wave clear.
  if (alive().length === 0) {
    if (state.score > state.best) {
      state.best = state.score;
      saveBest(state.best);
    }
    state = fresh(state.best, state.wave + 1, state.score, state.lives);
  }

  if (state.score > state.best) {
    state.best = state.score;
    saveBest(state.best);
  }
}

// --- Render ----------------------------------------------------------------

function drawHeader() {
  ctx.fillStyle = COLORS.panel;
  ctx.fillRect(0, 0, WIDTH, HEADER_H);
  ctx.fillStyle = COLORS.panelBorder;
  ctx.fillRect(0, HEADER_H - 1, WIDTH, 1);
  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "600 11px sans-serif";
  ctx.textBaseline = "middle";
  ctx.textAlign = "left";
  ctx.fillText("SCORE", PADDING, 22);
  ctx.fillText("WAVE", WIDTH / 2 - 30, 22);
  ctx.fillText("BEST", WIDTH - PADDING - 80, 22);
  ctx.fillStyle = COLORS.text;
  ctx.font = "700 22px ui-monospace, monospace";
  ctx.fillText(String(state.score).padStart(5, "0"), PADDING, 44);
  ctx.fillText(String(state.wave).padStart(2, "0"), WIDTH / 2 - 30, 44);
  ctx.fillText(String(state.best).padStart(5, "0"), WIDTH - PADDING - 80, 44);
}

// Lives sit in the footer row below the play floor — classic arcade
// layout, and out of the way of the BEST score column.
function drawLives() {
  const y = PLAY_BOTTOM + 8;
  ctx.fillStyle = COLORS.textMuted;
  ctx.font = "600 11px sans-serif";
  ctx.textAlign = "left";
  ctx.textBaseline = "middle";
  ctx.fillText("LIVES", PADDING, y + 5);
  for (let i = 0; i < state.lives; i++) {
    drawShipShape(PADDING + 50 + i * 30, y, 22, 10, COLORS.player);
  }
}

function drawShipShape(
  x: number,
  y: number,
  w: number,
  h: number,
  color: string,
) {
  ctx.fillStyle = color;
  ctx.fillRect(x, y + h - 4, w, 4);
  ctx.fillRect(x + w / 2 - 3, y, 6, h);
  ctx.fillRect(x + 4, y + h - 8, w - 8, 4);
}

function drawAlien(a: Alien) {
  const { x, y } = alienXY(a);
  ctx.fillStyle = alienColor(a.row);
  // body
  ctx.fillRect(x + 4, y + 4, ALIEN_W - 8, ALIEN_H - 8);
  // eyes
  ctx.fillStyle = COLORS.bg;
  ctx.fillRect(x + 9, y + 9, 4, 4);
  ctx.fillRect(x + ALIEN_W - 13, y + 9, 4, 4);
  // legs (alternate frame based on swarm dir for shimmy)
  const phase = (Math.floor(state.swarmDx / 8) + a.col) % 2 === 0;
  ctx.fillStyle = alienColor(a.row);
  ctx.fillRect(x + (phase ? 2 : 6), y + ALIEN_H - 4, 4, 4);
  ctx.fillRect(x + ALIEN_W - (phase ? 6 : 10), y + ALIEN_H - 4, 4, 4);
}

function drawShield(s: Shield) {
  for (let yi = 0; yi < s.cells.length; yi++) {
    for (let xi = 0; xi < s.cells[yi].length; xi++) {
      if (!s.cells[yi][xi]) continue;
      ctx.fillStyle = COLORS.shield;
      ctx.fillRect(
        s.x + xi * SHIELD_CELL,
        s.y + yi * SHIELD_CELL,
        SHIELD_CELL,
        SHIELD_CELL,
      );
    }
  }
}

function drawBullet(b: Bullet) {
  ctx.fillStyle = b.fromPlayer ? COLORS.playerBullet : COLORS.alienBullet;
  ctx.fillRect(b.x, b.y, BULLET_W, BULLET_H);
}

function render() {
  ctx.fillStyle = COLORS.bg;
  ctx.fillRect(0, 0, WIDTH, HEIGHT);

  drawHeader();

  // Floor line.
  ctx.fillStyle = COLORS.panelBorder;
  ctx.fillRect(0, PLAY_BOTTOM, WIDTH, 1);

  for (const a of state.aliens) if (a.alive) drawAlien(a);
  for (const s of state.shields) drawShield(s);
  for (const b of state.bullets) drawBullet(b);

  // Player.
  if (state.flashTimer % 0.2 < 0.1) {
    drawShipShape(state.playerX, PLAYER_Y, PLAYER_W, PLAYER_H, COLORS.player);
  }

  drawLives();

  // Overlay.
  if (state.phase !== "playing") {
    ctx.fillStyle = "rgba(5,7,11,0.7)";
    ctx.fillRect(0, HEADER_H, WIDTH, HEIGHT - HEADER_H);
    ctx.fillStyle = state.phase === "won" ? COLORS.green : COLORS.red;
    ctx.font = "700 28px sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(
      state.phase === "won" ? "YOU WIN" : "GAME OVER",
      WIDTH / 2,
      HEIGHT / 2 - 10,
    );
    ctx.fillStyle = COLORS.textMuted;
    ctx.font = "13px sans-serif";
    ctx.fillText(
      "Space to play again · R to reset · Esc to quit",
      WIDTH / 2,
      HEIGHT / 2 + 24,
    );
  } else {
    ctx.fillStyle = COLORS.textMuted;
    ctx.font = "12px sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "alphabetic";
    ctx.fillText(
      "←/→ move · space fire · R reset · Esc quit",
      WIDTH / 2,
      HEIGHT - 10,
    );
  }
}

await Andromeda.Window.mainloop(() => {
  update();
  render();
  win.presentCanvas(canvas);
});
