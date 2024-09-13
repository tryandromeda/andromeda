// deno-lint-ignore-file no-unused-vars
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
 * The `console` module provides a simple debugging console that is similar to the JavaScript console mechanism provided by web browsers.
 */
const console = {
  /**
   *  log function logs a message to the console.
   *
   * @example
   * ```ts
   * console.log("Hello, World!");
   * ```
   */
  log(...messages: string[]) {
    internal_print(messages.join("") + "\n");
  },

  /**
   * debug function logs a debug message to the console.
   *
   * @example
   * ```ts
   * console.debug("Hello, World!");
   */
  debug(...messages: string[]) {
    internal_print(COLORS.fg.cyan + messages.join("") + COLORS.reset + "\n");
  },

  /**
   * warn function logs a warning message to the console.
   *
   * @example
   * ```ts
   * console.warn("Hello, World!");
   * ```
   */
  warn(...messages: string[]) {
    internal_print(COLORS.fg.yellow + messages.join("") + COLORS.reset + "\n");
  },

  /**
   *  error function logs a warning message to the console.
   *
   * @example
   * ```ts
   * console.error("Hello, World!");
   * ```
   */
  error(...messages: string[]) {
    internal_print(COLORS.fg.red + messages.join("") + COLORS.reset + "\n");
  },

  /**
   *  info function logs an info message to the console.
   *
   * @example
   * ```ts
   * console.info("Hello, World!");
   * ```
   */
  info(...messages: string[]) {
    internal_print("\x1b[30m" + messages.join("") + COLORS.reset + "\n");
  },

  /**
   *  assert function tests if a condition is true.
   *
   * @example
   * ```ts
   * console.assert(1 === 1, "The condition is true!");
   * ```
   */
  assert(condition: boolean, ...messages: string[]) {
    if (!condition) {
      internal_print("Assertion Failed: " + messages.join("") + "\n");
    }
  },

  /**
   * clear function clears the console.
   *
   * @example
   * ```ts
   * console.clear();
   * ```
   */
  clear() {
    internal_print("\x1Bc");
  },
};
