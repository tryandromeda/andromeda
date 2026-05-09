const _results = [];

function test(fn, name) {
  const label = name || `test_${_results.length + 1}`;
  try {
    fn();
    _results.push({ name: label, pass: true });
  } catch (e) {
    _results.push({
      name: label,
      pass: false,
      message: String(e && e.stack || e),
    });
  }
}

function asyncTest(fn, name) {
  const label = name || `test_${_results.length + 1}`;
  const slot = { name: label, pass: false, pending: true };
  _results.push(slot);
  return Promise.resolve()
    .then(fn)
    .then(() => {
      slot.pass = true;
      slot.pending = false;
    })
    .catch((e) => {
      slot.pass = false;
      slot.pending = false;
      slot.message = String(e && e.stack || e);
    });
}

function assertEqual(actual, expected, msg) {
  if (actual !== expected) {
    throw new Error(
      `${msg || "assertEqual"}: expected ${stringify(expected)}, got ${
        stringify(actual)
      }`,
    );
  }
}

function assertNotEqual(actual, expected, msg) {
  if (actual === expected) {
    throw new Error(
      `${msg || "assertNotEqual"}: both were ${stringify(actual)}`,
    );
  }
}

function assertCloseTo(actual, expected, tolerance, msg) {
  if (typeof actual !== "number" || Number.isNaN(actual)) {
    throw new Error(
      `${msg || "assertCloseTo"}: actual is not a number (${actual})`,
    );
  }
  if (Math.abs(actual - expected) > tolerance) {
    throw new Error(
      `${
        msg || "assertCloseTo"
      }: expected ${expected} ±${tolerance}, got ${actual}`,
    );
  }
}

function assertGreaterThan(actual, threshold, msg) {
  if (!(actual > threshold)) {
    throw new Error(
      `${msg || "assertGreaterThan"}: expected > ${threshold}, got ${actual}`,
    );
  }
}

function assertTruthy(value, msg) {
  if (!value) {
    throw new Error(`${msg || "assertTruthy"}: got ${stringify(value)}`);
  }
}

function assertFalsy(value, msg) {
  if (value) {
    throw new Error(`${msg || "assertFalsy"}: got ${stringify(value)}`);
  }
}

function assertThrows(fn, msg) {
  try {
    fn();
  } catch (_e) {
    return;
  }
  throw new Error(`${msg || "assertThrows"}: did not throw`);
}

function stringify(value) {
  if (typeof value === "string") return JSON.stringify(value);
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  if (value === null || value === undefined) return String(value);
  try {
    return JSON.stringify(value);
  } catch (_e) {
    return String(value);
  }
}

function report() {
  let passed = 0;
  let failed = 0;
  for (const r of _results) {
    if (r.pending) {
      console.error(`PENDING: ${r.name}`);
      failed++;
      continue;
    }
    if (r.pass) {
      passed++;
    } else {
      failed++;
      console.error(`FAIL: ${r.name}\n  ${r.message}`);
    }
  }
  console.log(`\n${passed}/${_results.length} passed`);
  if (failed > 0) {
    throw new Error(`${failed} canvas test(s) failed`);
  }
}

globalThis.canvasHarness = {
  test,
  asyncTest,
  assertEqual,
  assertNotEqual,
  assertCloseTo,
  assertGreaterThan,
  assertTruthy,
  assertFalsy,
  assertThrows,
  report,
};
