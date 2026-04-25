// deno-lint-ignore-file no-explicit-any
const WIDTH = 600;
const HEIGHT = 150;
const FPS = 60;

const INITIAL_SPEED = 6;
const MAX_SPEED = 13;
const ACCELERATION = 0.001;
const GRAVITY = 0.6;
const INITIAL_JUMP_VELOCITY = -10;
const DROP_VELOCITY = -5;
const SPEED_DROP_COEFFICIENT = 3;
const MAX_JUMP_HEIGHT = 30;
const MIN_JUMP_HEIGHT = 30;

const DISTANCE_COEFFICIENT = 0.025;
const ACHIEVEMENT_DISTANCE = 100;
const SCORE_FLASH_DURATION = 250;
const SCORE_FLASH_ITERATIONS = 3;
const INVERT_DISTANCE = 700;
const INVERT_FADE_DURATION = 12000;

const BOTTOM_PAD = 10;
const TREX_START_X = 50;

const CLOUD_FREQUENCY = 0.5;
const MAX_CLOUDS = 6;
const BG_CLOUD_SPEED = 0.2;
const MIN_CLOUD_GAP = 100;
const MAX_CLOUD_GAP = 400;
const MIN_SKY_LEVEL = 71;
const MAX_SKY_LEVEL = 30;

const TREX_W = 44;
const TREX_H = 47;
const TREX_DUCK_W = 59;
const TREX_GROUND_Y = HEIGHT - TREX_H - BOTTOM_PAD;
const MIN_JUMP_Y = TREX_GROUND_Y - MIN_JUMP_HEIGHT;
const MAX_JUMP_Y = TREX_GROUND_Y - MAX_JUMP_HEIGHT;

const HORIZON_W = 1200;
const HORIZON_H = 12;
const HORIZON_Y = 127;

const DIGIT_W = 10;
const DIGIT_H = 13;
const SCORE_DIGITS = 5;
const SCORE_PAD = 11;
const SCORE_Y = 5;

const MOON_W = 20;
const MOON_H = 40;
const MOON_Y = 30;
const MOON_SPEED = 0.25;
const STAR_SIZE = 9;
const STAR_SPEED = 0.3;
const STAR_MAX_Y = 70;
const NUM_STARS = 4;
const NIGHT_FADE_SPEED = 0.035;

const BEST_KEY = "andromeda.dino.best";

const sprites = {
  dinoIdle: createImageBitmap("./assets/dino_idle.png"),
  dinoBlink: createImageBitmap("./assets/dino_blink.png"),
  dinoWalk1: createImageBitmap("./assets/dino_walk.png"),
  dinoWalk2: createImageBitmap("./assets/dino_walk2.png"),
  dinoDuck1: createImageBitmap("./assets/dino_duck.png"),
  dinoDuck2: createImageBitmap("./assets/dino_duck2.png"),
  dinoCrash: createImageBitmap("./assets/dino_crash.png"),
  cactusSmall: createImageBitmap("./assets/cactus1.png"),
  cactusSmallDouble: createImageBitmap("./assets/cactus_small_double.png"),
  cactusSmallTriple: createImageBitmap("./assets/cactus_small_triple.png"),
  cactusLarge: createImageBitmap("./assets/cactus2.png"),
  cactusLargeDouble: createImageBitmap("./assets/cactus_large_double.png"),
  cactusLargeTriple: createImageBitmap("./assets/cactus_large_triple.png"),
  birdIdle: createImageBitmap("./assets/bird_idle.png"),
  birdFlap: createImageBitmap("./assets/bird_flap.png"),
  cloud: createImageBitmap("./assets/cloud.png"),
  horizon: createImageBitmap("./assets/horizon.png"),
  moon0: createImageBitmap("./assets/moon_0.png"),
  moon1: createImageBitmap("./assets/moon_1.png"),
  moon2: createImageBitmap("./assets/moon_2.png"),
  moon3: createImageBitmap("./assets/moon_3.png"),
  moon4: createImageBitmap("./assets/moon_4.png"),
  moon5: createImageBitmap("./assets/moon_5.png"),
  moon6: createImageBitmap("./assets/moon_6.png"),
  star: createImageBitmap("./assets/star.png"),
  digit0: createImageBitmap("./assets/digit_0.png"),
  digit1: createImageBitmap("./assets/digit_1.png"),
  digit2: createImageBitmap("./assets/digit_2.png"),
  digit3: createImageBitmap("./assets/digit_3.png"),
  digit4: createImageBitmap("./assets/digit_4.png"),
  digit5: createImageBitmap("./assets/digit_5.png"),
  digit6: createImageBitmap("./assets/digit_6.png"),
  digit7: createImageBitmap("./assets/digit_7.png"),
  digit8: createImageBitmap("./assets/digit_8.png"),
  digit9: createImageBitmap("./assets/digit_9.png"),
  textHi: createImageBitmap("./assets/text_hi.png"),
  gameOver: createImageBitmap("./assets/game_over.png"),
  restart: createImageBitmap("./assets/restart.png"),
};

