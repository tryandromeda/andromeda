setTimeout(function() {
  console.log("WPT_RESULTS:" + JSON.stringify(_test_results));
  if (typeof process !== "undefined" && process.exit) {
    process.exit(0);
  }
}, 10);
