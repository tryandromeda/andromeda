// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * Implementation of the Performance API for Andromeda
 * Based on: https://developer.mozilla.org/en-US/docs/Web/API/Performance
 * Spec: https://w3c.github.io/hr-time/
 */

interface PerformanceMark {
  readonly name: string;
  readonly entryType: "mark";
  readonly startTime: number;
  readonly duration: number;
  readonly detail?: unknown;
}

interface PerformanceMeasure {
  readonly name: string;
  readonly entryType: "measure";
  readonly startTime: number;
  readonly duration: number;
  readonly detail?: unknown;
}

interface PerformanceEntry {
  readonly name: string;
  readonly entryType: string;
  readonly startTime: number;
  readonly duration: number;
}

const performanceEntries: PerformanceEntry[] = [];
const performanceMarks = new Map<string, PerformanceMark>();

/**
 * Implementation of the Performance interface
 * Provides high-resolution time measurements and performance monitoring
 */
class AndromedaPerformance {
  /**
   * Returns a high-resolution timestamp in milliseconds
   * @returns {number} The current time relative to the time origin
   */
  now(): number {
    return internal_performance_now();
  }

  /**
   * Returns the time origin (when the performance measurement started)
   * @returns {number} The time origin in milliseconds since Unix epoch
   */
  get timeOrigin(): number {
    return internal_performance_time_origin();
  }

  /**
   * Creates a named timestamp in the performance timeline
   * @param {string} markName - The name of the mark
   * @param {object} markOptions - Optional mark options
   * @returns {PerformanceMark} The created performance mark
   */
  mark(
    markName: string,
    markOptions?: { detail?: unknown; startTime?: number },
  ): PerformanceMark {
    if (typeof markName !== "string") {
      throw new TypeError("Mark name must be a string");
    }

    if (markName.length === 0) {
      throw new TypeError("Mark name cannot be empty");
    }

    const restrictedNames = [
      "navigationStart",
      "unloadEventStart",
      "unloadEventEnd",
      "redirectStart",
      "redirectEnd",
      "fetchStart",
      "domainLookupStart",
      "domainLookupEnd",
      "connectStart",
      "connectEnd",
      "secureConnectionStart",
      "requestStart",
      "responseStart",
      "responseEnd",
      "domLoading",
      "domInteractive",
      "domContentLoadedEventStart",
      "domContentLoadedEventEnd",
      "domComplete",
      "loadEventStart",
      "loadEventEnd",
    ];

    if (restrictedNames.includes(markName)) {
      throw new Error(
        `Cannot create mark with restricted name: ${markName}`,
      );
    }

    const startTime = markOptions?.startTime ?? this.now();
    const detail = markOptions?.detail;

    const mark: PerformanceMark = {
      name: markName,
      entryType: "mark",
      startTime,
      duration: 0,
      detail,
    };

    performanceMarks.set(markName, mark);
    performanceEntries.push(mark);

    return mark;
  }

  /**
   * Creates a named timestamp between two marks or times
   * @param {string} measureName - The name of the measure
   * @param {string | object} startOrMeasureOptions - Start mark name or measure options
   * @param {string} endMark - End mark name (if using string parameters)
   * @returns {PerformanceMeasure} The created performance measure
   */
  measure(
    measureName: string,
    startOrMeasureOptions?: string | {
      start?: string | number;
      end?: string | number;
      detail?: unknown;
      duration?: number;
    },
    endMark?: string,
  ): PerformanceMeasure {
    if (typeof measureName !== "string") {
      throw new TypeError("Measure name must be a string");
    }

    if (measureName.length === 0) {
      throw new TypeError("Measure name cannot be empty");
    }

    let startTime: number;
    let endTime: number;
    let detail: unknown;

    if (
      typeof startOrMeasureOptions === "object" &&
      startOrMeasureOptions !== null
    ) {
      const options = startOrMeasureOptions;
      detail = options.detail;

      if (
        options.duration !== undefined &&
        (options.start !== undefined || options.end !== undefined)
      ) {
        if (options.start !== undefined && options.end !== undefined) {
          throw new TypeError(
            "Cannot specify duration with both start and end",
          );
        }
      }

      if (options.duration !== undefined) {
        if (options.start !== undefined) {
          startTime = this.#resolveTimeValue(options.start);
          endTime = startTime + options.duration;
        } else if (options.end !== undefined) {
          endTime = this.#resolveTimeValue(options.end);
          startTime = endTime - options.duration;
        } else {
          endTime = this.now();
          startTime = endTime - options.duration;
        }
      } else {
        startTime = options.start !== undefined
          ? this.#resolveTimeValue(options.start)
          : 0;
        endTime = options.end !== undefined
          ? this.#resolveTimeValue(options.end)
          : this.now();
      }
    } else {
      const startMark = startOrMeasureOptions;
      startTime = startMark ? this.#resolveTimeValue(startMark) : 0;
      endTime = endMark ? this.#resolveTimeValue(endMark) : this.now();
    }

    const duration = endTime - startTime;

    const measure: PerformanceMeasure = {
      name: measureName,
      entryType: "measure",
      startTime,
      duration,
      detail,
    };

    performanceEntries.push(measure);

    return measure;
  }