const digitSprites = [
  sprites.digit0,
  sprites.digit1,
  sprites.digit2,
  sprites.digit3,
  sprites.digit4,
  sprites.digit5,
  sprites.digit6,
  sprites.digit7,
  sprites.digit8,
  sprites.digit9,
];

const moonPhases = [
  sprites.moon0,
  sprites.moon1,
  sprites.moon2,
  sprites.moon3,
  sprites.moon4,
  sprites.moon5,
  sprites.moon6,
];

type CactusVariant = {
  sprite: ImageBitmap;
  w: number;
  h: number;
  yPos: number;
};
const cactusVariants: CactusVariant[] = [
  { sprite: sprites.cactusSmall, w: 17, h: 35, yPos: 105 },
  { sprite: sprites.cactusSmallDouble, w: 34, h: 35, yPos: 105 },
  { sprite: sprites.cactusSmallTriple, w: 51, h: 35, yPos: 105 },
  { sprite: sprites.cactusLarge, w: 25, h: 50, yPos: 90 },
  { sprite: sprites.cactusLargeDouble, w: 50, h: 50, yPos: 90 },
  { sprite: sprites.cactusLargeTriple, w: 75, h: 50, yPos: 90 },
];

type Box = { x: number; y: number; w: number; h: number };

const trexRunBoxes: Box[] = [
  { x: 22, y: 0, w: 17, h: 16 },
  { x: 1, y: 18, w: 30, h: 9 },
  { x: 10, y: 35, w: 14, h: 8 },
  { x: 1, y: 24, w: 29, h: 5 },
  { x: 5, y: 30, w: 21, h: 4 },
  { x: 9, y: 34, w: 15, h: 4 },
];
const trexDuckBoxes: Box[] = [{ x: 1, y: 18, w: 55, h: 25 }];

const cactusSmallBoxes: Box[] = [
  { x: 0, y: 7, w: 5, h: 27 },
  { x: 4, y: 0, w: 6, h: 34 },
  { x: 10, y: 4, w: 7, h: 14 },
];
const cactusLargeBoxes: Box[] = [
  { x: 0, y: 12, w: 7, h: 38 },
  { x: 8, y: 0, w: 7, h: 49 },
  { x: 13, y: 10, w: 10, h: 38 },
];
const birdBoxes: Box[] = [
  { x: 15, y: 15, w: 16, h: 5 },
  { x: 18, y: 21, w: 24, h: 6 },
  { x: 2, y: 14, w: 4, h: 3 },
  { x: 6, y: 10, w: 4, h: 7 },
  { x: 10, y: 8, w: 6, h: 9 },
];

function cactusBoxesFor(v: CactusVariant): Box[] {
  const single = v.h === 35 ? cactusSmallBoxes : cactusLargeBoxes;
  const unit = v.h === 35 ? 17 : 25;
  const count = Math.round(v.w / unit);
  const out: Box[] = [];
  for (let i = 0; i < count; i++) {
    for (const b of single) {
      out.push({ x: b.x + i * unit, y: b.y, w: b.w, h: b.h });
    }
  }
  return out;
}

type Phase = "ready" | "playing" | "crashed";

interface Cloud {
  x: number;
  y: number;
  gap: number;
}

