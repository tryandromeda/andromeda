// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// External function declarations for runtime operations

// Type definitions
type ConsoleValue =
  | string
  | number
  | boolean
  | null
  | undefined
  | object
  | ConsoleValue[];

// ANSI color codes for styling output
const COLORS = {
  reset: "\x1b[0m",
  bright: "\x1b[1m",
  dim: "\x1b[2m",
  underscore: "\x1b[4m",
  blink: "\x1b[5m",
  reverse: "\x1b[7m",
  hidden: "\x1b[8m",
  fg: {
    black: "\x1b[30m",
    red: "\x1b[31m",
    green: "\x1b[32m",
    yellow: "\x1b[33m",
    blue: "\x1b[34m",
    magenta: "\x1b[35m",
    cyan: "\x1b[36m",
    white: "\x1b[37m",
    gray: "\x1b[90m",
  },
  bg: {
    black: "\x1b[40m",
    red: "\x1b[41m",
    green: "\x1b[42m",
    yellow: "\x1b[43m",
    blue: "\x1b[44m",
    magenta: "\x1b[45m",
    cyan: "\x1b[46m",
    white: "\x1b[47m",
  },
};

/**
 * Utility functions for formatting
 */
function getIndent(): string {
  const indentLevel = __andromeda__.get_group_indent();
  return "  ".repeat(indentLevel);
}

function formatValue(value: ConsoleValue): string {
  if (value === null) return "null";
  if (value === undefined) return "undefined";
  if (typeof value === "string") return value;
  if (typeof value === "number") return value.toString();
  if (typeof value === "boolean") return value.toString();
  if (typeof value === "function") {
    const funcName = (value as unknown as Record<string, unknown>).name;
    return `[Function: ${
      typeof funcName === "string" ? funcName : "anonymous"
    }]`;
  }
  if (Array.isArray(value)) {
    return `[${value.map(formatValueForContainer).join(", ")}]`;
  }
  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, ConsoleValue>).map((
      [k, v],
    ) => `${k}: ${formatValueForContainer(v)}`);
    return `{ ${entries.join(", ")} }`;
  }
  return String(value);
}

function formatValueForContainer(value: ConsoleValue): string {
  if (value === null) return "null";
  if (value === undefined) return "undefined";
  if (typeof value === "string") return `"${value}"`; // Strings should be quoted in arrays/objects
  if (typeof value === "number") return value.toString();
  if (typeof value === "boolean") return value.toString();
  if (typeof value === "function") {
    const funcName = (value as unknown as Record<string, unknown>).name;
    return `[Function: ${
      typeof funcName === "string" ? funcName : "anonymous"
    }]`;
  }
  if (Array.isArray(value)) {
    return `[${value.map(formatValueForContainer).join(", ")}]`;
  }
  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, ConsoleValue>).map((
      [k, v],
    ) => `${k}: ${formatValueForContainer(v)}`);
    return `{ ${entries.join(", ")} }`;
  }
  return String(value);
}

function formatArgs(args: ConsoleValue[]): string {
  if (args.length === 0) return "";

  // Handle format specifiers in the first argument if it's a string
  const first = args[0];
  const rest = args.slice(1);

  if (typeof first === "string" && rest.length > 0) {
    // Enhanced format specifier support following WHATWG spec with CSS styling
    const formatted = first;
    let argIndex = 0;
    let result = "";
    let i = 0;
    const currentStyles: string[] = []; // Stack of active styles

    while (i < formatted.length) {
      if (formatted[i] === "%" && i + 1 < formatted.length) {
        const nextChar = formatted[i + 1];
        if (nextChar === "%") {
          result += "%";
          i += 2;
        } else if (argIndex < rest.length) {
          const arg = rest[argIndex];
          let converted = false;

          switch (nextChar) {
            case "s": // String
              result += String(arg);
              converted = true;
              break;
            case "d":
            case "i": // Integer
              // If current is a Symbol, let converted be NaN
              if (typeof arg === "symbol") {
                result += "NaN";
              } else {
                result += String(parseInt(String(arg), 10) || 0);
              }
              converted = true;
              break;
            case "f": // Float
              // If current is a Symbol, let converted be NaN
              if (typeof arg === "symbol") {
                result += "NaN";
              } else {
                result += String(parseFloat(String(arg)) || 0);
              }
              converted = true;
              break;
            case "o": // Optimally useful formatting
              result += formatOptimallyUseful(arg);
              converted = true;
              break;
            case "O": // Generic JavaScript object formatting
              result += formatGenericObject(arg);
              converted = true;
              break;
            case "c": // CSS styling
              if (typeof arg === "string") {
                // Convert CSS to ANSI and add to result
                const ansiCode = __andromeda__.internal_css_to_ansi(arg);
                if (ansiCode) {
                  result += ansiCode;
                  currentStyles.push(ansiCode);
                }
              }
              converted = true;
              break;
          }

          if (converted) {
            argIndex++;
            i += 2;
          } else {
            result += formatted[i];
            i++;
          }
        } else {
          result += formatted[i];
          i++;
        }
      } else {
        result += formatted[i];
        i++;
      }
    }

    // Add reset codes to clear any applied styles at the end
    if (currentStyles.length > 0) {
      result += COLORS.reset;
    }

    const remainingArgs = rest.slice(argIndex);
    if (remainingArgs.length > 0) {
      return result + " " + remainingArgs.map(formatValue).join(" ");
    }
    return result;
  }

  return args.map(formatValue).join(" ");
}

