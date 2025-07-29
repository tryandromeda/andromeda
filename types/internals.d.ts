// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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
 * The `internal_mk_dir_all` function creates a directory and all its parent directories in the file system.
 */
declare function internal_mk_dir_all(path: string): void;

/**
 * The `internal_read_file` function reads a file as binary data from the file system.
 */
declare function internal_read_file(path: string): Uint8Array;

/**
 * The `internal_write_file` function writes binary data to a file in the file system.
 */
declare function internal_write_file(path: string, data: Uint8Array): void;

/**
 * The `internal_remove` function removes a file from the file system.
 */
declare function internal_remove(path: string): void;

/**
 * The `internal_remove_all` function recursively removes a file or directory from the file system.
 */
declare function internal_remove_all(path: string): void;

/**
 * The `internal_rename` function renames/moves a file or directory in the file system.
 */
declare function internal_rename(oldPath: string, newPath: string): void;

/**
 * The `internal_exists` function checks if a file or directory exists in the file system.
 */
declare function internal_exists(path: string): boolean;

/**
 * The `internal_truncate` function truncates a file to a specified length.
 */
declare function internal_truncate(path: string, length: number): void;

/**
 * The `internal_chmod` function changes the permissions of a file or directory.
 */
declare function internal_chmod(path: string, mode: number): void;

/**
 * The `internal_symlink` function creates a symbolic link.
 */
declare function internal_symlink(target: string, linkPath: string): void;

/**
 * The `internal_read_link` function reads the target of a symbolic link.
 */
declare function internal_read_link(path: string): string;

/**
 * The `internal_real_path` function resolves the absolute path of a file, resolving symbolic links.
 */
declare function internal_real_path(path: string): string;

/**
 * The `internal_read_dir` function reads the contents of a directory.
 */
declare function internal_read_dir(
  path: string,
): Array<
  { name: string; isFile: boolean; isDirectory: boolean; isSymlink: boolean; }
>;

/**
 * The `internal_stat` function gets information about a file or directory.
 */
declare function internal_stat(path: string): {
  isFile: boolean;
  isDirectory: boolean;
  isSymlink: boolean;
  size: number;
  modified: number;
  accessed: number;
  created: number;
  mode: number;
};

/**
 * The `internal_lstat` function gets information about a file or directory without following symbolic links.
 */
declare function internal_lstat(path: string): {
  isFile: boolean;
  isDirectory: boolean;
  isSymlink: boolean;
  size: number;
  modified: number;
  accessed: number;
  created: number;
  mode: number;
};

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
 * The `internal_open_file` function opens a file and returns a file descriptor.
 */
declare function internal_open_file(path: string, mode: string): number;

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
 * The `internal_url_get_protocol` function returns the protocol (scheme) of a URL string.
 */
declare function internal_url_get_protocol(url: string): string;

/**
 * The `internal_url_get_username` function returns the username of a URL string.
 */
declare function internal_url_get_username(url: string): string;

/**
 * The `internal_url_get_password` function returns the password of a URL string.
 */
declare function internal_url_get_password(url: string): string;

/**
 * The `internal_url_get_host` function returns the host of a URL string.
 */
declare function internal_url_get_host(url: string): string;

/**
 * The `internal_url_get_hostname` function returns the hostname of a URL string.
 */
declare function internal_url_get_hostname(url: string): string;

/**
 * The `internal_url_get_port` function returns the port of a URL string.
 */
declare function internal_url_get_port(url: string): string;

/**
 * The `internal_url_get_pathname` function returns the pathname of a URL string.
 */
declare function internal_url_get_pathname(url: string): string;

/**
 * The `internal_url_get_search` function returns the search of a URL string.
 */
declare function internal_url_get_search(url: string): string;

/**
 * The `internal_url_get_hash` function returns the hash of a URL string.
 */
declare function internal_url_get_hash(url: string): string;

/**
 * The `internal_btoa` function encodes a string in base64.
 */
declare function internal_btoa(input: string): string;

/**
 * The `internal_atob` function decodes a string in base64.
 */
declare function internal_atob(input: string): string;

/**
 * The `internal_canvas_create` function creates a canvas with the specified width and height.
 */
declare function internal_canvas_create(width: number, height: number): number;

/**
 * The `internal_canvas_get_width` function gets the width of the canvas.
 */
declare function internal_canvas_get_width(rid: number): number;

