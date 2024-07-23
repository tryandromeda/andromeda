/**
 * Debug function to log messages to the console.
 */
declare function debug(message: string): void;

/**
 * The `console` module provides a simple debugging console that is similar to the JavaScript console mechanism provided by web browsers.
 */
declare function assert(condition: boolean, message: string): void;

/**
 * The `assertEquals` function tests if two values are equal.
 */
declare function assertEquals<T>(value1: T, value2: T, message: string): void;

/**
 * The `assertNotEquals` function tests if two values are not equal.
 */
declare function assertNotEquals<T>(
  value1: T,
  value2: T,
  message: string,
): void;

/**
 * The `assertThrows` function tests if a function throws an error.
 */
declare function assertThrows(fn: () => void, message: string): void;

/**
 * The Andromeda namespace for the Andromeda runtime.
 */
declare namespace Andromeda {
  /**
   * readFileSync reads a file from the file system.
   */
  function readTextFileSync(path: string): string;

  /**
   * writeFileSync writes a file to the file system.
   */
  function writeTextFileSync(path: string, data: string): void;

  /**
   * exit exits the program with an optional exit code.
   */
  function exit(code?: number): void;

  namespace stdin {
    /**
     * readLine reads a line from standard input.
     */
    function readLine(): string;
  }

  /**
   * stdout namespace for writing to standard output.
   */
  namespace stdout {
    /**
     * write writes a string to standard output.
     */
    function write(message: string): void;
  }
}

/**
 * The `prompt` function prompts the user for input.
 */
declare function prompt(message: string): string;