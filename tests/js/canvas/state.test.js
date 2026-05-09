// deno-lint-ignore-file no-undef
import "./_harness.js";
const { test, assertEqual, assertCloseTo, report } = globalThis.canvasHarness;

function ctx() {
  return new OffscreenCanvas(64, 64).getContext("2d");
}

test(() => {
  const c = ctx();
  c.lineWidth = 4;
  c.save();
  c.lineWidth = 12;
  assertEqual(c.lineWidth, 12);
  c.restore();
  assertEqual(c.lineWidth, 4);
}, "save/restore preserves lineWidth");

test(() => {
  const c = ctx();
  c.fillStyle = "#abcdef";
  const beforeSave = c.fillStyle;
  c.save();
  c.fillStyle = "#123456";
  c.restore();
  assertEqual(typeof c.fillStyle, "string");
  assertEqual(c.fillStyle, beforeSave);
}, "save/restore preserves fillStyle string");

test(() => {
  const c = ctx();
  c.lineCap = "round";
  c.lineJoin = "bevel";
  c.miterLimit = 7;
  c.save();
  c.lineCap = "square";
  c.lineJoin = "miter";
  c.miterLimit = 2;
  c.restore();
  assertEqual(c.lineCap, "round");
  assertEqual(c.lineJoin, "bevel");
  assertEqual(c.miterLimit, 7);
}, "save/restore preserves line styles");

test(() => {
  const c = ctx();
  c.setLineDash([5, 5, 2]);
  c.lineDashOffset = 3;
  c.save();
  c.setLineDash([1, 1]);
  c.lineDashOffset = 9;
  c.restore();
  const dash = c.getLineDash();
  assertEqual(dash.length, 6);
  assertEqual(dash[0], 5);
  assertEqual(dash[1], 5);
  assertEqual(dash[2], 2);
  assertEqual(dash[3], 5);
  assertEqual(dash[4], 5);
  assertEqual(dash[5], 2);
}, "setLineDash normalizes odd-length and getLineDash returns even-length copy");

test(() => {
  const c = ctx();
  c.globalAlpha = 0.5;
  c.globalCompositeOperation = "multiply";
  c.save();
  c.globalAlpha = 0.1;
  c.globalCompositeOperation = "screen";
  c.restore();
  assertCloseTo(c.globalAlpha, 0.5, 1e-6);
  assertEqual(c.globalCompositeOperation, "multiply");
}, "save/restore preserves globalAlpha and globalCompositeOperation");

test(() => {
  const c = ctx();
  c.shadowBlur = 8;
  c.shadowColor = "#ff0000";
  c.shadowOffsetX = 2;
  c.shadowOffsetY = -3;
  c.save();
  c.shadowBlur = 0;
  c.shadowColor = "#000000";
  c.shadowOffsetX = 0;
  c.shadowOffsetY = 0;
  c.restore();
  assertEqual(c.shadowBlur, 8);
  assertEqual(c.shadowOffsetX, 2);
  assertEqual(c.shadowOffsetY, -3);
}, "save/restore preserves shadow properties");

test(() => {
  const c = ctx();
  c.font = "20px sans-serif";
  c.textAlign = "center";
  c.textBaseline = "middle";
  c.save();
  c.font = "10px serif";
  c.textAlign = "left";
  c.textBaseline = "top";
  c.restore();
  assertEqual(c.textAlign, "center");
  assertEqual(c.textBaseline, "middle");
}, "save/restore preserves text alignment properties");

test(() => {
  const c = ctx();
  c.lineWidth = 5;
  c.fillStyle = "#abcdef";
  c.shadowBlur = 3;
  c.reset();
  assertEqual(c.lineWidth, 1);
  assertEqual(c.shadowBlur, 0);
}, "reset() restores defaults");

report();
