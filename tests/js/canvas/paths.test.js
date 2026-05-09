// deno-lint-ignore-file no-undef
import "./_harness.js";
const { test, assertEqual, assertTruthy, assertFalsy, report } = globalThis.canvasHarness;

function ctx() {
  return new OffscreenCanvas(100, 100).getContext("2d");
}

test(() => {
  const c = ctx();
  c.beginPath();
  c.rect(10, 10, 30, 30);
  assertTruthy(c.isPointInPath(20, 20));
  assertFalsy(c.isPointInPath(50, 50));
}, "isPointInPath: rect contains center, not far point");

test(() => {
  const c = ctx();
  c.beginPath();
  c.arc(50, 50, 20, 0, Math.PI * 2);
  assertTruthy(c.isPointInPath(50, 50));
  assertFalsy(c.isPointInPath(50, 80));
}, "isPointInPath: arc inclusion");

test(() => {
  const c = ctx();
  c.lineWidth = 8;
  c.beginPath();
  c.moveTo(10, 50);
  c.lineTo(90, 50);
  assertTruthy(c.isPointInStroke(50, 50));
  assertFalsy(c.isPointInStroke(50, 80));
}, "isPointInStroke uses current lineWidth");

test(() => {
  const p = new Path2D();
  p.rect(0, 0, 10, 10);
  p.moveTo(20, 20);
  p.lineTo(30, 20);
  assertEqual(typeof p.addPath, "function");
}, "Path2D exposes the spec methods");

test(() => {
  const p = new Path2D("M10 10 L20 10 L20 20 Z");
  const c = ctx();
  assertTruthy(c.isPointInPath(p, 15, 12));
  assertFalsy(c.isPointInPath(p, 50, 50));
}, "Path2D from SVG path string is hit-testable");

test(() => {
  const c = ctx();
  c.beginPath();
  c.roundRect(10, 10, 50, 50, 5);
  assertTruthy(c.isPointInPath(35, 35));
}, "roundRect produces a closed shape");

report();
