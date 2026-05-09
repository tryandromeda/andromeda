// deno-lint-ignore-file no-undef
import "./_harness.js";
const { test, assertEqual, report } = globalThis.canvasHarness;

function ctx() {
  return new OffscreenCanvas(32, 32).getContext("2d");
}

const MODES = [
  "source-over",
  "source-in",
  "source-out",
  "source-atop",
  "destination-over",
  "destination-in",
  "destination-out",
  "destination-atop",
  "lighter",
  "copy",
  "xor",
  "multiply",
  "screen",
  "overlay",
  "darken",
  "lighten",
  "color-dodge",
  "color-burn",
  "hard-light",
  "soft-light",
  "difference",
  "exclusion",
  "hue",
  "saturation",
  "color",
  "luminosity",
];

for (const mode of MODES) {
  test(() => {
    const c = ctx();
    c.globalCompositeOperation = mode;
    assertEqual(c.globalCompositeOperation, mode);
  }, `globalCompositeOperation round-trips ${mode}`);
}

test(() => {
  const c = ctx();
  c.globalCompositeOperation = "source-over";
  c.globalCompositeOperation = "not-a-real-mode";
  assertEqual(c.globalCompositeOperation, "source-over");
}, "invalid globalCompositeOperation keeps previous value");

test(() => {
  const c = ctx();
  c.globalAlpha = 0.42;
  assertEqual(Math.round(c.globalAlpha * 100) / 100, 0.42);
}, "globalAlpha round-trips a fractional value");

test(() => {
  const c = ctx();
  c.globalAlpha = -1;
  assertEqual(c.globalAlpha, 0);
  c.globalAlpha = 5;
  assertEqual(c.globalAlpha, 1);
}, "globalAlpha is clamped to [0, 1]");

report();