/**
 * The `internal_canvas_get_height` function gets the height of the canvas.
 */
declare function internal_canvas_get_height(rid: number): number;
/**
 * The `internal_canvas_arc` function creates an arc on the canvas.
 */
declare function internal_canvas_arc(
  rid: number,
  x: number,
  y: number,
  radius: number,
  start_angle: number,
  end_angle: number,
): void;

/**
 * The `internal_canvas_arc_to` function creates an arc on the canvas.
 */
declare function internal_canvas_arc_to(
  rid: number,
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  radius: number,
): void;

/**
 * The `internal_canvas_begin_path` function begins a new path on the canvas.
 */
declare function internal_canvas_begin_path(rid: number): void;

/**
 * The `internal_canvas_bezier_curve_to` function creates a bezier curve on the canvas.
 */
declare function internal_canvas_bezier_curve_to(
  rid: number,
  cp1x: number,
  cp1y: number,
  cp2x: number,
  cp2y: number,
  x: number,
  y: number,
): void;
/**
 * Clears the specified rectangle on the canvas.
 */
declare function internal_canvas_clear_rect(
  rid: number,
  x: number,
  y: number,
  width: number,
  height: number,
): void;

/**
 * The `internal_canvas_close_path` function closes the current path on the canvas.
 */
declare function internal_canvas_close_path(rid: number): void;
/**
 * Draws a filled rectangle on the specified canvas.
 */
declare function internal_canvas_fill_rect(
  rid: number,
  x: number,
  y: number,
  width: number,
  height: number,
): void;

/**
 * The `internal_canvas_render` function renders the canvas to finalize GPU operations.
 * Returns true if rendering was successful, false otherwise.
 */
declare function internal_canvas_render(rid: number): boolean;

/**
 * The `internal_canvas_save_as_png` function saves the canvas as a PNG file.
 * Returns true if save was successful, false otherwise.
 */
declare function internal_canvas_save_as_png(
  rid: number,
  path: string,
): boolean;

/**
 * The `internal_canvas_get_fill_style` function gets the current fill style of the canvas context.
 * Returns the fill style as a CSS color string.
 */
declare function internal_canvas_get_fill_style(rid: number): string | number;

/**
 * The `internal_canvas_set_fill_style` function sets the fill style of the canvas context.
 * Accepts CSS color strings like '#ff0000', 'rgb(255, 0, 0)', 'rgba(255, 0, 0, 0.5)', 'red', etc.
 */
declare function internal_canvas_set_fill_style(
  rid: number,
  style: string | number,
): void;

/**
 * The `internal_canvas_move_to` function moves the path starting point to the specified coordinates.
 */
declare function internal_canvas_move_to(
  rid: number,
  x: number,
  y: number,
): void;

/**
 * The `internal_canvas_line_to` function connects the last point in the current sub-path to the specified coordinates with a straight line.
 */
declare function internal_canvas_line_to(
  rid: number,
  x: number,
  y: number,
): void;

/**
 * The `internal_canvas_fill` function fills the current path with the current fill style.
 */
declare function internal_canvas_fill(rid: number): void;

/**
 * The `internal_canvas_stroke` function strokes the current path with the current stroke style.
 */
declare function internal_canvas_stroke(rid: number): void;

/**
 * The `internal_canvas_rect` function adds a rectangle to the current path.
 */
declare function internal_canvas_rect(
  rid: number,
  x: number,
  y: number,
  width: number,
  height: number,
): void;

/**
 * The `internal_canvas_set_line_width` function sets the line width for stroking on the canvas.
 */
declare function internal_canvas_set_line_width(
  rid: number,
  lineWidth: number,
): void;

/**
 * The `internal_canvas_set_stroke_style` function sets the stroke style of the canvas context.
 * Accepts CSS color strings like '#ff0000', 'rgb(255, 0, 0)', 'rgba(255, 0, 0, 0.5)', 'red', etc.
 */
declare function internal_canvas_set_stroke_style(
  rid: number,
  style: string,
): void;

/**
 * The `internal_canvas_get_global_alpha` function gets the global alpha value of the canvas context.
 * Returns the global alpha as an integer (scaled by 1000).
 */
declare function internal_canvas_get_global_alpha(rid: number): number;

/**
 * The `internal_canvas_set_global_alpha` function sets the global alpha value of the canvas context.
 * Accepts a global alpha value as an integer (scaled by 1000).
 */
