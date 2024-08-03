// deno-lint-ignore-file no-unused-vars


/**
 * The `assert` function tests if a condition is `true`. If the condition is `false`, an error is thrown with an optional message.
 * 
 * @example
 * ```ts
 * assert(1 === 1, "The condition is true!");
 * ```
 */
function assert(condition: boolean, message: string) {
  if (!condition) {
    throw new Error(message);
  }
}

/**
 * The `assertEquals` function tests if two values are equal.
 * 
 * @example
 * ```ts
 * assertEquals(1, 1, "The values are equal!");
 * ```
 */
function assertEquals<A>(value1: A, value2: A, message: string) {
  if (value1 !== value2) {
    console.error(message);
  }
}

/**
 * The `assertNotEquals` function tests if two values are not equal.
 * 
 * @example
 * ```ts
 * assertNotEquals(1, 2, "The values are not equal!");
 * ```
 */
function assertNotEquals<A>(value1: A, value2: A, message: string) {
  if (value1 === value2) {
    console.error(message);
  }
}

/**
 * The `assertThrows` function tests if a function throws an error.
 * 
 * @example
 * ```ts
 * assertThrows(() => {
 *  throw new Error("Hello, World!");
 * }, "An error occurred!");
 */
function assertThrows(fn: () => void, message: string) {
  try {
    fn();
  } catch (error) {
    return;
  }

  console.error(message);
}

/**
 * Andromeda namespace for the Andromeda runtime.
 */
const Andromeda = {
  /**
   * the readFileSync function reads a file from the filesystem.
   * 
   * @example
   * ```ts
   * const data = Andromeda.readFileSync("hello.txt");
   * console.log(data);
   * ```
   */
  readTextFileSync(path: string): string {
    return internal_read_text_file(path);
  },

  /**
   * The writeFileSync function writes data to a file on the filesystem.
   * 
   * @example
   * ```ts
   * Andromeda.writeFileSync("hello.txt", "Hello, World!");
   * ```
   */
  writeTextFileSync(path: string, data: string): void {
    internal_write_text_file(path, data);
  },

  /**
   * The `exit` function exits the program with an optional exit code.
   * 
   * @example
   * ```ts
   * Andromeda.exit(0);
   * ```
   */
  exit(code?: number): void {
    internal_exit(code || 0);
  },

  /**
   * The `sleep` function returns a Promise to be resolved after the specified time in milliseconds.
   * 
   * @example
   * ```ts
   * Andromeda.sleep(1000).then(() => {
   *  console.log("Hello, World!");
   * });
   * ```
   */
  sleep(duration: number): Promise<void> {
    return internal_sleep(duration)
  },

  /**
   * stdin namespace for reading from standard input.
   */
  stdin: {
    /**
     * The `readLine` function reads a line from standard input.
     * 
     * @example
     * ```ts
     * const name = Andromeda.stdin.readLine();
     * console.log(`Hello, ${name}!`);
     * ```
     */
    readLine(): string {
      return internal_read_line();
    },
  },

  /**
   * stdout namespace for writing to standard output.
   */
  stdout: {
    /**
     * The `write` function writes a string to standard output.
     * 
     * @example
     * ```ts
     * Andromeda.stdout.write("Hello, World!");
     * ```
     */
    write(message: string): void {
      internal_write(message);
    },

    /**
     * The `writeLine` function writes a string followed by a newline to standard output.
     * 
     * @example
     * ```ts
     * Andromeda.stdout.writeLine("Hello, World!");
     * ```
     */
    writeLine(message: string): void {
      internal_write_line(message + "\n");
    },
  },
};

/**
 * The prompt function prompts the user for input.
 * 
 * @example
 * ```ts
 * const name = prompt("What is your name?");
 * console.log(`Hello, ${name}!`);
 * ```
 */
function prompt(message: string): string {
  internal_print(message+ ": ");
  return Andromeda.stdin.readLine();
}