  /**
   * Resolve a time value (either a number or mark name) to a timestamp
   */
  #resolveTimeValue(value: string | number): number {
    if (typeof value === "number") {
      return value;
    }

    if (typeof value === "string") {
      const mark = performanceMarks.get(value);
      if (!mark) {
        throw new Error(`Mark "${value}" does not exist`);
      }
      return mark.startTime;
    }

    throw new TypeError("Time value must be a number or mark name");
  }

  /**
   * Removes all performance entries from the timeline
   */
  clearMarks(markName?: string): void {
    if (markName !== undefined) {
      if (typeof markName !== "string") {
        throw new TypeError("Mark name must be a string");
      }
      // Remove specific mark
      performanceMarks.delete(markName);
      const index = performanceEntries.findIndex((entry) =>
        entry.entryType === "mark" && entry.name === markName
      );
      if (index !== -1) {
        performanceEntries.splice(index, 1);
      }
    } else {
      // Remove all marks
      performanceMarks.clear();
      for (let i = performanceEntries.length - 1; i >= 0; i--) {
        if (performanceEntries[i].entryType === "mark") {
          performanceEntries.splice(i, 1);
        }
      }
    }
  }

  /**
   * Removes performance measures from the timeline
   */
  clearMeasures(measureName?: string): void {
    if (measureName !== undefined) {
      if (typeof measureName !== "string") {
        throw new TypeError("Measure name must be a string");
      }
      // Remove specific measure
      for (let i = performanceEntries.length - 1; i >= 0; i--) {
        if (
          performanceEntries[i].entryType === "measure" &&
          performanceEntries[i].name === measureName
        ) {
          performanceEntries.splice(i, 1);
        }
      }
    } else {
      // Remove all measures
      for (let i = performanceEntries.length - 1; i >= 0; i--) {
        if (performanceEntries[i].entryType === "measure") {
          performanceEntries.splice(i, 1);
        }
      }
    }
  }

  /**
   * Returns a list of performance entries
   */
  getEntries(): PerformanceEntry[] {
    return [...performanceEntries];
  }

  /**
   * Returns a list of performance entries by type
   */
  getEntriesByType(type: string): PerformanceEntry[] {
    if (typeof type !== "string") {
      throw new TypeError("Entry type must be a string");
    }
    return performanceEntries.filter((entry) => entry.entryType === type);
  }

  /**
   * Returns a list of performance entries by name
   */
  getEntriesByName(name: string, type?: string): PerformanceEntry[] {
    if (typeof name !== "string") {
      throw new TypeError("Entry name must be a string");
    }
    return performanceEntries.filter((entry) => {
      return entry.name === name &&
        (type === undefined || entry.entryType === type);
    });
  }

  /**
   * Converts the Performance object to a JSON representation
   */
  toJSON(): object {
    return {
      timeOrigin: this.timeOrigin,
    };
  }
}

(globalThis as unknown as { performance: AndromedaPerformance }).performance =
  new AndromedaPerformance();
