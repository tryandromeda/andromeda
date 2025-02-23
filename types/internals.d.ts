/**
 * The `internal_read_text_file` function reads a file from the file system.
 */
declare function internal_read_text_file(path: string): string;

/**
 * The `internal_write_text_file` function writes a text file to the file system.
 */
declare function internal_write_text_file(path: string, data: string): void;

/**
 * The `internal_create_file` function creates a file in the file system.
 */
declare function internal_create_file(path: string): void;

/**
 * The `internal_copy_file` function copies a file in the file system.
 */
declare function internal_copy_file(source: string, destination: string): void;

/**
 * The `internal_mk_dir` function creates a directory in the file system.
 */
declare function internal_mk_dir(path: string): void;

/**
 * The `internal_exit` function exits the program with an optional exit code.
 */
declare function internal_exit(code: number): void;

/**
 * The `internal_read_line` function reads a line from standard input.
 */
declare function internal_read_line(): string;

/**
 * The `internal_write` function writes a string to standard output.
 */
declare function internal_write(message: string): void;

/**
 * The `internal_write_line` function writes a string to standard output followed by a newline.
 */
declare function internal_write_line(message: string): void;

/**
 * The `internal_file_open` function opens a File and returns a Rid.
 */
declare function internal_open_file(path: string): void;

/**
 * The `internal_sleep` function returns a Promise to be resolved after the specified time un milliseconds.
 */
declare function internal_sleep(duration: number): Promise<void>;

/**
 *  The `internal_print` function to log messages to the console.
 */
declare function internal_print(message: string): void;

/**
 * The `internal_get_cli_args` function to get the command line arguments.
 */
declare function internal_get_cli_args(): string[];

/**
 * The `internal_get_env` function to get the environment variable.
 */
declare function internal_get_env(key: string): string;

/**
 * The `internal_set_env` function to set the environment variable.
 */
declare function internal_set_env(key: string, value: string): void;

/**
 * The `internal_delete_env` function to delete the environment variable.
 */
declare function internal_delete_env(key: string): void;

/**
 * The `internal_get_env_keys` function to get the environment variable keys.
 */
declare function internal_get_env_keys(): string[];

/**
 * The `internal_url_parse` function to parse a URL string.
 */
declare function internal_url_parse(url: string, base: string): string;

/**
 * The `internal_url_parse_no_base` function to parse a URL string without a base URL.
 */
declare function internal_url_parse_no_base(url: string): string;

/**
 * The `internal_btoa` function encodes a string in base64.
 */
declare function internal_btoa(url: string): string;