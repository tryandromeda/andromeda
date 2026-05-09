// deno-lint-ignore-file no-undef
import "./_harness.js";
const {
  test,
  assertEqual,
  assertCloseTo,
  assertGreaterThan,
  report,
} = globalThis.canvasHarness;

function ctx() {
  return new OffscreenCanvas(64, 64).getContext("2d");
}

test(() => {
  const c = ctx();
  assertEqual(c.letterSpacing, "0px");
  assertEqual(c.wordSpacing, "0px");
  assertEqual(c.fontKerning, "auto");
  assertEqual(c.fontStretch, "normal");
  assertEqual(c.fontVariantCaps, "normal");
  assertEqual(c.textRendering, "auto");
  assertEqual(c.lang, "inherit");
}, "text-style properties have spec defaults");

test(() => {
  const c = ctx();
  c.letterSpacing = "2px";
  assertEqual(c.letterSpacing, "2px");
  c.letterSpacing = "0.5em";
  assertEqual(c.letterSpacing, "0.5em");
}, "letterSpacing round-trips lengths");

test(() => {
  const c = ctx();
  c.wordSpacing = "10px";
  assertEqual(c.wordSpacing, "10px");
  c.wordSpacing = "-3px";
  assertEqual(c.wordSpacing, "-3px");
}, "wordSpacing round-trips lengths including negative");

test(() => {
  const c = ctx();
  for (const v of ["auto", "normal", "none"]) {
    c.fontKerning = v;
    assertEqual(c.fontKerning, v);
  }
}, "fontKerning round-trips all keywords");

test(() => {
  const c = ctx();
  for (
    const v of [
      "ultra-condensed",
      "extra-condensed",
      "condensed",
      "semi-condensed",
      "normal",
      "semi-expanded",
      "expanded",
      "extra-expanded",
      "ultra-expanded",
    ]
  ) {
    c.fontStretch = v;
    assertEqual(c.fontStretch, v);
  }
}, "fontStretch round-trips all keywords");

test(() => {
  const c = ctx();
  for (
    const v of [
      "normal",
      "small-caps",
      "all-small-caps",
      "petite-caps",
      "all-petite-caps",
      "unicase",
      "titling-caps",
    ]
  ) {
    c.fontVariantCaps = v;
    assertEqual(c.fontVariantCaps, v);
  }
}, "fontVariantCaps round-trips all keywords");

test(() => {
  const c = ctx();
  for (
    const v of [
      "auto",
      "optimizeSpeed",
      "optimizeLegibility",
      "geometricPrecision",
    ]
  ) {
    c.textRendering = v;
    assertEqual(c.textRendering, v);
  }
}, "textRendering round-trips all keywords");

test(() => {
  const c = ctx();
  c.lang = "ja";
  assertEqual(c.lang, "ja");
  c.lang = "en-US";
  assertEqual(c.lang, "en-US");
  c.lang = "";
  assertEqual(c.lang, "");
  c.lang = "inherit";
  assertEqual(c.lang, "inherit");
}, "lang round-trips arbitrary strings");

test(() => {
  const c = ctx();
  c.letterSpacing = "5px";
  c.letterSpacing = "garbage";
  assertEqual(c.letterSpacing, "5px");
  c.wordSpacing = "8px";
  c.wordSpacing = "not-a-length";
  assertEqual(c.wordSpacing, "8px");
}, "invalid spacing values keep previous value");

test(() => {
  const c = ctx();
  c.fontKerning = "normal";
  c.fontKerning = "wat";
  assertEqual(c.fontKerning, "normal");
  c.fontStretch = "condensed";
  c.fontStretch = "huge";
  assertEqual(c.fontStretch, "condensed");
  c.fontVariantCaps = "small-caps";
  c.fontVariantCaps = "BIG";
  assertEqual(c.fontVariantCaps, "small-caps");
  c.textRendering = "geometricPrecision";
  c.textRendering = "fast";
  assertEqual(c.textRendering, "geometricPrecision");
}, "invalid keyword values keep previous value");

