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
}

/**
 * The `Storage` class provides a way to store key/value pairs.
 */
declare interface Storage {
  /**
   * The number of items in storage.
   */
  readonly length: number;
  
  /**
   * `getItem `retrieves a value from storage.
   *
   * @example
   * ```ts
   * const value = storage.getItem("name");
   * console.log(value);
   * ```
   */
  getItem(key: string): string | null;

  /**
   * `setItem` stores a value in storage.
   *
   * @example
   * ```ts
   * storage.setItem("name", "Alice");
   * ```
   */
  setItem(key: string, value: string): void;

  /**
   * `removeItem` removes a value from storage.
   *
   * @example
   * ```ts
   * storage.removeItem("name");
   * ```
   */
  removeItem(key: string): void;

  /**
   * `key` retrieves a key from storage by index.
   */
  key(index: number): string | null;
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
