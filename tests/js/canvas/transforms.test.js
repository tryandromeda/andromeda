// deno-lint-ignore-file no-undef
import "./_harness.js";
const { test, assertCloseTo, assertEqual, report } = globalThis.canvasHarness;

function ctx() {
  return new OffscreenCanvas(64, 64).getContext("2d");
}

function approxIdentity(m) {
  assertCloseTo(m.a, 1, 1e-6, "a");
  assertCloseTo(m.b, 0, 1e-6, "b");
  assertCloseTo(m.c, 0, 1e-6, "c");
  assertCloseTo(m.d, 1, 1e-6, "d");
  assertCloseTo(m.e, 0, 1e-6, "e");
  assertCloseTo(m.f, 0, 1e-6, "f");
}

test(() => {
  const c = ctx();
  approxIdentity(c.getTransform());
}, "getTransform returns identity by default");

test(() => {
  const c = ctx();
  c.translate(10, 20);
  c.scale(2, 3);
  const m = c.getTransform();
  assertCloseTo(m.a, 2, 1e-6);
  assertCloseTo(m.d, 3, 1e-6);
  assertCloseTo(m.e, 10, 1e-6);
  assertCloseTo(m.f, 20, 1e-6);
}, "translate then scale composes correctly");

test(() => {
  const c = ctx();
  c.translate(5, 5);
  c.save();
  c.translate(20, 30);
  c.restore();
  const m = c.getTransform();
  assertCloseTo(m.e, 5, 1e-6);
  assertCloseTo(m.f, 5, 1e-6);
}, "save/restore preserves transform");

test(() => {
  const c = ctx();
  c.translate(10, 0);
  c.resetTransform();
  approxIdentity(c.getTransform());
}, "resetTransform restores identity");

test(() => {
  const c = ctx();
  c.setTransform(2, 0, 0, 2, 5, 7);
  const m = c.getTransform();
  assertCloseTo(m.a, 2, 1e-6);
  assertCloseTo(m.d, 2, 1e-6);
  assertCloseTo(m.e, 5, 1e-6);
  assertCloseTo(m.f, 7, 1e-6);
}, "setTransform writes the full matrix");

test(() => {
  const c = ctx();
  c.translate(1, 2);
  c.rotate(Math.PI / 2);
  const m = c.getTransform();
  assertCloseTo(Math.abs(m.a), 0, 1e-6);
  assertCloseTo(Math.abs(m.d), 0, 1e-6);
  assertCloseTo(m.e, 1, 1e-6);
  assertCloseTo(m.f, 2, 1e-6);
}, "rotate after translate keeps origin");

test(() => {
  const c = ctx();
  assertEqual(typeof c.transform, "function");
  c.transform(1, 0, 0, 1, 5, 5);
  c.transform(1, 0, 0, 1, 5, 5);
  const m = c.getTransform();
  assertCloseTo(m.e, 10, 1e-6);
  assertCloseTo(m.f, 10, 1e-6);
}, "transform composes successively");

report();