declare function internal_canvas_set_global_alpha(
  rid: number,
  alpha: number,
): void;

/**
 * The `internal_image_bitmap_create` function creates an ImageBitmap resource and returns its Rid.
 */
declare function internal_image_bitmap_create(path: string): number;
/**
 * The `internal_image_bitmap_get_width` function returns the width of the ImageBitmap resource.
 */
declare function internal_image_bitmap_get_width(rid: number): number;
/**
 * The `internal_image_bitmap_get_height` function returns the height of the ImageBitmap resource.
 */
declare function internal_image_bitmap_get_height(rid: number): number;
/**
 * The `internal_canvas_quadratic_curve_to` function creates a quadratic curve on the canvas.
 */
declare function internal_canvas_quadratic_curve_to(
  rid: number,
  cpx: number,
  cpy: number,
  x: number,
  y: number,
): void;

/**
 * The `internal_canvas_ellipse` function creates an ellipse on the canvas.
 */
declare function internal_canvas_ellipse(
  rid: number,
  x: number,
  y: number,
  radiusX: number,
  radiusY: number,
  rotation: number,
  startAngle: number,
  endAngle: number,
  counterclockwise?: boolean,
): void;

/**
 * The `internal_canvas_round_rect` function adds a rounded rectangle to the current path.
 */
declare function internal_canvas_round_rect(
  rid: number,
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number,
): void;

/**
 * The `internal_canvas_save` function saves the current canvas state (styles, transformations, etc.).
 */
declare function internal_canvas_save(rid: number): void;

/**
 * The `internal_canvas_restore` function restores the most recently saved canvas state.
 */
declare function internal_canvas_restore(rid: number): void;

/**
 * The `internal_canvas_get_stroke_style` function gets the current stroke style of the canvas context.
 * Returns the stroke style as a CSS color string.
 */
declare function internal_canvas_get_stroke_style(rid: number): string;

/**
 * The `internal_canvas_get_line_width` function gets the current line width of the canvas context.
 * Returns the line width as a number.
 */
declare function internal_canvas_get_line_width(rid: number): number;

/**
 * The `internal_canvas_get_line_width` function creates a gradient along the line connecting two given coordinates.
 */
declare function internal_canvas_create_linear_gradient(
  x0: number,
  y0: number,
  x1: number,
  y1: number,
): number;

/**
 * The `internal_canvas_get_line_width` function creates a radial gradient using the size and coordinates of two circles.
 */
declare function internal_canvas_create_radial_gradient(
  x0: number,
  y0: number,
  r0: number,
  x1: number,
  y1: number,
  r1: number,
): number;

/**
 * The `internal_canvas_get_line_width` function creates a gradient around a point with given coordinates.
 */
declare function internal_canvas_create_conic_gradient(
  startAngle: number,
  x: number,
  y: number,
): number;

/**
 * The `internal_canvas_gradient_add_color_stop` adds a new color stop to a given canvas gradient.
 */
declare function internal_canvas_gradient_add_color_stop(
  rid: number,
  offset: number,
  color: string,
): void;

/**
 * The `internal_text_encode` function encodes a string into a byte sequence.
 */
declare function internal_text_encode(input: string): string;

/**
 * The `internal_text_decode` function decodes a byte sequence into a string.
 */
declare function internal_text_decode(
  bytes: string,
  encoding: string,
  fatal: boolean,
): string;

/**
 * The `internal_text_encode_into` function encodes a string into a byte sequence and writes it to a destination buffer.
 */
declare function internal_text_encode_into(
  source: string,
  dest: string,
  destLen: number,
): string;

declare function internal_crypto_getRandomValues<
  T extends Uint8Array | Uint16Array | Uint32Array,
