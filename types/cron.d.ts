// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * Andromeda Cron API
 * Based on Deno's cron implementation with similar semantics.
 */

/**
 * CronScheduleExpression defines the different ways to specify a time component in a cron schedule.
 */
export type CronScheduleExpression = number | { exact: number | number[]; } | {
  start?: number;
  end?: number;
  every?: number;
};

/**
 * CronSchedule is the interface used for JSON format cron schedule.
 */
export interface CronSchedule {
  minute?: CronScheduleExpression;
  hour?: CronScheduleExpression;
  dayOfMonth?: CronScheduleExpression;
  month?: CronScheduleExpression;
  dayOfWeek?: CronScheduleExpression;
}

/**
 * Create a cron job that will periodically execute the provided handler
 * callback based on the specified schedule.
 *
 * ```ts
 * Andromeda.cron("sample cron", "20 * * * *", () => {
 *   console.log("cron job executed");
 * });
 * ```
 *
 * ```ts
 * Andromeda.cron("sample cron", { hour: { every: 6 } }, () => {
 *   console.log("cron job executed");
 * });
 * ```
 *
 * `schedule` can be a string in the Unix cron format or in JSON format
 * as specified by interface {@linkcode CronSchedule}, where time is specified
 * using UTC time zone.
 */
export function cron(
  name: string,
  schedule: string | CronSchedule,
  handler: () => Promise<void> | void,
): Promise<void>;

/**
 * Create a cron job that will periodically execute the provided handler
 * callback based on the specified schedule.
 *
 * ```ts
 * Andromeda.cron("sample cron", "20 * * * *", {
 *   backoffSchedule: [100, 1000, 5000],
 *   signal: abortController.signal,
 * }, () => {
 *   console.log("cron job executed");
 * });
 * ```
 */
export function cron(
  name: string,
  schedule: string | CronSchedule,
  options: { backoffSchedule?: number[]; signal?: AbortSignal; },
  handler: () => Promise<void> | void,
): Promise<void>;

declare global {
  namespace Andromeda {
    export { cron, CronSchedule, CronScheduleExpression };
  }
}
