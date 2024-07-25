// deno-lint-ignore-file no-unused-vars

/**
 * The `console` module provides a simple debugging console that is similar to the JavaScript console mechanism provided by web browsers.
 */
const console = {
    /**
     *  log function logs a message to the console.
     */
    log(message: string) {
        internal_log(message);
    },

    /**
     * debug function logs a debug message to the console.
     */
    debug(message: string) {
        internal_log("[debug]: " + message);
    },

    /**
     * warn function logs a warning message to the console.
     */
    warn(message: string) {
        internal_log("[warn]: " + message);
    },

    /**
     *  error function logs a warning message to the console.
     */
    error(message: string) {
        internal_log("[error]: " + message);
    },
};