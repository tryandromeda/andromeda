// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * The `assert` function tests if a condition is true.
 *
 * @example
 * ```ts
 * assert(1 === 1, "The condition is true!");
 * ```
 */
declare function assert(condition: boolean, message: string): void;

/**
 * The `assertEquals` function tests if two values are equal.
 *
 * @example
 * ```ts
 * assertEquals(1, 1, "The values are equal!");
 * ```
 */
declare function assertEquals<T>(value1: T, value2: T, message: string): void;

/**
 * The `assertNotEquals` function tests if two values are not equal.
 *
 * @example
 * ```ts
 * assertNotEquals(1, 2, "The values are not equal!");
 * ```
 */
declare function assertNotEquals<T>(
  value1: T,
  value2: T,
  message: string,
): void;

/**
 * The `assertThrows` function tests if a function throws an error.
 *
 * @example
 * ```ts
 * assertThrows(() => {
 *  throw new Error("Hello, World!");
 * }, "An error occurred!");
 */
declare function assertThrows(fn: () => void, message: string): void;

/**
 * The Andromeda namespace for the Andromeda runtime.
 */
declare namespace Andromeda {
  /**
   * The `args` property contains the command-line arguments.
   */
  const args: string[];
  /**
   * readFileSync reads a file from the file system.
   *
   * @example
   * ```ts
   * const data = Andromeda.readFileSync("hello.txt");
   * console.log(data);
   * ```
   */
  function readTextFileSync(path: string): string;

  /**
   * writeFileSync writes a file to the file system.
   *
   * @example
   * ```ts
   * Andromeda.writeFileSync("hello.txt", "Hello, World!");
   * ```
   */
  function writeTextFileSync(path: string, data: string): void;

  /**
   * exit exits the program with an optional exit code.
   *
   * @example
   * ```ts
   * Andromeda.exit(0);
   * ```
   */
  function exit(code?: number): void;

  /**
   * Returns a Promise to be resolved after the specified time un milliseconds.
   *
   * @example
   * ```ts
   * Andromeda.sleep(1000).then(() => {
   *  console.log("Hello, World!");
   * });
   */
  function sleep(duration: number): Promise<void>;

  namespace stdin {
    /**
     * readLine reads a line from standard input.
     *
     * @example
     * ```ts
     * const name = Andromeda.stdin.readLine();
     * console.log(`Hello, ${name}!`);
     * ```
     */
    function readLine(): string;
  }

  /**
   * stdout namespace for writing to standard output.
   */
  namespace stdout {
    /**
     * write writes a string to standard output.
     *
     * @example
     * ```ts
     * Andromeda.stdout.write("Hello, World!");
     * ```
     */
    function write(message: string): void;
  }

  /**
   * env namespace for environment variables.
   */
  namespace env {
    /**
     * get returns the value of an environment variable.
     *
     * @example
     * ```ts
     * const value = Andromeda.env.get("HOME");
     * console.log(value);
     * ```
     */
    function get(key: string): string;

    /**
     * set sets the value of an environment variable.
     *
     * @example
     * ```ts
     * Andromeda.env.set("HOME", "/home/user");
     * ```
     */
    function set(key: string, value: string): void;

    /**
     * remove deletes an environment variable.
     *
     * @example
     * ```ts
     * Andromeda.env.remove("HOME");
     * ```
     */
    function remove(key: string): void;

    /**
     * keys returns the keys of all environment variables.
     *
     * @example
     * ```ts
     * const keys = Andromeda.env.keys();
     * console.log(keys);
     * ```
     */
    function keys(): string[];
  }
}
/**
 * The `prompt` function prompts the user for input.
 *
 * @example
 * ```ts
 * const name = prompt("What is your name?");
 * console.log(`Hello, ${name}!`);
 * ```
 */
declare function prompt(message: string): string;

/**
 * The `confirm` function prompts the user for confirmation.
 */
declare function confirm(message: string): boolean;

/**
 * An offscreen Canvas implementation.
 */
declare class Canvas {
  /**
   * Create a new off-screen canvas with the given dimensions.
   */
  constructor(width: number, height: number);
  /** Get the width of the canvas. */
  getWidth(): number;
  /** Get the height of the canvas. */
  getHeight(): number;
  /**
   * Returns a 2D rendering context or null if not available.
   */
  getContext(type: "2d"): CanvasRenderingContext2D | null;
}

/**
 * Factory to create a Canvas instance.
 */
declare function createCanvas(width: number, height: number): Canvas;

/**
 * The 2D rendering context for a Canvas.
 */
declare class CanvasRenderingContext2D {
  /**
   * Draws a filled rectangle whose starting corner is at (x, y).
   */
  fillRect(x: number, y: number, width: number, height: number): void;
  /**
   * Clears the specified rectangular area, making it fully transparent.
   */
  clearRect(x: number, y: number, width: number, height: number): void;
}


/**
 * Creates an ImageBitmap from a file path or URL.
 * @param path The file path or URL to load.
 */
declare function createImageBitmap(path: string): Promise<ImageBitmap>;


