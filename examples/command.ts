const echo = new Andromeda.Command("cmd", {
  args: ["/c", "echo", "Hello from Andromeda.Command!"],
});
const syncOutput = echo.outputSync();
console.log("Sync output:", syncOutput.stdout);
console.log("Exit code:", syncOutput.code);
console.log("Success:", syncOutput.success);
console.log("Signal:", syncOutput.signal);

const dir = new Andromeda.Command("cmd", { args: ["/c", "dir", "."] });
const asyncOutput = await dir.output();
console.log("\nAsync dir output:");
console.log(asyncOutput.stdout);

const cd = new Andromeda.Command("cmd", {
  args: ["/c", "cd"],
  cwd: "C:\\Windows",
});
const cdOutput = await cd.output();
console.log("Working directory:", cdOutput.stdout);

const envCmd = new Andromeda.Command("cmd", {
  args: ["/c", "echo", "%MY_VAR%"],
  env: { MY_VAR: "custom_value_from_andromeda" },
});
const envOutput = await envCmd.output();
console.log("Custom env:", envOutput.stdout);

const bad = new Andromeda.Command("cmd", {
  args: ["/c", "dir", "nonexistent_path_12345"],
});
const badOutput = await bad.output();
console.log("\nFailing command stderr:", badOutput.stderr);
console.log("Failing command success:", badOutput.success);
console.log("Failing command code:", badOutput.code);

const ping = new Andromeda.Command("cmd", {
  args: ["/c", "ping", "-n", "1", "127.0.0.1"],
});
const child = ping.spawn();
console.log("\nSpawned process pid:", child.pid);
const status = await child.status;
console.log("Status:", status.success, "code:", status.code);

const child2cmd = new Andromeda.Command("cmd", {
  args: ["/c", "echo", "from child output"],
});
const child2 = child2cmd.spawn();
const child2Output = await child2.output();
console.log("\nChild output:", child2Output.stdout);
console.log("Child stderr:", child2Output.stderr);

const ls = new Andromeda.Command("bash", { args: ["-c", "ls -la"] });
const lsOutput = await ls.output();
console.log("\nls output:");
console.log(lsOutput.stdout);

const quiet = new Andromeda.Command("cmd", {
  args: ["/c", "echo", "you wont see this"],
  stdout: "null",
});
const quietOutput = await quiet.output();
console.log(
  "Null stdout (should be empty):",
  JSON.stringify(quietOutput.stdout),
);
