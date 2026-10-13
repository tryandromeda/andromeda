var _test_count = 0;
var _test_results = [];

function test(func, name) {
  _test_count++;
  var testName = name || "test_" + _test_count;
  var completed = false;
  function recordFailure(message) {
    _test_results.push({
      name: testName,
      pass: false,
      message: message,
    });
    completed = true;
  }
  var test_object = {
    add_cleanup: function(cleanupFunc) {},
    done: function() {
      completed = true;
    },
    step_func: function(fn) {
      return function() {
        try {
          return fn.apply(this, arguments);
        } catch (e) {
          recordFailure(e.toString());
          throw e;
        }
      };
    },
    step_func_done: function(fn) {
      return function() {
        try {
          if (typeof fn === "function") {
            fn.apply(this, arguments);
          }
          test_object.done();
        } catch (e) {
          recordFailure(e.toString());
          throw e;
        }
      };
    },
    unreached_func: function(description) {
      return function() {
        recordFailure(description || "Reached unreachable code");
        throw new Error(description || "Reached unreachable code");
      };
    },
    step_timeout: function(fn, timeout) {
      setTimeout(function() {
        try {
          fn();
        } catch (e) {
          recordFailure(e.toString());
        }
      }, timeout);
    },
  };
  try {
    func(test_object);
    if (!completed) {
      _test_results.push({ name: testName, pass: true });
      completed = true;
    }
  } catch (e) {
    if (!completed) {
      _test_results.push({
        name: testName,
        pass: false,
        message: e.toString(),
      });
      completed = true;
    }
  }
}

function async_test(func, name) {
  return test(func, name);
}

function promise_test(func, name) {
  _test_count++;
  try {
    var result = func();
    if (result && typeof result.then === "function") {
      result.then(
        function() {
          _test_results.push({
            name: name || "test_" + _test_count,
            pass: true,
          });
        },
        function(e) {
          _test_results.push({
            name: name || "test_" + _test_count,
            pass: false,
            message: e.toString(),
          });
        },
      );
    } else {
      _test_results.push({ name: name || "test_" + _test_count, pass: true });
    }
  } catch (e) {
    _test_results.push({
      name: name || "test_" + _test_count,
      pass: false,
      message: e.toString(),
    });
  }
}

function assert_true(actual, description) {
  if (!actual) {
    throw new Error(
      description || "assert_true failed: expected true, got " + actual,
    );
  }
}

function assert_false(actual, description) {
  if (actual) {
    throw new Error(
      description || "assert_false failed: expected false, got " + actual,
    );
  }
}

function assert_equals(actual, expected, description) {
  if (actual !== expected) {
    throw new Error(
      description ||
        "assert_equals failed: expected " + expected + ", got " + actual,
    );
  }
}

function assert_not_equals(actual, expected, description) {
  if (actual === expected) {
    throw new Error(
      description ||
        "assert_not_equals failed: expected not " + expected + ", got " +
          actual,
    );
  }
}

function assert_array_equals(actual, expected, description) {
  if (!Array.isArray(actual) || !Array.isArray(expected)) {
    throw new Error(
      description ||
        "assert_array_equals failed: both arguments must be arrays",
    );
  }
  if (actual.length !== expected.length) {
    throw new Error(
      description || "assert_array_equals failed: array lengths differ",
    );
  }
  for (var i = 0; i < actual.length; i++) {
    if (actual[i] !== expected[i]) {
      throw new Error(
        description ||
          "assert_array_equals failed at index " + i + ": expected " +
            expected[i] + ", got " + actual[i],
      );
    }
  }
}

function assert_throws_exactly(exception, func, description) {
  try {
    func();
  } catch (e) {
    if (e !== exception) {
      throw new Error(
        description ||
          "assert_throws_exactly failed: expected " + exception + ", got " + e,
      );
    }
    return;
  }
  throw new Error(
    description || "assert_throws_exactly failed: no exception was thrown",
  );
}

function done() {}

function assert_throws_js(constructor, func, description) {
  try {
    func();
    throw new Error(
      description ||
        "assert_throws_js failed: expected " + constructor.name +
          " to be thrown",
    );
  } catch (e) {
    if (!(e instanceof constructor)) {
      throw new Error(
        description ||
          "assert_throws_js failed: expected " + constructor.name + ", got " +
            e.constructor.name,
      );
    }
  }
}