interface Obstacle {
  kind: "cactus";
  variant: CactusVariant;
  x: number;
  yPos: number;
  w: number;
  h: number;
  boxes: Box[];
  followingCreated: boolean;
}

interface Bird {
  kind: "bird";
  x: number;
  yPos: number;
  w: number;
  h: number;
  frameTimer: number;
  flapFrame: 0 | 1;
  speedOffset: number;
  followingCreated: boolean;
}

type Entity = Obstacle | Bird;

interface Star {
  x: number;
  y: number;
}

interface NightMode {
  phase: number;
  opacity: number;
  x: number;
  stars: Star[];
}

interface State {
  phase: Phase;
  lastMs: number;
  startMs: number;
  speed: number;
  distance: number;
  highScore: number;
  achievement: boolean;
  achievementFlashTimer: number;
  achievementFlashIter: number;
  inverted: boolean;
  invertTimer: number;
  trexX: number;
  trexY: number;
  trexVy: number;
  jumping: boolean;
  ducking: boolean;
  speedDrop: boolean;
  reachedMinHeight: boolean;
  blinkTimer: number;
  blinkOn: boolean;
  runFrame: 0 | 1;
  runFrameTimer: number;
  duckFrame: 0 | 1;
  duckFrameTimer: number;
  horizonOffset: number;
  horizonSegment: 0 | 1;
  clouds: Cloud[];
  cloudSpawnGap: number;
  entities: Entity[];
  spawnGap: number;
  obstacleHistory: string[];
  night: NightMode;
  jumpCount: number;
  introX: number;
  gameOverMs: number;
}

function score(distance: number): number {
  return Math.floor(distance * DISTANCE_COEFFICIENT);
}

function loadBest(): number {
  const raw = Number(localStorage.getItem(BEST_KEY));
  return Number.isFinite(raw) && raw >= 0 ? Math.floor(raw) : 0;
}
function saveBest(n: number) {
  localStorage.setItem(BEST_KEY, String(n));
}