// Optimally useful formatting for %o specifier
function formatOptimallyUseful(value: ConsoleValue): string {
  if (value === null) return "null";
  if (value === undefined) return "undefined";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  if (typeof value === "function") {
    const funcName = (value as unknown as Record<string, unknown>).name;
    return `ƒ ${typeof funcName === "string" ? funcName : "anonymous"}()`;
  }
  if (Array.isArray(value)) {
    if (value.length <= 5) {
      return `(${value.length}) [${
        value.map(formatValueForContainer).join(", ")
      }]`;
    } else {
      return `(${value.length}) [${
        value.slice(0, 3).map(formatValueForContainer).join(", ")
      }, …]`;
    }
  }
  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, ConsoleValue>);
    if (entries.length <= 3) {
      return `{${
        entries.map(([k, v]) => `${k}: ${formatValueForContainer(v)}`).join(
          ", ",
        )
      }}`;
    } else {
      const preview = entries.slice(0, 2).map(([k, v]) =>
        `${k}: ${formatValueForContainer(v)}`
      );
      return `{${preview.join(", ")}, …}`;
    }
  }
  return String(value);
}

// Generic JavaScript object formatting for %O specifier
function formatGenericObject(value: ConsoleValue): string {
  if (value === null) return "null";
  if (value === undefined) return "undefined";
  if (typeof value === "string") return `"${value}"`;
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  if (typeof value === "function") {
    const funcName = (value as unknown as Record<string, unknown>).name;
    return `[Function: ${
      typeof funcName === "string" ? funcName : "anonymous"
    }]`;
  }
  if (Array.isArray(value)) {
    return `[${value.map(formatValueForContainer).join(", ")}]`;
  }
  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, ConsoleValue>).map((
      [k, v],
    ) => `${k}: ${formatValueForContainer(v)}`);
    return `{ ${entries.join(", ")} }`;
  }
  return String(value);
}

function createTable(data: ConsoleValue[], headers?: string[]): string {
  if (!Array.isArray(data) || data.length === 0) {
    return formatValue(data);
  }

  // Simple table implementation
  const table: string[][] = [];
  const firstItem = data[0];
  const cols = headers ||
    (typeof firstItem === "object" && firstItem !== null ?
      Object.keys(firstItem as Record<string, ConsoleValue>) :
      []);

  // Add header row
  table.push(["(index)", ...cols]);

  // Add data rows
  data.forEach((row, index) => {
    const tableRow = [index.toString()];
    cols.forEach((col) => {
      const value = row && typeof row === "object" ?
        (row as Record<string, ConsoleValue>)[col] :
        "";
      tableRow.push(formatValue(value));
    });
    table.push(tableRow);
  });

  // Calculate column widths
  const widths = table[0].map((_, colIndex) =>
    Math.max(...table.map((row) => String(row[colIndex] || "").length))
  );

  // Format table
  return table.map((row) =>
    row.map((cell, colIndex) => String(cell || "").padEnd(widths[colIndex]))
      .join(" | ")
  ).join("\n");
}

/**
 * WHATWG-compliant Console API implementation for Andromeda
 *
 * This implementation provides a comprehensive console interface following
 * the JavaScript console mechanism provided by web browsers.
 *
 * This implementation follows the WHATWG Console specification:
 * https://console.spec.whatwg.org/
 */