test(() => {
  const c = ctx();
  c.letterSpacing = "3px";
  c.wordSpacing = "5px";
  c.fontKerning = "none";
  c.fontStretch = "expanded";
  c.fontVariantCaps = "small-caps";
  c.textRendering = "optimizeLegibility";
  c.lang = "ja";
  c.save();
  c.letterSpacing = "0px";
  c.wordSpacing = "0px";
  c.fontKerning = "auto";
  c.fontStretch = "normal";
  c.fontVariantCaps = "normal";
  c.textRendering = "auto";
  c.lang = "inherit";
  c.restore();
  assertEqual(c.letterSpacing, "3px");
  assertEqual(c.wordSpacing, "5px");
  assertEqual(c.fontKerning, "none");
  assertEqual(c.fontStretch, "expanded");
  assertEqual(c.fontVariantCaps, "small-caps");
  assertEqual(c.textRendering, "optimizeLegibility");
  assertEqual(c.lang, "ja");
}, "save/restore round-trips every text-style property");

test(() => {
  const c = ctx();
  c.letterSpacing = "9px";
  c.fontKerning = "none";
  c.lang = "ja";
  c.reset();
  assertEqual(c.letterSpacing, "0px");
  assertEqual(c.fontKerning, "auto");
  assertEqual(c.lang, "inherit");
}, "reset() restores text-style defaults");

test(() => {
  const c = ctx();
  c.font = "20px sans-serif";
  c.letterSpacing = "0.5em";
  assertEqual(c.letterSpacing, "0.5em");
}, "letterSpacing keeps original units");

test(() => {
  const c = ctx();
  c.font = "32px sans-serif";
  c.letterSpacing = "0px";
  const baseline = c.measureText("Hello").width;

  c.letterSpacing = "5px";
  const wider = c.measureText("Hello").width;

  assertGreaterThan(wider, baseline);
  assertCloseTo(wider - baseline, 20, 0.5, "letterSpacing applies (n-1) times");
}, "letterSpacing widens measureText proportionally");

test(() => {
  const c = ctx();
  c.font = "24px sans-serif";
  c.wordSpacing = "0px";
  const baseline = c.measureText("a b c").width;

  c.wordSpacing = "10px";
  const wider = c.measureText("a b c").width;

  assertGreaterThan(wider, baseline);
  assertCloseTo(wider - baseline, 20, 0.5, "wordSpacing applies per space");
}, "wordSpacing widens measureText per whitespace");

test(() => {
  const c = ctx();
  c.font = "24px sans-serif";
  c.letterSpacing = "3px";
  c.wordSpacing = "10px";
  const both = c.measureText("a b").width;

  c.letterSpacing = "0px";
  c.wordSpacing = "0px";
  const neither = c.measureText("a b").width;

  assertGreaterThan(both, neither);
  assertCloseTo(both - neither, 16, 0.5, "combined spacing is additive");
}, "letterSpacing and wordSpacing combine");

test(() => {
  const c = ctx();
  c.font = "20px sans-serif";
  c.letterSpacing = "20px";
  const m = c.measureText("");
  assertEqual(m.width, 0);
}, "empty string measureText returns 0 even with spacing");

test(() => {
  const c = ctx();
  c.font = "20px sans-serif";
  const baseline = c.measureText("Hello").width;
  c.letterSpacing = "0.5em";
  const wider = c.measureText("Hello").width;
  assertCloseTo(
    wider - baseline,
    40,
    0.5,
    "em resolves against current font size",
  );
}, "letterSpacing in em units resolves against font size");

test(() => {
  const c = ctx();
  c.font = "20px sans-serif";
  c.fontStretch = "condensed";
  const m = c.measureText("Hello");
  assertGreaterThan(m.width, 0);
}, "fontStretch does not break measureText when face is unavailable");

report();
