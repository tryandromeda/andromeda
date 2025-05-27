// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
   * The `args` property contains the command-line arguments passed to the program.
   */
  args: internal_get_cli_args(),
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
   * The `createFileSync` function creates a file in the file system.
   *
   * @example
   * ```ts
   * Andromeda.createFileSync("hello.txt");
   * ```
   */
  createFileSync(path: string): void {
    internal_create_file(path);
  },

  /**
   * The `copyFileSync` function copies a file in the file system.
   *
   * @example
   * ```ts
   * Andromeda.copyFileSync("hello.txt", "world.txt");
   * ```
   */
  copyFileSync(source: string, destination: string): void {
    internal_copy_file(source, destination);
  },

  /**
   * The `mkdirSync` function creates a directory in the file system.
   *
   * @example
   * ```ts
   * Andromeda.mkdirSync("hello");
   * ```
   */
  mkdirSync(path: string): void {
    internal_mk_dir(path);
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
    return internal_sleep(duration);
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

  /**
   * env namespace for environment variables.
   */
  env: {
    /**
     * The `get` function gets the environment variable.
     *
     * @example
     * ```ts
     * const value = Andromeda.env.get("PATH");
     * console.log(value);
     * ```
     */
    get(key: string): string {
      return internal_get_env(key);
    },

    /**
     * The `set` function sets the environment variable.
     *
     * @example
     * ```ts
     * Andromeda.env.set("HI", "Hello, World!");
     * ```
     */
    set(key: string, value: string): void {
      internal_set_env(key, value);
    },

    /**
     * The `remove` function deletes the environment variable.
     *
     * @example
     * ```ts
     * Andromeda.env.delete("HI");
     * ```
     */
    remove(key: string): void {
      internal_delete_env(key);
    },

    /**
     * The `keys` function gets the environment variable keys.
     *
     * @example
     * ```ts
     * const keys = Andromeda.env.keys();
     * console.log(keys);
     * ```
     */
    keys(): string[] {
      return internal_get_env_keys();
    },
  },
};

/**
 * The `prompt` function prompts the user for input.
 *
 * @example
 * ```ts
 * const name = prompt("What is your name?");
 * console.log(`Hello, ${name}!`);
 * ```
 */
function prompt(message: string): string {
  internal_print(message + ": ");
  return Andromeda.stdin.readLine();
}

/**
 * The `confirm` function prompts the user for confirmation.
 *
 * @example
 * ```ts
 * if (confirm("Are you sure?")) {
 *  console.log("The user is sure!");
 * }
 * ```
 */
function confirm(message: string): boolean {
  internal_print(message + " [y/N]: ");
  const response = Andromeda.stdin.readLine();
  return response.includes("y");
}

/**
 * The `alert` function displays a message to the user and waits for the user to hit enter.
 */
function alert(message: string) {
  internal_print(message + " [Enter]");
  Andromeda.stdin.readLine();
}

/**
 * Takes the input data, in the form of a Unicode string containing only characters in the range U+0000 to U+00FF,
 * each representing a binary byte with values 0x00 to 0xFF respectively, and converts it to its base64 representation,
 * which it returns.
 */
function btoa(input: string): string {
  return internal_btoa(input);
}

/**
 * Takes the input data, in the form of a Unicode string containing base64-encoded binary data,
 * decodes it, and returns a string consisting of characters in the range U+0000 to U+00FF,
 * each representing a binary byte with values 0x00 to 0xFF respectively,
 * corresponding to that binary data.
 */
function atob(input: string): string {
  return internal_atob(input);
}