>(array: T): T;
declare function internal_crypto_randomUUID(): string;
declare function internal_subtle_digest(
  algorithm: AlgorithmIdentifier,
  data: Uint8Array | ArrayBuffer,
): ArrayBuffer;
declare function internal_subtle_generateKey(
  algorithm: AlgorithmIdentifier,
  extractable: boolean,
  keyUsages: KeyUsage[],
): CryptoKey | CryptoKeyPair;
declare function internal_subtle_importKey(
  format: KeyFormat,
  keyData: ArrayBuffer | Uint8Array | object,
  algorithm: AlgorithmIdentifier,
  extractable: boolean,
  keyUsages: KeyUsage[],
): CryptoKey;
declare function internal_subtle_exportKey(
  format: KeyFormat,
  key: CryptoKey,
): ArrayBuffer | object;
declare function internal_subtle_encrypt(
  algorithm: AlgorithmIdentifier,
  key: CryptoKey,
  data: Uint8Array | ArrayBuffer,
): ArrayBuffer;
declare function internal_subtle_decrypt(
  algorithm: AlgorithmIdentifier,
  key: CryptoKey,
  data: Uint8Array | ArrayBuffer,
): ArrayBuffer;
declare function internal_subtle_sign(
  algorithm: AlgorithmIdentifier,
  key: CryptoKey,
  data: Uint8Array | ArrayBuffer,
): ArrayBuffer;
declare function internal_subtle_verify(
  algorithm: AlgorithmIdentifier,
  key: CryptoKey,
  signature: Uint8Array | ArrayBuffer,
  data: Uint8Array | ArrayBuffer,
): boolean;

/**
 * The `internal_performance_now` function returns the current time in milliseconds since the page load.
 */
declare function internal_performance_now(): number;

/**
 * The `internal_performance_time_origin` function returns the time origin in milliseconds since the Unix epoch.
 * This is the time when the performance timing started for the current page.
 */
declare function internal_performance_time_origin(): number;

/**
 * The `internal_navigator_user_agent` function returns the user agent string for the browser.
 * This follows the HTML specification for navigator.userAgent.
 */
declare function internal_navigator_user_agent(): string;

/**
 * The `internal_add_signal_listener` function adds a signal listener for the specified signal.
 * The signal can be a string like "SIGINT", "SIGTERM", etc.
 */
declare function internal_add_signal_listener(
  signal: string,
  handler: () => void,
): string | void;

/**
 * The `internal_remove_signal_listener` function removes a signal listener for the specified signal.
 * The signal can be a string like "SIGINT", "SIGTERM", etc.
 */
declare function internal_remove_signal_listener(
  signal: string,
  handler: () => void,
): string | void;

// localStorage operations
/**
 * The `localStorage_length` function returns the number of items in localStorage.
 */
declare function localStorage_length(): number;

/**
 * The `localStorage_key` function returns the key at the specified index in localStorage.
 */
declare function localStorage_key(index: number): string | null;

/**
 * The `localStorage_getItem` function retrieves an item from localStorage by key.
 */
declare function localStorage_getItem(key: string): string | null;

/**
 * The `localStorage_setItem` function stores an item in localStorage with the specified key and value.
 */
declare function localStorage_setItem(key: string, value: string): void;

/**
 * The `localStorage_removeItem` function removes an item from localStorage by key.
 */
declare function localStorage_removeItem(key: string): void;

/**
 * The `localStorage_clear` function removes all items from localStorage.
 */
declare function localStorage_clear(): void;

/**
 * The `localStorage_keys` function returns an array of all keys in localStorage.
 */
declare function localStorage_keys(): string[];

// sessionStorage operations
/**
 * The `sessionStorage_length` function returns the number of items in sessionStorage.
 */
declare function sessionStorage_length(): number;

/**
 * The `sessionStorage_key` function returns the key at the specified index in sessionStorage.
 */
declare function sessionStorage_key(index: number): string | null;

/**
 * The `sessionStorage_getItem` function retrieves an item from sessionStorage by key.
 */
declare function sessionStorage_getItem(key: string): string | null;

/**
 * The `sessionStorage_setItem` function stores an item in sessionStorage with the specified key and value.
 */
declare function sessionStorage_setItem(key: string, value: string): void;

/**
 * The `sessionStorage_removeItem` function removes an item from sessionStorage by key.
 */
declare function sessionStorage_removeItem(key: string): void;

/**
 * The `sessionStorage_clear` function removes all items from sessionStorage.
 */
declare function sessionStorage_clear(): void;

/**
 * The `sessionStorage_keys` function returns an array of all keys in sessionStorage.
 */
declare function sessionStorage_keys(): string[];

/**
 * Creates a new storage instance.
 */
declare function storage_new(persistent: boolean): boolean;

/**
 * Deletes the storage instance.
 */
declare function storage_delete(storageType: boolean): boolean;

/**
 * Returns the number of items in the storage.
 */
