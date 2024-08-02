/**
 * The `internal_read_text_file` function reads a file from the file system.
 */
declare function internal_read_text_file(path: string): string;

/**
 * The `internal_write_text_file` function writes a text file to the file system.
 */
declare function internal_write_text_file(path: string, data: string): void;

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
 * The `internal_sqlite_execute` function executes a SQL statement on the internal SQLite database.
 */
declare function internal_sqlite_execute(statement: string): void;
