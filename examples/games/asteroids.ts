const WIDTH = 720;
const HEIGHT = 540;
const HEADER_H = 56;
const PADDING = 16;
const PLAY_TOP = HEADER_H;

const SHIP_R = 11;
const ROT_SPEED = 4.0; // rad/sec
const THRUST = 220; // px/s^2
const FRICTION = 0.4; // velocity decay per second
const MAX_SPEED = 360;
const FIRE_COOLDOWN = 0.18;
const BULLET_SPEED = 460;
const BULLET_LIFE = 0.9;

const ROCK_BIG = 36;
const ROCK_MED = 22;
const ROCK_SML = 12;
const SCORE_BIG = 20;
const SCORE_MED = 50;
const SCORE_SML = 100;

const BEST_KEY = "andromeda.asteroids.best";

const COLORS = {
  bg: "#04060a",
  text: "#e7e9ee",
  textMuted: "#8b93a7",
  panel: "#0d121a",
  panelBorder: "#1f2532",
  ship: "#e7e9ee",
  thrust: "#fbbf24",
  bullet: "#fef3c7",
  rock: "#a8b3cf",
  red: "#ef4444",
  green: "#22c55e",
};

interface Vec {
  x: number;
  y: number;
}
interface Bullet extends Vec {
  vx: number;
  vy: number;
  life: number;
}
interface Rock extends Vec {
  vx: number;
  vy: number;
  r: number;
  shape: number[]; // radius offsets per vertex
  rot: number;
  drot: number;
}
type Phase = "playing" | "lost";

interface State {
  phase: Phase;
  score: number;
  best: number;
  lives: number;
  level: number;
  ship: Vec & { vx: number; vy: number; angle: number };
  invuln: number;
  thrusting: boolean;
  rotL: boolean;
  rotR: boolean;
  fireCooldown: number;
  bullets: Bullet[];
  rocks: Rock[];
  flash: number;
  hyperCooldown: number;
}

function loadBest(): number {
  const n = Number(localStorage.getItem(BEST_KEY));
  return Number.isFinite(n) && n >= 0 ? Math.floor(n) : 0;
}
function saveBest(n: number) {
  localStorage.setItem(BEST_KEY, String(n));
}

function shapeFor(verts = 12): number[] {
  const out: number[] = [];
  for (let i = 0; i < verts; i++) out.push(0.75 + Math.random() * 0.45);
  return out;
}

function spawnRocks(count: number, ship: Vec): Rock[] {
  const out: Rock[] = [];
  for (let i = 0; i < count; i++) {
    let x = 0, y = 0;
    do {
      x = Math.random() * WIDTH;
      y = PLAY_TOP + Math.random() * (HEIGHT - PLAY_TOP);
    } while (Math.hypot(x - ship.x, y - ship.y) < 140);
    const a = Math.random() * Math.PI * 2;
    const s = 30 + Math.random() * 70;
    out.push({
      x,
      y,
      vx: Math.cos(a) * s,
      vy: Math.sin(a) * s,
      r: ROCK_BIG,
      shape: shapeFor(),
      rot: Math.random() * Math.PI * 2,
      drot: (Math.random() - 0.5) * 1.2,
    });
  }
  return out;
}

function fresh(best: number, score = 0, lives = 3, level = 1): State {
  const ship = {
    x: WIDTH / 2,
    y: PLAY_TOP + (HEIGHT - PLAY_TOP) / 2,
    vx: 0,
    vy: 0,
    angle: -Math.PI / 2,
  };
  return {
    phase: "playing",
    score,
    best,
    lives,
    level,
    ship,
    invuln: 1.5,
    thrusting: false,
    rotL: false,
    rotR: false,
    fireCooldown: 0,
    bullets: [],
    rocks: spawnRocks(3 + level, ship),
    flash: 0,
    hyperCooldown: 0,
  };
}

const win = Andromeda.createWindow({
  title: "Andromeda Asteroids",
  width: WIDTH,
  height: HEIGHT,
});
const canvas = new OffscreenCanvas(WIDTH, HEIGHT);
const ctx = canvas.getContext("2d")!;
let state = fresh(loadBest());

win.addEventListener("keydown", (e: any) => {
  const c: string = e.detail.code;
  if (c === "Escape") return win.close();
  if (c === "KeyR") return (state = fresh(state.best));
  if (state.phase !== "playing") {
    if (c === "Space" || c === "Enter") state = fresh(state.best);
    return;
  }
  if (c === "ArrowLeft" || c === "KeyA") state.rotL = true;
  if (c === "ArrowRight" || c === "KeyD") state.rotR = true;
  if (c === "ArrowUp" || c === "KeyW") state.thrusting = true;
  if (c === "Space" && !e.detail.repeat) tryFire();
  if ((c === "ShiftLeft" || c === "ShiftRight") && !e.detail.repeat) {
    hyperspace();
  }
});
win.addEventListener("keyup", (e: any) => {
  const c: string = e.detail.code;
  if (c === "ArrowLeft" || c === "KeyA") state.rotL = false;
  if (c === "ArrowRight" || c === "KeyD") state.rotR = false;
  if (c === "ArrowUp" || c === "KeyW") state.thrusting = false;
});