const andromedaConsole = {
  /**
   * Logs a message to the console.
   * Supports format specifiers: %s (string), %d/%i (integer), %f (float), %% (literal %)
   *
   * @example
   * ```ts
   * console.log("Hello, World!");
   * console.log("User %s is %d years old", "John", 25);
   * ```
   */
  log(...args: ConsoleValue[]) {
    const message = getIndent() + formatArgs(args);
    __andromeda__.internal_print(message + "\n");
  },

  /**
   * Logs a debug message to the console.
   *
   * @example
   * ```ts
   * console.debug("Debug information", { value: 42 });
   * ```
   */
  debug(...args: ConsoleValue[]) {
    const message = getIndent() + COLORS.fg.cyan + formatArgs(args) +
      COLORS.reset;
    __andromeda__.internal_print(message + "\n");
  },

  /**
   * Logs an informational message to the console.
   *
   * @example
   * ```ts
   * console.info("Information message");
   * ```
   */
  info(...args: ConsoleValue[]) {
    const message = getIndent() + COLORS.fg.blue + formatArgs(args) +
      COLORS.reset;
    __andromeda__.internal_print(message + "\n");
  },

  /**
   * Logs a warning message to the console.
   *
   * @example
   * ```ts
   * console.warn("This is a warning!");
   * ```
   */
  warn(...args: ConsoleValue[]) {
    const message = getIndent() + COLORS.fg.yellow + formatArgs(args) +
      COLORS.reset;
    __andromeda__.internal_print_err(message + "\n");
  },

  /**
   * Logs an error message to the console.
   *
   * @example
   * ```ts
   * console.error("An error occurred!", error);
   * ```
   */
  error(...args: ConsoleValue[]) {
    const message = getIndent() + COLORS.fg.red + formatArgs(args) +
      COLORS.reset;
    __andromeda__.internal_print_err(message + "\n");
  },

  /**
   * Tests if a condition is true. If not, logs an assertion failure message.
   * Follows WHATWG Console specification behavior.
   *
   * @example
   * ```ts
   * console.assert(1 === 1, "Math works correctly");
   * console.assert(false, "This will log an error");
   * ```
   */
  assert(condition?: boolean, ...args: ConsoleValue[]) {
    if (condition) return; // Early return if condition is true

    let message: string;
    if (args.length === 0) {
      message = "Assertion failed";
    } else {
      const first = args[0];
      if (typeof first === "string") {
        // Prepend "Assertion failed: " to the first argument if it's a string
        const modifiedArgs = [`Assertion failed: ${first}`, ...args.slice(1)];
        message = formatArgs(modifiedArgs);
      } else {
        // Prepend "Assertion failed" as a separate argument
        const modifiedArgs = ["Assertion failed", ...args];
        message = formatArgs(modifiedArgs);
      }
    }

    // Use error level for assertion failures per WHATWG spec
    const errorMessage = getIndent() + COLORS.fg.red + message + COLORS.reset;
    __andromeda__.internal_print_err(errorMessage + "\n");
  },

  /**
   * Clears the console and resets the group stack.
   * Follows WHATWG Console specification.
   *
   * @example
   * ```ts
   * console.clear();
   * ```
   */
  clear() {
    // Clear console through the backend (which also resets group stack per WHATWG spec)
    __andromeda__.clear_console();
  },

  /**
   * Logs a count of how many times this line has been called with the given label.
   * Follows WHATWG Console specification behavior.
   *
   * @example
   * ```ts
   * console.count(); // default: 1
   * console.count("myLabel"); // myLabel: 1
   * console.count("myLabel"); // myLabel: 2
   * ```
   */
  count(label: string = "default") {
    const message = __andromeda__.count(label);
    console.info(message);
  },

  /**
   * Resets the count for the given label.
   * Warns if the label doesn't exist, following WHATWG spec.
   *
   * @example
   * ```ts
   * console.countReset("myLabel");
   * ```
   */
  countReset(label: string = "default") {
    const message = __andromeda__.count_reset(label);
    if (message.includes("does not exist")) {
      console.warn(message);
    } else {
      console.info(message);
    }
  },

  /**
   * Creates a new inline group in the console output.
   *
   * @example
   * ```ts
   * console.group("My Group");
   * console.log("Inside group");
   * console.groupEnd();
   * ```
   */
  group(...args: ConsoleValue[]) {
    if (args.length > 0) {
      console.log(formatArgs(args));
    }
    __andromeda__.group_start(args.length > 0 ? String(args[0]) : "");
  },

  /**
   * Creates a new inline group in the console output that is initially collapsed.
   * In this implementation, it behaves the same as group().
   *
   * @example
   * ```ts
   * console.groupCollapsed("Collapsed Group");
   * console.log("This is inside");
   * console.groupEnd();
   * ```
   */
  groupCollapsed(...args: ConsoleValue[]) {
    console.group(...args);
  },

  /**
   * Exits the current inline group.
   *
   * @example
   * ```ts
   * console.group("Group");
   * console.log("Inside");
   * console.groupEnd(); // Exit group
   * ```
   */
  groupEnd() {
    __andromeda__.group_end();
  },

  /**
   * Displays an interactive table with data.
   *
   * @example
   * ```ts
   * console.table([{name: "John", age: 30}, {name: "Jane", age: 25}]);
   * console.table([1, 2, 3, 4]);
   * ```
   */
  table(tabularData?: ConsoleValue, properties?: string[]) {
    if (tabularData === null || tabularData === undefined) {
      console.log(tabularData);
      return;
    }

    if (Array.isArray(tabularData)) {
      console.log(createTable(tabularData, properties));
    } else if (typeof tabularData === "object") {
      const entries = Object.entries(
        tabularData as Record<string, ConsoleValue>,
      ).map(([key, value]) => ({ Key: key, Value: value }));
      console.log(createTable(entries));
    } else {
      console.log(tabularData);
    }
  },

  /**
   * Displays an interactive listing of the properties of the specified object.
   *
   * @example
   * ```ts
   * console.dir({ name: "John", age: 30 });
   * ```
   */
  dir(obj?: ConsoleValue, _options?: Record<string, ConsoleValue>) {
    // Simple implementation - in a full implementation this would show
    // an expandable object representation
    console.log(formatValue(obj));
  },

  /**
   * Displays an XML/HTML Element representation of the specified object if possible
   * or the JavaScript Object representation if not.
   *
   * @example
   * ```ts
   * console.dirxml(document.body); // In browser context
   * ```
   */
  dirxml(...args: ConsoleValue[]) {
    // For non-browser environments, this behaves like dir()
    args.forEach((arg) => console.dir(arg));
  },

  /**
   * Starts a timer with the specified label.
   * Warns if timer already exists, following WHATWG spec.
   *
   * @example
   * ```ts
   * console.time("myTimer");
   * // ... some operation
   * console.timeEnd("myTimer");
   * ```
   */
  time(label: string = "default") {
    const result = __andromeda__.time_start(label);
    if (result) {
      console.warn(result);
    }
  },

  /**
   * Logs the current value of a timer.
   * Warns if timer doesn't exist, following WHATWG spec.
   *
   * @example
   * ```ts
   * console.time("myTimer");
   * setTimeout(() => console.timeLog("myTimer", "checkpoint"), 100);
   * ```
   */
  timeLog(label: string = "default", ...args: ConsoleValue[]) {
    const result = __andromeda__.time_log(
      label,
      args.length > 0 ? formatArgs(args) : "",
    );
    if (result.includes("does not exist")) {
      console.warn(result);
    } else {
      console.info(result);
    }
  },

  /**
   * Stops a timer and logs the final time.
   * Warns if timer doesn't exist, following WHATWG spec.
   *
   * @example
   * ```ts
   * console.time("myTimer");
   * // ... some operation
   * console.timeEnd("myTimer"); // myTimer: 42ms
   * ```
   */
  timeEnd(label: string = "default") {
    const result = __andromeda__.time_end(label);
    if (result.includes("does not exist")) {
      console.warn(result);
    } else {
      console.info(result);
    }
  },

  /**
   * Logs a stack trace to the console.
   *
   * @example
   * ```ts
   * console.trace("Trace point reached");
   * ```
   */
  trace(...args: ConsoleValue[]) {
    const message = args.length > 0 ? formatArgs(args) : "Trace";
    const stack = __andromeda__.get_stack_trace();
    console.log(`${message}\n${stack}`);
  },

  /**
   * Starts a profile (no-op in this implementation).
   * This method exists for compatibility.
   */
  profile(_label?: string) {
    // No-op in this implementation
  },

  /**
   * Ends a profile (no-op in this implementation).
   * This method exists for compatibility.
   */
  profileEnd(_label?: string) {
    // No-op in this implementation
  },

  /**
   * Adds a timestamp to the console (no-op in this implementation).
   * This method exists for compatibility.
   */
  timeStamp(_label?: string) {
    // No-op in this implementation
  },
};

// // Export the console object
globalThis.console = andromedaConsole;
