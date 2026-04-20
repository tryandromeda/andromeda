// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// @ts-ignore - cross-file handoff
(globalThis as any).__andromeda_os = {
  hostname(): string {
    // @ts-ignore - internal op surface
    return (globalThis as any).__andromeda__.internal_os_hostname();
  },
  osRelease(): string {
    // @ts-ignore - internal op surface
    return (globalThis as any).__andromeda__.internal_os_release();
  },
  osName(): string {
    // @ts-ignore - internal op surface
    return (globalThis as any).__andromeda__.internal_os_name();
  },
  osUptime(): number {
    // @ts-ignore - internal op surface
    return Number((globalThis as any).__andromeda__.internal_os_uptime());
  },
  loadavg(): [number, number, number] {
    // @ts-ignore - internal op surface
    const raw = (globalThis as any).__andromeda__.internal_os_loadavg();
    return JSON.parse(raw);
  },
  memoryUsage(): {
    rss: number;
    heapTotal: number;
    heapUsed: number;
    external: number;
  } {
    // @ts-ignore - internal op surface
    const raw = (globalThis as any).__andromeda__.internal_os_memory_usage();
    return JSON.parse(raw);
  },
  consoleSize(): { columns: number; rows: number } {
    // @ts-ignore - internal op surface
    const raw = (globalThis as any).__andromeda__.internal_os_console_size();
    return JSON.parse(raw);
  },
};