function tryFire() {
  if (state.fireCooldown > 0) return;
  state.fireCooldown = FIRE_COOLDOWN;
  const dx = Math.cos(state.ship.angle);
  const dy = Math.sin(state.ship.angle);
  state.bullets.push({
    x: state.ship.x + dx * SHIP_R,
    y: state.ship.y + dy * SHIP_R,
    vx: state.ship.vx + dx * BULLET_SPEED,
    vy: state.ship.vy + dy * BULLET_SPEED,
    life: BULLET_LIFE,
  });
}

function hyperspace() {
  if (state.hyperCooldown > 0) return;
  state.hyperCooldown = 4;
  state.ship.x = Math.random() * WIDTH;
  state.ship.y = PLAY_TOP + Math.random() * (HEIGHT - PLAY_TOP);
  state.ship.vx = 0;
  state.ship.vy = 0;
  state.invuln = 1;
}

function wrap(p: Vec) {
  if (p.x < 0) p.x += WIDTH;
  if (p.x > WIDTH) p.x -= WIDTH;
  if (p.y < PLAY_TOP) p.y += HEIGHT - PLAY_TOP;
  if (p.y > HEIGHT) p.y -= HEIGHT - PLAY_TOP;
}

function splitRock(r: Rock): Rock[] {
  const child = r.r === ROCK_BIG ? ROCK_MED : r.r === ROCK_MED ? ROCK_SML : 0;
  if (!child) return [];
  const out: Rock[] = [];
  for (let i = 0; i < 2; i++) {
    const a = Math.atan2(r.vy, r.vx) + (Math.random() - 0.5) * 1.2;
    const s = Math.hypot(r.vx, r.vy) * (0.9 + Math.random() * 0.5);
    out.push({
      x: r.x,
      y: r.y,
      vx: Math.cos(a) * s,
      vy: Math.sin(a) * s,
      r: child,
      shape: shapeFor(8 + Math.floor(Math.random() * 4)),
      rot: Math.random() * Math.PI * 2,
      drot: (Math.random() - 0.5) * 1.6,
    });
  }
  return out;
}

function rockScore(r: Rock) {
  return r.r === ROCK_BIG
    ? SCORE_BIG
    : r.r === ROCK_MED
    ? SCORE_MED
    : SCORE_SML;
}

let last = Date.now();

