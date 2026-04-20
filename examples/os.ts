// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

function formatUptime(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds));
  const days = Math.floor(total / 86_400);
  const hours = Math.floor((total % 86_400) / 3_600);
  const minutes = Math.floor((total % 3_600) / 60);
  const secs = total % 60;

  const parts: string[] = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0 || days > 0) parts.push(`${hours}h`);
  if (minutes > 0 || hours > 0 || days > 0) parts.push(`${minutes}m`);
  parts.push(`${secs}s`);
  return parts.join(" ");
}

const hostname = Andromeda.hostname();
const release = Andromeda.osRelease();
const uptimeSeconds = Andromeda.osUptime();
const load = Andromeda.loadavg();
const memory = Andromeda.memoryUsage();
const consoleSize = Andromeda.consoleSize();

console.log(`Hostname: ${hostname}`);
console.log(`OS Release: ${release}`);
console.log(`Uptime: ${uptimeSeconds}s (${formatUptime(uptimeSeconds)})`);
console.log(`Load Average (1m, 5m, 15m): [${load[0]}, ${load[1]}, ${load[2]}]`);
console.log(
  `Memory Usage (bytes): rss=${memory.rss}, heapTotal=${memory.heapTotal}, heapUsed=${memory.heapUsed}, external=${memory.external}`,
);
console.log(
  `Console Size: columns=${consoleSize.columns}, rows=${consoleSize.rows}`,
);
