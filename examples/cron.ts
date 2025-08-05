let executionCount = 0;
const maxExecutions = 3;

Andromeda.cron("simple-test", "* * * * *", () => {
  executionCount++;
  console.log(
    `Cron job executed ${executionCount}/${maxExecutions} at:`,
    new Date().toISOString(),
  );

  if (executionCount >= maxExecutions) {
    console.log("Test completed - stopping runtime");
    Andromeda.exit(0);
  }
});

console.log("Cron job registered successfully!");
console.log("Waiting for cron executions (every minute)...");

console.log("Starting cron monitoring...");
