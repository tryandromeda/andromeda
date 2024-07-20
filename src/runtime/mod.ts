// deno-lint-ignore-file no-unused-vars

/**
 * The `console` module provides a simple debugging console that is similar to the JavaScript console mechanism provided by web browsers.
 */
const console = {
    /**
     *  Logs a message to the console.
     */
    log(message: string) {
        debug("[log]: " + message);
    },

    /**
     *  Logs a warning message to the console.
     */
    error(message: string) {
        debug("[error]: " + message);
    },
};

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
 * Marc namespace for the Andromeda runtime.
 */
const Marc = {
    /**
     * the `_internal_read_file` function reads a file from the filesystem.
     */
    readTextFileSync(path: string): string {
        return _internal_read_file(path);
    },

    /**
     * The writeFileSync function writes data to a file on the filesystem.
     */
    writeTextFileSync(path: string, data: string): void {
        _internal_write_text_file(path, data);
    }
};