declare function storage_length(storageType: boolean): number;

/**
 * Returns the key at the specified index in the storage.
 */
declare function storage_key(
  storageType: boolean,
  index: number,
): string | null;

/**
 * Retrieves an item from the storage.
 */
declare function storage_getItem(
  storageType: boolean,
  key: string,
): string | null;

/**
 * Stores an item in the storage.
 */
declare function storage_setItem(
  storageType: boolean,
  key: string,
  value: string,
): void;

/**
 * Removes an item from the storage.
 */
declare function storage_removeItem(storageType: boolean, key: string): void;

/**
 * Clears the storage.
 */
declare function storage_clear(storageType: boolean): void;

/**
 * Returns an array of all keys in the storage.
 */
declare function storage_iterate_keys(storageType: boolean): string[];

/**
 * The `sqlite_database_sync_constructor` function initializes a new SQLite database.
 */
declare function sqlite_database_sync_constructor(
  filename: string,
  options?: DatabaseSyncOptions,
): number;

/**
 * The `sqlite_database_sync_close` function closes a SQLite database.
 */
declare function sqlite_database_sync_close(dbId: number): void;

/**
 * The `sqlite_database_sync_enable_load_extension` function enables or disables extension loading.
 */
declare function sqlite_database_sync_enable_load_extension(
  dbId: number,
  enabled: boolean,
): void;

/**
 * The `sqlite_database_sync_exec` function executes SQL on a database.
 */
declare function sqlite_database_sync_exec(dbId: number, sql: string): void;

/**
 * The `sqlite_database_sync_function` function registers a custom function with SQLite.
 */
declare function sqlite_database_sync_function(
  dbId: number,
  name: string,
  // deno-lint-ignore no-explicit-any
  fn: any,
  options?: FunctionOptions,
): void;

/**
 * The `sqlite_database_sync_load_extension` function loads an extension into SQLite.
 */
declare function sqlite_database_sync_load_extension(
  dbId: number,
  path: string,
  entryPoint?: string,
): void;

/**
 * The `sqlite_database_sync_open` function opens a SQLite database.
 */
declare function sqlite_database_sync_open(
  dbId: number,
  filename: string,
  options?: DatabaseSyncOptions,
): void;

/**
 * The `sqlite_database_sync_prepare` function prepares a SQL statement.
 */
declare function sqlite_database_sync_prepare(
  dbId: number,
  sql: string,
): number;

/**
 * The `sqlite_statement_sync_all` function executes a statement and returns all rows.
 */
declare function sqlite_statement_sync_all(
  dbId: number,
  stmtId: number,
  ...params: SQLInputValue[]
): unknown[];

/**
 * The `sqlite_statement_sync_expanded_sql` function returns the expanded SQL of a prepared statement.
 */
declare function sqlite_statement_sync_expanded_sql(stmtId: number): string;

/**
 * The `sqlite_statement_sync_get` function executes a statement and returns the first row.
 */
declare function sqlite_statement_sync_get(
  dbId: number,
  stmtId: number,
  ...params: SQLInputValue[]
): unknown;

/**
 * The `sqlite_statement_sync_iterate` function executes a statement and returns an iterator of rows.
 */
declare function sqlite_statement_sync_iterate(
  dbId: number,
  stmtId: number,
  ...params: SQLInputValue[]
): unknown[];

/**
 * The `sqlite_statement_sync_run` function executes a statement that modifies the database.
 */
declare function sqlite_statement_sync_run(
  dbId: number,
  stmtId: number,
  ...params: SQLInputValue[]
): unknown;

/**
 * The `sqlite_statement_sync_set_allow_bare_named_parameters` function configures named parameter handling.
 */
declare function sqlite_statement_sync_set_allow_bare_named_parameters(
  stmtId: number,
  allowBare: boolean,
): void;

/**
 * The `sqlite_statement_sync_set_read_bigints` function configures bigint return handling.
 */
declare function sqlite_statement_sync_set_read_bigints(
  stmtId: number,
  readBigInts: boolean,
): void;

/**
 * The `sqlite_statement_sync_source_sql` function returns the original SQL of a prepared statement.
 */
declare function sqlite_statement_sync_source_sql(stmtId: number): string;

/**
 * The `sqlite_statement_sync_finalize` function finalizes a prepared statement.
 */
declare function sqlite_statement_sync_finalize(stmtId: number): void;
