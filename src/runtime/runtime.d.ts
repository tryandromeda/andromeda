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
 * The `_internal_read_file` function reads a file from the file system.
 */
declare function _internal_read_file(path: string): string;

/**
 * The `_internal_write_file` function writes a file to the file system.
 */
declare function _internal_write_file(path: string, data: string): void;

/**
 * The `_internal_write_text_file` function writes a text file to the file system.
 */
declare function _internal_write_text_file(path: string, data: string): void;

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
}