function update() {
  const now = Date.now();
  const dt = Math.min(0.05, (now - last) / 1000);
  last = now;

  if (state.phase !== "playing") return;
  state.flash = Math.max(0, state.flash - dt);
  state.invuln = Math.max(0, state.invuln - dt);
  state.hyperCooldown = Math.max(0, state.hyperCooldown - dt);
  state.fireCooldown = Math.max(0, state.fireCooldown - dt);

  // Ship rotation + thrust.
  const s = state.ship;
  if (state.rotL) s.angle -= ROT_SPEED * dt;
  if (state.rotR) s.angle += ROT_SPEED * dt;
  if (state.thrusting) {
    s.vx += Math.cos(s.angle) * THRUST * dt;
    s.vy += Math.sin(s.angle) * THRUST * dt;
  }
  // Soft cap + friction.
  s.vx *= 1 - FRICTION * dt;
  s.vy *= 1 - FRICTION * dt;
  const sp = Math.hypot(s.vx, s.vy);
  if (sp > MAX_SPEED) {
    s.vx = (s.vx / sp) * MAX_SPEED;
    s.vy = (s.vy / sp) * MAX_SPEED;
  }
  s.x += s.vx * dt;
  s.y += s.vy * dt;
  wrap(s);

  // Bullets.
  for (const b of state.bullets) {
    b.x += b.vx * dt;
    b.y += b.vy * dt;
    b.life -= dt;
    wrap(b);
  }
  state.bullets = state.bullets.filter((b) => b.life > 0);

  // Rocks.
  for (const r of state.rocks) {
    r.x += r.vx * dt;
    r.y += r.vy * dt;
    r.rot += r.drot * dt;
    wrap(r);
  }

  // Bullet vs rock.
  const newRocks: Rock[] = [];
  outer: for (const r of state.rocks) {
    for (const b of state.bullets) {
      if (Math.hypot(b.x - r.x, b.y - r.y) < r.r) {
        b.life = -1;
        state.score += rockScore(r);
        newRocks.push(...splitRock(r));
        continue outer;
      }
    }
    newRocks.push(r);
  }
  state.rocks = newRocks;
  state.bullets = state.bullets.filter((b) => b.life > 0);

  // Ship vs rock.
  if (state.invuln <= 0) {
    for (const r of state.rocks) {
      if (Math.hypot(s.x - r.x, s.y - r.y) < r.r + SHIP_R - 2) {
        state.lives--;
        state.flash = 0.4;
        state.invuln = 1.8;
        s.vx = 0;
        s.vy = 0;
        s.x = WIDTH / 2;
        s.y = PLAY_TOP + (HEIGHT - PLAY_TOP) / 2;
        if (state.lives <= 0) {
          state.phase = "lost";
          if (state.score > state.best) {
            state.best = state.score;
            saveBest(state.best);
          }
        }
        break;
      }
    }
  }

  // Cleared field — next level.
  if (state.rocks.length === 0) {
    state = fresh(state.best, state.score, state.lives, state.level + 1);
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
  ctx.textAlign = "left";
  ctx.textBaseline = "middle";
  ctx.fillText("SCORE", PADDING, 20);
  ctx.fillText("LEVEL", WIDTH / 2 - 30, 20);
  ctx.fillText("BEST", WIDTH - PADDING - 80, 20);
  ctx.fillStyle = COLORS.text;
  ctx.font = "700 22px ui-monospace, monospace";
  ctx.fillText(String(state.score).padStart(5, "0"), PADDING, 40);
  ctx.fillText(String(state.level).padStart(2, "0"), WIDTH / 2 - 30, 40);
  ctx.fillText(String(state.best).padStart(5, "0"), WIDTH - PADDING - 80, 40);

  for (let i = 0; i < state.lives; i++) {
    drawShipIcon(WIDTH - PADDING - 20 - i * 22, 22);
  }
}

function drawShipIcon(cx: number, cy: number) {
  ctx.save();
  ctx.translate(cx, cy);
  ctx.rotate(-Math.PI / 2);
  ctx.strokeStyle = COLORS.text;
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  ctx.moveTo(8, 0);
  ctx.lineTo(-6, -5);
  ctx.lineTo(-3, 0);
  ctx.lineTo(-6, 5);
  ctx.closePath();
  ctx.stroke();
  ctx.restore();
}

function drawShip() {
  if (state.invuln > 0 && Math.floor(state.invuln * 10) % 2 === 0) return;
  ctx.save();
  ctx.translate(state.ship.x, state.ship.y);
  ctx.rotate(state.ship.angle);
  ctx.strokeStyle = COLORS.ship;
  ctx.lineWidth = 1.6;
  ctx.beginPath();
  ctx.moveTo(SHIP_R, 0);
  ctx.lineTo(-SHIP_R * 0.85, -SHIP_R * 0.7);
  ctx.lineTo(-SHIP_R * 0.55, 0);
  ctx.lineTo(-SHIP_R * 0.85, SHIP_R * 0.7);
  ctx.closePath();
  ctx.stroke();

  if (state.thrusting && Math.random() > 0.3) {
    ctx.strokeStyle = COLORS.thrust;
    ctx.beginPath();
    ctx.moveTo(-SHIP_R * 0.85, -SHIP_R * 0.35);
    ctx.lineTo(-SHIP_R * 1.5, 0);
    ctx.lineTo(-SHIP_R * 0.85, SHIP_R * 0.35);
    ctx.stroke();
  }
  ctx.restore();
}

function drawRock(r: Rock) {
  ctx.save();
  ctx.translate(r.x, r.y);
  ctx.rotate(r.rot);
  ctx.strokeStyle = COLORS.rock;
  ctx.lineWidth = 1.4;
  ctx.beginPath();
  for (let i = 0; i < r.shape.length; i++) {
    const a = (i / r.shape.length) * Math.PI * 2;
    const rr = r.r * r.shape[i];
    const x = Math.cos(a) * rr;
    const y = Math.sin(a) * rr;
    if (i === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  }
  ctx.closePath();
  ctx.stroke();
  ctx.restore();
}

function render() {
  ctx.fillStyle = COLORS.bg;
  ctx.fillRect(0, 0, WIDTH, HEIGHT);
  drawHeader();

  for (const r of state.rocks) drawRock(r);
  for (const b of state.bullets) {
    ctx.fillStyle = COLORS.bullet;
    ctx.fillRect(b.x - 1, b.y - 1, 2, 2);
  }
  drawShip();

  if (state.flash > 0) {
    ctx.fillStyle = `rgba(239,68,68,${(state.flash / 0.4) * 0.4})`;
    ctx.fillRect(0, PLAY_TOP, WIDTH, HEIGHT - PLAY_TOP);
  }

  if (state.phase !== "playing") {
    ctx.fillStyle = "rgba(4,6,10,0.7)";
    ctx.fillRect(0, PLAY_TOP, WIDTH, HEIGHT - PLAY_TOP);
    ctx.fillStyle = COLORS.red;
    ctx.font = "700 30px sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText("GAME OVER", WIDTH / 2, HEIGHT / 2 - 10);
    ctx.fillStyle = COLORS.textMuted;
    ctx.font = "13px sans-serif";
    ctx.fillText(
      "space to play again · R to reset · esc to quit",
      WIDTH / 2,
      HEIGHT / 2 + 22,
    );
  } else {
    ctx.fillStyle = COLORS.textMuted;
    ctx.font = "12px sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "alphabetic";
    ctx.fillText(
      "←/→ rotate · ↑ thrust · space fire · shift hyperspace",
      WIDTH / 2,
      HEIGHT - 8,
    );
  }
}

await Andromeda.Window.mainloop(() => {
  update();
  render();
  win.presentCanvas(canvas);
});