function randInt(min: number, max: number) {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function placeStars(width: number): Star[] {
  const seg = Math.round(width / NUM_STARS);
  const out: Star[] = [];
  for (let i = 0; i < NUM_STARS; i++) {
    out.push({ x: randInt(seg * i, seg * (i + 1)), y: randInt(0, STAR_MAX_Y) });
  }
  return out;
}

function fresh(highScore: number): State {
  return {
    phase: "ready",
    lastMs: Date.now(),
    startMs: Date.now(),
    speed: INITIAL_SPEED,
    distance: 0,
    highScore,
    achievement: false,
    achievementFlashTimer: 0,
    achievementFlashIter: 0,
    inverted: false,
    invertTimer: 0,
    trexX: 0,
    trexY: TREX_GROUND_Y,
    trexVy: 0,
    jumping: false,
    ducking: false,
    speedDrop: false,
    reachedMinHeight: false,
    blinkTimer: 0,
    blinkOn: false,
    runFrame: 0,
    runFrameTimer: 0,
    duckFrame: 0,
    duckFrameTimer: 0,
    horizonOffset: 0,
    horizonSegment: 0,
    clouds: [],
    cloudSpawnGap: randInt(MIN_CLOUD_GAP, MAX_CLOUD_GAP),
    entities: [],
    spawnGap: 0,
    obstacleHistory: [],
    night: { phase: 0, opacity: 0, x: WIDTH, stars: placeStars(WIDTH) },
    jumpCount: 0,
    introX: 0,
    gameOverMs: 0,
  };
}

const win = Andromeda.createWindow({
  title: "Press space to start",
  width: WIDTH * 2,
  height: HEIGHT * 2,
});
const canvas = new OffscreenCanvas(WIDTH, HEIGHT);
const ctx = canvas.getContext("2d")!;

let state = fresh(loadBest());

function startJump() {
  if (state.jumping || state.ducking) return;
  state.jumping = true;
  state.trexVy = INITIAL_JUMP_VELOCITY - state.speed / 10;
  state.reachedMinHeight = false;
  state.speedDrop = false;
  state.jumpCount++;
}

function endJump() {
  if (state.reachedMinHeight && state.trexVy < DROP_VELOCITY) {
    state.trexVy = DROP_VELOCITY;
  }
}

function setDuck(on: boolean) {
  if (state.jumping) {
    if (on) {
      state.speedDrop = true;
      state.trexVy = 1;
    }
    return;
  }
  state.ducking = on;
}

function jumpAction() {
  if (state.phase === "crashed") {
    if (Date.now() - state.gameOverMs < 1200) return;
    state = fresh(state.highScore);
    state.phase = "playing";
    state.lastMs = Date.now();
    state.startMs = Date.now();
    return;
  }
  if (state.phase === "ready") {
    state.phase = "playing";
    state.lastMs = Date.now();
    state.startMs = Date.now();
  }
  startJump();
}

win.addEventListener("keydown", (e: any) => {
  const code: string = e.detail.code;
  if (code === "Escape") {
    win.close();
    return;
  }
  if (e.detail.repeat) return;
  if (code === "Space" || code === "ArrowUp" || code === "KeyW") jumpAction();
  if (code === "ArrowDown" || code === "KeyS") setDuck(true);
});
win.addEventListener("keyup", (e: any) => {
  const code: string = e.detail.code;
  if (code === "Space" || code === "ArrowUp" || code === "KeyW") endJump();
  if (code === "ArrowDown" || code === "KeyS") {
    state.speedDrop = false;
    setDuck(false);
  }
});
win.addEventListener("mousedown", (e: any) => {
  if (e.detail.button === 0) jumpAction();
});

function spawnCloud() {
  state.clouds.push({
    x: WIDTH,
    y: randInt(MAX_SKY_LEVEL, MIN_SKY_LEVEL),
    gap: randInt(MIN_CLOUD_GAP, MAX_CLOUD_GAP),
  });
}

const OBSTACLE_TYPES = [
  { id: "cactusSmall", minSpeed: 0, minGap: 120, multipleSpeed: 4 },
  { id: "cactusLarge", minSpeed: 0, minGap: 120, multipleSpeed: 7 },
  { id: "pterodactyl", minSpeed: 8.5, minGap: 150, multipleSpeed: 999 },
] as const;
type ObstacleId = typeof OBSTACLE_TYPES[number]["id"];

const GAP_COEFFICIENT = 0.6;
const MAX_GAP_COEFFICIENT = 1.5;
const MAX_OBSTACLE_DUPLICATION = 2;
const BIRD_SPEED_OFFSET = 0.8;
const BIRD_Y_POSITIONS = [100, 75, 50];

function smallCactusVariants(): CactusVariant[] {
  return cactusVariants.filter((v) => v.h === 35);
}
function largeCactusVariants(): CactusVariant[] {
  return cactusVariants.filter((v) => v.h === 50);
}

function isDuplicate(typeId: ObstacleId): boolean {
  let count = 0;
  for (const t of state.obstacleHistory) {
    if (t === typeId) count++;
    else break;
  }
  return count >= MAX_OBSTACLE_DUPLICATION;
}

function recordObstacle(typeId: ObstacleId) {
  state.obstacleHistory.unshift(typeId);
  state.obstacleHistory.length = Math.min(
    state.obstacleHistory.length,
    MAX_OBSTACLE_DUPLICATION,
  );
}

function pickCactusVariant(
  pool: CactusVariant[],
  multipleSpeed: number,
): CactusVariant {
  let size = randInt(1, 3);
  if (size > 1 && multipleSpeed > state.speed) size = 1;
  const candidates = pool.filter((v) =>
    Math.round(v.w / (v.h === 35 ? 17 : 25)) === size
  );
  return candidates[Math.floor(Math.random() * candidates.length)];
}

function getGap(width: number, minGap: number): number {
  const min = Math.round(width * state.speed + minGap * GAP_COEFFICIENT);
  const max = Math.round(min * MAX_GAP_COEFFICIENT);
  return randInt(min, max);
}

function spawnObstacle() {
  const typeIdx = Math.floor(Math.random() * OBSTACLE_TYPES.length);
  const type = OBSTACLE_TYPES[typeIdx];
  if (state.speed < type.minSpeed || isDuplicate(type.id)) {
    spawnObstacle();
    return;
  }

  if (type.id === "pterodactyl") {
    const yPos =
      BIRD_Y_POSITIONS[Math.floor(Math.random() * BIRD_Y_POSITIONS.length)];
    const speedOffset = (Math.random() > 0.5 ? 1 : -1) * BIRD_SPEED_OFFSET;
    state.entities.push({
      kind: "bird",
      x: WIDTH,
      yPos,
      w: 46,
      h: 40,
      frameTimer: 0,
      flapFrame: 0,
      speedOffset,
      followingCreated: false,
    });
    state.spawnGap = getGap(46, type.minGap);
    recordObstacle(type.id);
    return;
  }

  const pool = type.id === "cactusSmall"
    ? smallCactusVariants()
    : largeCactusVariants();
  const variant = pickCactusVariant(pool, type.multipleSpeed);
  state.entities.push({
    kind: "cactus",
    variant,
    x: WIDTH,
    yPos: variant.yPos,
    w: variant.w,
    h: variant.h,
    boxes: cactusBoxesFor(variant),
    followingCreated: false,
  });
  state.spawnGap = getGap(variant.w, type.minGap);
  recordObstacle(type.id);
}

function updateNight(active: boolean, dt: number) {
  const n = state.night;
  if (active && n.opacity === 0) {
    n.phase = (n.phase + 1) % moonPhases.length;
  }
  if (active && n.opacity < 1) {
    n.opacity = Math.min(1, n.opacity + NIGHT_FADE_SPEED);
  } else if (!active && n.opacity > 0) {
    n.opacity = Math.max(0, n.opacity - NIGHT_FADE_SPEED);
  }

  if (n.opacity > 0) {
    n.x -= MOON_SPEED;
    if (n.x < -MOON_W) n.x = WIDTH;
    for (const s of n.stars) {
      s.x -= STAR_SPEED;
      if (s.x < -STAR_SIZE) s.x = WIDTH;
    }
  } else {
    n.stars = placeStars(WIDTH);
  }
}

function update() {
  const now = Date.now();
  const dt = Math.min(50, now - state.lastMs);
  state.lastMs = now;

  if (state.phase === "ready") {
    state.blinkTimer += dt;
    if (
      state.blinkTimer > (state.blinkOn ? 200 : 2000 + Math.random() * 4000)
    ) {
      state.blinkTimer = 0;
      state.blinkOn = !state.blinkOn;
    }
    return;
  }

  if (state.phase === "crashed") {
    return;
  }

  if (state.speed < MAX_SPEED) {
    state.speed = Math.min(MAX_SPEED, state.speed + ACCELERATION * dt);
  }
  state.distance += (state.speed * FPS * dt) / 1000;

  const sc = score(state.distance);
  if (!state.achievement && sc > 0 && sc % ACHIEVEMENT_DISTANCE === 0) {
    state.achievement = true;
    state.achievementFlashTimer = 0;
    state.achievementFlashIter = 0;
  }
  if (state.achievement) {
    state.achievementFlashTimer += dt;
    if (state.achievementFlashTimer > SCORE_FLASH_DURATION * 2) {
      state.achievementFlashTimer = 0;
      state.achievementFlashIter++;
      if (state.achievementFlashIter >= SCORE_FLASH_ITERATIONS) {
        state.achievement = false;
      }
    }
  }

  if (state.invertTimer > INVERT_FADE_DURATION) {
    state.invertTimer = 0;
    state.inverted = false;
  } else if (state.invertTimer > 0) {
    state.invertTimer += dt;
  } else if (sc > 0 && sc % INVERT_DISTANCE === 0) {
    state.invertTimer = dt;
    state.inverted = true;
  }
  updateNight(state.inverted, dt);

  if (state.introX < TREX_START_X) {
    state.introX = Math.min(
      TREX_START_X,
      state.introX + (TREX_START_X / 1500) * dt,
    );
    state.trexX = state.introX;
  } else {
    state.trexX = TREX_START_X;
  }

  if (state.jumping) {
    const framesElapsed = dt / (1000 / FPS);
    if (state.speedDrop) {
      state.trexY += Math.round(
        state.trexVy * SPEED_DROP_COEFFICIENT * framesElapsed,
      );
    } else state.trexY += Math.round(state.trexVy * framesElapsed);
    state.trexVy += GRAVITY * framesElapsed;

    if (state.trexY < MIN_JUMP_Y || state.speedDrop) {
      state.reachedMinHeight = true;
    }
    if (state.trexY < MAX_JUMP_Y || state.speedDrop) endJump();
    if (state.trexY > TREX_GROUND_Y) {
      state.trexY = TREX_GROUND_Y;
      state.jumping = false;
      state.trexVy = 0;
      state.speedDrop = false;
    }
  }

  state.runFrameTimer += dt;
  if (state.runFrameTimer > 1000 / 12) {
    state.runFrameTimer = 0;
    state.runFrame = state.runFrame === 0 ? 1 : 0;
  }
  state.duckFrameTimer += dt;
  if (state.duckFrameTimer > 1000 / 8) {
    state.duckFrameTimer = 0;
    state.duckFrame = state.duckFrame === 0 ? 1 : 0;
  }

  state.horizonOffset =
    (state.horizonOffset + Math.floor((state.speed * FPS * dt) / 1000)) %
    HORIZON_W;

  for (const c of state.clouds) c.x -= (BG_CLOUD_SPEED * dt * state.speed) / 16;
  state.clouds = state.clouds.filter((c) => c.x + 46 > 0);
  if (
    state.clouds.length < MAX_CLOUDS &&
    (state.clouds.length === 0 ||
      WIDTH - state.clouds[state.clouds.length - 1].x >
        state.clouds[state.clouds.length - 1].gap) &&
    Math.random() < CLOUD_FREQUENCY
  ) {
    spawnCloud();
  }

  for (const e of state.entities) {
    const entSpeed = e.kind === "bird"
      ? state.speed + e.speedOffset
      : state.speed;
    e.x -= (entSpeed * FPS * dt) / 1000;
    if (e.kind === "bird") {
      e.frameTimer += dt;
      if (e.frameTimer > 1000 / 6) {
        e.frameTimer = 0;
        e.flapFrame = e.flapFrame === 0 ? 1 : 0;
      }
    }
  }
  state.entities = state.entities.filter((e) => e.x + e.w > 0);

  if (state.entities.length === 0) {
    spawnObstacle();
  } else {
    const last = state.entities[state.entities.length - 1];
    if (!last.followingCreated && last.x + last.w + state.spawnGap < WIDTH) {
      spawnObstacle();
      last.followingCreated = true;
    }
  }

  if (now - state.startMs > 800) {
    for (const e of state.entities) {
      if (collides(e)) {
        crash();
        break;
      }
    }
  }

  // High score.
  if (sc > state.highScore) {
    state.highScore = sc;
    saveBest(state.highScore);
  }
}

function trexBoxes(): Box[] {
  return state.ducking ? trexDuckBoxes : trexRunBoxes;
}

function trexAabb(): Box {
  const w = state.ducking ? TREX_DUCK_W : TREX_W;
  return { x: state.trexX + 1, y: state.trexY + 1, w: w - 2, h: TREX_H - 2 };
}

function entityBoxes(e: Entity): Box[] {
  if (e.kind === "cactus") return e.boxes;
  return birdBoxes;
}

function entityAabb(e: Entity): Box {
  return { x: e.x + 1, y: e.yPos + 1, w: e.w - 2, h: e.h - 2 };
}

function aabbHits(a: Box, b: Box) {
  return a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h &&
    a.y + a.h > b.y;
}

function collides(e: Entity): boolean {
  const tBox = trexAabb();
  const eBox = entityAabb(e);
  if (!aabbHits(tBox, eBox)) return false;
  for (const tb of trexBoxes()) {
    const tw: Box = { x: tBox.x + tb.x, y: tBox.y + tb.y, w: tb.w, h: tb.h };
    for (const eb of entityBoxes(e)) {
      const ew: Box = { x: eBox.x + eb.x, y: eBox.y + eb.y, w: eb.w, h: eb.h };
      if (aabbHits(tw, ew)) return true;
    }
  }
  return false;
}

function crash() {
  state.phase = "crashed";
  state.gameOverMs = Date.now();
}

function drawHorizon() {
  const o = state.horizonOffset;
  ctx.drawImage(sprites.horizon, -o, HORIZON_Y);
  ctx.drawImage(sprites.horizon, HORIZON_W - o, HORIZON_Y);
}

function drawClouds() {
  for (const c of state.clouds) ctx.drawImage(sprites.cloud, c.x, c.y);
}

function drawNight() {
  const n = state.night;
  if (n.opacity <= 0) return;
  ctx.globalAlpha = n.opacity;
  for (const s of n.stars) ctx.drawImage(sprites.star, Math.round(s.x), s.y);
  ctx.drawImage(moonPhases[n.phase], Math.round(n.x), MOON_Y);
  ctx.globalAlpha = 1;
}

function dinoSprite(): ImageBitmap {
  if (state.phase === "crashed") return sprites.dinoCrash;
  if (state.ducking) {
    return state.duckFrame === 0 ? sprites.dinoDuck1 : sprites.dinoDuck2;
  }
  if (state.jumping || state.phase === "ready") {
    if (state.phase === "ready" && state.blinkOn) return sprites.dinoBlink;
    return sprites.dinoIdle;
  }
  return state.runFrame === 0 ? sprites.dinoWalk1 : sprites.dinoWalk2;
}

function drawDino() {
  ctx.drawImage(dinoSprite(), state.trexX, state.trexY);
}

function drawEntities() {
  for (const e of state.entities) {
    if (e.kind === "cactus") {
      ctx.drawImage(e.variant.sprite, e.x, e.yPos);
    } else {
      ctx.drawImage(
        e.flapFrame === 0 ? sprites.birdIdle : sprites.birdFlap,
        e.x,
        e.yPos,
      );
    }
  }
}

function drawDigitsAt(value: number, leftX: number, padTo: number) {
  let s = String(value);
  while (s.length < padTo) s = "0" + s;
  for (let i = 0; i < s.length; i++) {
    const d = s.charCodeAt(i) - 48;
    ctx.drawImage(digitSprites[d], leftX + i * SCORE_PAD, SCORE_Y);
  }
}

function drawScore() {
  const flashing = state.achievement &&
    state.achievementFlashTimer < SCORE_FLASH_DURATION;
  const scoreLeft = WIDTH - SCORE_PAD * (SCORE_DIGITS + 1);

  if (state.highScore > 0) {
    const hiLeft = scoreLeft - SCORE_DIGITS * 2 * DIGIT_W;
    ctx.globalAlpha = 0.8;
    ctx.drawImage(sprites.textHi, hiLeft, SCORE_Y);
    // "HI" sprite occupies positions 0-1; position 2 is the gap; digits 3-7.
    drawDigitsAt(state.highScore, hiLeft + 3 * SCORE_PAD, SCORE_DIGITS);
    ctx.globalAlpha = 1;
  }
  if (!flashing) drawDigitsAt(score(state.distance), scoreLeft, SCORE_DIGITS);
}

function drawGameOver() {
  const goW = 191;
  const goH = 11;
  const goX = (WIDTH - goW) / 2;
  const goY = Math.round((HEIGHT - 25) / 3);
  ctx.drawImage(sprites.gameOver, goX, goY);

  const restartW = 36;
  const restartH = 32;
  ctx.drawImage(
    sprites.restart,
    Math.round((WIDTH - restartW) / 2),
    Math.round(goY + goH + (HEIGHT - goY - goH - restartH) / 2),
  );
}

function clearBackground() {
  const dark = state.night.opacity;
  const r = Math.round(255 * (1 - dark));
  ctx.fillStyle = `rgb(${r},${r},${r})`;
  ctx.fillRect(0, 0, WIDTH, HEIGHT);
}

function render() {
  clearBackground();
  drawNight();
  drawClouds();
  drawHorizon();
  drawEntities();
  drawDino();
  drawScore();
  if (state.phase === "crashed") drawGameOver();
}

await Andromeda.Window.mainloop(() => {
  update();
  render();
  win.presentCanvas(canvas);
});
