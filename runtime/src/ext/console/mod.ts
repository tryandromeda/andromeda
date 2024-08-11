// deno-lint-ignore-file no-unused-vars

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
  log(message: string) {
    internal_print(message + "\n");
  },

  /**
   * debug function logs a debug message to the console.
   *
   * @example
   * ```ts
   * console.debug("Hello, World!");
   */
  debug(message: string) {
    internal_print("\x1b[36m" + message + "\x1b[0m\n");
  },

  /**
   * warn function logs a warning message to the console.
   *
   * @example
   * ```ts
   * console.warn("Hello, World!");
   * ```
   */
  warn(message: string) {
    internal_print("\x1b[33m" + message + "\x1b[0m\n");
  },

  /**
   *  error function logs a warning message to the console.
   *
   * @example
   * ```ts
   * console.error("Hello, World!");
   * ```
   */
  error(message: string) {
    internal_print("\x1b[31m" + message + "\x1b[0m\n");
  },

  /**
   *  info function logs an info message to the console.
   *
   * @example
   * ```ts
   * console.info("Hello, World!");
   * ```
   */
  info(message: string) {
    internal_print("\x1b[30m" + message + "\x1b[0m\n");
  },

  /**
   *  assert function tests if a condition is true.
   *
   * @example
   * ```ts
   * console.assert(1 === 1, "The condition is true!");
   * ```
   */
  assert(condition: boolean, message: string) {
    if (!condition) {
      internal_print("[assert]: " + message + "\n");
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
