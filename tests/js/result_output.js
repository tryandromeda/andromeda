const outputResults = () => {
  try {
    const resultsJson = JSON.stringify(_test_results);
    console.log("WPT_RESULTS:" + resultsJson);
  } catch (e) {
    console.error("Failed to output test results:", e);
  }
};

outputResults();

setTimeout(outputResults, 10);

setTimeout(() => {
  outputResults();
  Andromeda.exit(0);
}, 50);
