// deno-lint-ignore-file no-unused-vars


/**
 * The `assert` function tests if a condition is `true`. If the condition is `false`, an error is thrown with an optional message.
 */
function assert(condition: boolean, message: string) {
  if (!condition) {
    throw new Error(message);
  }
}

/**
 * The `assertEquals` function tests if two values are equal.
 */
function assertEquals<A>(value1: A, value2: A, message: string) {
  if (value1 !== value2) {
    console.error(message);
  }
}

/**
 * The `assertNotEquals` function tests if two values are not equal.
 */
function assertNotEquals<A>(value1: A, value2: A, message: string) {
  if (value1 === value2) {
    console.error(message);
  }
}

/**
 * The `assertThrows` function tests if a function throws an error.
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
   * the `_internal_read_file` function reads a file from the filesystem.
   */
  readTextFileSync(path: string): string {
    return internal_read_text_file(path);
  },

  /**
   * The writeFileSync function writes data to a file on the filesystem.
   */
  writeTextFileSync(path: string, data: string): void {
    internal_write_text_file(path, data);
  },

  /**
   * The `exit` function exits the program with an optional exit code.
   */
  exit(code?: number): void {
    internal_exit(code || 0);
  },

  /**
   * The `sleep` function returns a Promise to be resolved after the specified time un milliseconds.
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
     */
    write(message: string): void {
      internal_write(message);
    },

    /**
     * The `writeLine` function writes a string followed by a newline to standard output.
     */
    writeLine(message: string): void {
      internal_write_line(message + "\n");
    },
  },
};

/**
 * The prompt function prompts the user for input.
 */
function prompt(message: string): string {
  console.log(message);
  return Andromeda.stdin.readLine();
}
