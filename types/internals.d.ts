// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
declare namespace __andromeda__ {
  /**
   * The `internal_read_text_file` function reads a file from the file system.
   */
  export function internal_read_text_file(path: string): string;

  /**
   * The `internal_read_text_file_async` function asynchronously reads a file from the file system.
   */
  export function internal_read_text_file_async(path: string): Promise<string>;

  /**
   * The `internal_write_text_file` function writes a text file to the file system.
   */
  export function internal_write_text_file(path: string, data: string): void;

  /**
   * The `internal_write_text_file_async` function asynchronously writes a text file to the file system.
   */
  export function internal_write_text_file_async(
    path: string,
    data: string,
  ): Promise<string>;

  /**
   * The `internal_create_file` function creates a file in the file system.
   */
  export function internal_create_file(path: string): void;

  /**
   * The `internal_create_file_async` function asynchronously creates a file in the file system.
   */
  export function internal_create_file_async(path: string): Promise<string>;

  /**
   * The `internal_copy_file` function copies a file in the file system.
   */
  export function internal_copy_file(
    source: string,
    destination: string,
  ): void;

  /**
   * The `internal_copy_file_async` function asynchronously copies a file in the file system.
   */
  export function internal_copy_file_async(
    source: string,
    destination: string,
  ): Promise<string>;

  /**
   * The `internal_mk_dir` function creates a directory in the file system.
   */
  export function internal_mk_dir(path: string): void;

  /**
   * The `internal_mk_dir_all` function creates a directory and all its parent directories in the file system.
   */
  export function internal_mk_dir_all(path: string): void;

  /**
   * The `internal_mk_dir_async` function asynchronously creates a directory in the file system.
   */
  export function internal_mk_dir_async(path: string): Promise<void>;

  /**
   * The `internal_mk_dir_all_async` function asynchronously creates a directory and all its parent directories in the file system.
   */
  export function internal_mk_dir_all_async(path: string): Promise<void>;

  /**
   * The `internal_read_file` function reads a file as binary data from the file system.
   */
  export function internal_read_file(path: string): Uint8Array;

  /**
   * The `internal_read_file_async` function asynchronously reads a file as binary data from the file system.
   */
  export function internal_read_file_async(path: string): Promise<Uint8Array>;

  /**
   * The `internal_write_file` function writes binary data to a file in the file system.
   */
  export function internal_write_file(path: string, data: Uint8Array): void;

  /**
   * The `internal_write_file_async` function asynchronously writes binary data to a file in the file system.
   */
  export function internal_write_file_async(
    path: string,
    data: Uint8Array,
  ): Promise<string>;

  /**
   * The `internal_remove` function removes a file from the file system.
   */
  export function internal_remove(path: string): void;

  /**
   * The `internal_remove_async` function asynchronously removes a file from the file system.
   */
  export function internal_remove_async(path: string): Promise<string>;

  /**
   * The `internal_remove_all` function recursively removes a file or directory from the file system.
   */
  export function internal_remove_all(path: string): void;

  /**
   * The `internal_remove_all_async` function asynchronously removes a file or directory recursively from the file system.
   */
  export function internal_remove_all_async(path: string): Promise<void>;

  /**
   * The `internal_rename` function renames/moves a file or directory in the file system.
   */
  export function internal_rename(oldPath: string, newPath: string): void;

  /**
   * The `internal_rename_async` function asynchronously renames/moves a file or directory in the file system.
   */
  export function internal_rename_async(
    oldPath: string,
    newPath: string,
  ): Promise<void>;

  /**
   * The `internal_exists` function checks if a file or directory exists in the file system.
   */
  export function internal_exists(path: string): boolean;

  /**
   * The `internal_exists_async` function asynchronously checks if a file or directory exists in the file system.
   */
  export function internal_exists_async(path: string): Promise<boolean>;

  /**
   * The `internal_truncate` function truncates a file to a specified length.
   */
  export function internal_truncate(path: string, length: number): void;

  /**
   * The `internal_chmod` function changes the permissions of a file or directory.
   */
  export function internal_chmod(path: string, mode: number): void;

  /**
   * The `internal_symlink` function creates a symbolic link.
   */
  export function internal_symlink(target: string, linkPath: string): void;

  /**
   * The `internal_read_link` function reads the target of a symbolic link.
   */
  export function internal_read_link(path: string): string;

  /**
   * The `internal_real_path` function resolves the absolute path of a file, resolving symbolic links.
   */
  export function internal_real_path(path: string): string;

  /**
   * The `internal_read_dir` function reads the contents of a directory.
   */
  export function internal_read_dir(
    path: string,
  ): Array<
    { name: string; isFile: boolean; isDirectory: boolean; isSymlink: boolean; }
  >;

  /**
   * The `internal_stat` function gets information about a file or directory.
   */
  export function internal_stat(path: string): {
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
  export function internal_lstat(path: string): {
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
  export function internal_exit(code: number): void;

  /**
   * The `internal_read_line` function reads a line from standard input.
   */
  export function internal_read_line(): string;

  /**
   * The `internal_write` function writes a string to standard output.
   */
  export function internal_write(message: string): void;

  /**
   * The `internal_write_line` function writes a string to standard output followed by a newline.
   */
  export function internal_write_line(message: string): void;

  /**
   * The `internal_open_file` function opens a file and returns a file descriptor.
   */
  export function internal_open_file(path: string, mode: string): number;

  /**
   * The `internal_sleep` function returns a Promise to be resolved after the specified time un milliseconds.
   */
  export function internal_sleep(duration: number): Promise<void>;

  /**
   *  The `internal_print` function to log messages to the console.
   */
  export function internal_print(message: string): void;

  /**
   * The `internal_print_err` function to log error messages to the console.
   */
  export function internal_print_err(message: string): void;

  /**
   * The `clear_console` function clears the console.
   */
  export function clear_console(): void;

  /**
   * The `get_stack_trace` function returns the current stack trace.
   */
  export function get_stack_trace(): string;

  /**
   * The `internal_get_cli_args` function to get the command line arguments.
   */
  export function internal_get_cli_args(): string[];

  /**
   * The `internal_get_env` function to get the environment variable.
   */
  export function internal_get_env(key: string): string;

  /**
   * The `internal_set_env` function to set the environment variable.
   */
  export function internal_set_env(key: string, value: string): void;

  /**
   * The `internal_delete_env` function to delete the environment variable.
   */
  export function internal_delete_env(key: string): void;

  /**
   * The `internal_get_env_keys` function to get the environment variable keys.
   */
  export function internal_get_env_keys(): string[];

  /**
   * The `internal_url_parse` function to parse a URL string.
   */
  export function internal_url_parse(url: string, base: string): string;

  /**
   * The `internal_url_parse_no_base` function to parse a URL string without a base URL.
   */
  export function internal_url_parse_no_base(url: string): string;
  export function internal_url_get_origin(url: string): string;
  export function internal_url_set_hostname(url: string, v: string): string;
  export function internal_url_set_port(url: string, v: string): string;
  export function internal_url_set_pathname(url: string, v: string): string;
  export function internal_url_set_search(url: string, v: string): string;
  export function internal_url_set_hash(url: string, v: string): string;
  export function internal_url_set_username(url: string, v: string): string;
  export function internal_url_set_password(url: string, v: string): string;

  /**
   * The `internal_url_get_protocol` function returns the protocol (scheme) of a URL string.
   */
  export function internal_url_get_protocol(url: string): string;

  /**
   * The `internal_url_get_username` function returns the username of a URL string.
   */
  export function internal_url_get_username(url: string): string;

  /**
   * The `internal_url_get_password` function returns the password of a URL string.
   */
  export function internal_url_get_password(url: string): string;

  /**
   * The `internal_url_get_host` function returns the host of a URL string.
   */
  export function internal_url_get_host(url: string): string;

  /**
   * The `internal_url_get_hostname` function returns the hostname of a URL string.
   */
  export function internal_url_get_hostname(url: string): string;

  /**
   * The `internal_url_get_port` function returns the port of a URL string.
   */
  export function internal_url_get_port(url: string): string;

  /**
   * The `internal_url_get_pathname` function returns the pathname of a URL string.
   */
  export function internal_url_get_pathname(url: string): string;

  /**
   * The `internal_url_get_search` function returns the search of a URL string.
   */
  export function internal_url_get_search(url: string): string;

  /**
   * The `internal_url_get_hash` function returns the hash of a URL string.
   */
  export function internal_url_get_hash(url: string): string;

  /**
   * The `internal_btoa` function encodes a string in base64.
   */
  export function internal_btoa(input: string): string;

  /**
   * The `internal_atob` function decodes a string in base64.
   */
  export function internal_atob(input: string): string;

  /**
   * The `internal_canvas_create` function creates a canvas with the specified width and height.
   */
  export function internal_canvas_create(
    width: number,
    height: number,
  ): number;

  /**
   * The `internal_canvas_get_width` function gets the width of the canvas.
   */
  export function internal_canvas_get_width(rid: number): number;

  /**
   * The `internal_canvas_get_height` function gets the height of the canvas.
   */
  export function internal_canvas_get_height(rid: number): number;
  /**
   * The `internal_canvas_arc` function creates an arc on the canvas.
   */
  export function internal_canvas_arc(
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
  export function internal_canvas_arc_to(
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
  export function internal_canvas_begin_path(rid: number): void;

  /**
   * The `internal_canvas_bezier_curve_to` function creates a bezier curve on the canvas.
   */
  export function internal_canvas_bezier_curve_to(
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
  export function internal_canvas_clear_rect(
    rid: number,
    x: number,
    y: number,
    width: number,
    height: number,
  ): void;

  /**
   * The `internal_canvas_close_path` function closes the current path on the canvas.
   */
  export function internal_canvas_close_path(rid: number): void;
  /**
   * Draws a filled rectangle on the specified canvas.
   */
  export function internal_canvas_fill_rect(
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
  export function internal_canvas_render(rid: number): boolean;

  /**
   * The `internal_canvas_save_as_png` function saves the canvas as a PNG file.
   * Returns true if save was successful, false otherwise.
   */
  export function internal_canvas_save_as_png(
    rid: number,
    path: string,
  ): boolean;

  /**
   * The `internal_canvas_get_fill_style` function gets the current fill style of the canvas context.
   * Returns the fill style as a CSS color string.
   */
  export function internal_canvas_get_fill_style(rid: number): string | number;

  /**
   * The `internal_canvas_set_fill_style` function sets the fill style of the canvas context.
   * Accepts CSS color strings like '#ff0000', 'rgb(255, 0, 0)', 'rgba(255, 0, 0, 0.5)', 'red', etc.
   */
  export function internal_canvas_set_fill_style(
    rid: number,
    style: string | number,
  ): void;

  /**
   * The `internal_canvas_move_to` function moves the path starting point to the specified coordinates.
   */
  export function internal_canvas_move_to(
    rid: number,
    x: number,
    y: number,
  ): void;

  /**
   * The `internal_canvas_line_to` function connects the last point in the current sub-path to the specified coordinates with a straight line.
   */
  export function internal_canvas_line_to(
    rid: number,
    x: number,
    y: number,
  ): void;

  /**
   * The `internal_canvas_fill` function fills the current path with the current fill style.
   */
  export function internal_canvas_fill(rid: number): void;

  /**
   * The `internal_canvas_stroke` function strokes the current path with the current stroke style.
   */
  export function internal_canvas_stroke(rid: number): void;

  /**
   * The `internal_canvas_rect` function adds a rectangle to the current path.
   */
  export function internal_canvas_rect(
    rid: number,
    x: number,
    y: number,
    width: number,
    height: number,
  ): void;

  /**
   * The `internal_canvas_set_line_width` function sets the line width for stroking on the canvas.
   */
  export function internal_canvas_set_line_width(
    rid: number,
    lineWidth: number,
  ): void;

  /**
   * The `internal_canvas_set_stroke_style` function sets the stroke style of the canvas context.
   * Accepts CSS color strings like '#ff0000', 'rgb(255, 0, 0)', 'rgba(255, 0, 0, 0.5)', 'red', etc.
   */
  export function internal_canvas_set_stroke_style(
    rid: number,
    style: string,
  ): void;

  /**
   * The `internal_canvas_set_line_dash` function sets the line dash pattern and optional offset
   * for the specified canvas resource. The `pattern` argument may be an array-like value or
   * a string; the runtime will accept either and coerce/parse it as needed.
   */
  export function internal_canvas_set_line_dash(
    rid: number,
    pattern: unknown,
    offset?: number,
  ): void;

  /**
   * The `internal_canvas_get_line_dash` function returns the current line dash pattern and offset
   * for the specified canvas resource. For compatibility the runtime returns a JSON string describing
   * the dash pattern and offset (e.g. '{"dash":[5,3],"offset":2}').
   */
  export function internal_canvas_get_line_dash(rid: number): string;

  /**
   * The `internal_canvas_get_global_alpha` function gets the global alpha value of the canvas context.
   * Returns the global alpha as a number (floating point between 0.0 and 1.0).
   */
  export function internal_canvas_get_global_alpha(rid: number): number;

  /**
   * The `internal_canvas_set_global_alpha` function sets the global alpha value of the canvas context.
   * Accepts a global alpha value as an integer (scaled by 1000).
   */
  export function internal_canvas_set_global_alpha(
    rid: number,
    alpha: number,
  ): void;

  /**
   * The `internal_image_bitmap_create` function creates an ImageBitmap resource and returns its Rid.
   */
  export function internal_image_bitmap_create(path: string): number;
  /**
   * The `internal_image_bitmap_get_width` function returns the width of the ImageBitmap resource.
   */
  export function internal_image_bitmap_get_width(rid: number): number;
  /**
   * The `internal_image_bitmap_get_height` function returns the height of the ImageBitmap resource.
   */
  export function internal_image_bitmap_get_height(rid: number): number;
  /**
   * The `internal_canvas_quadratic_curve_to` function creates a quadratic curve on the canvas.
   */
  export function internal_canvas_quadratic_curve_to(
    rid: number,
    cpx: number,
    cpy: number,
    x: number,
    y: number,
  ): void;

  /**
   * The `internal_canvas_ellipse` function creates an ellipse on the canvas.
   */
  export function internal_canvas_ellipse(
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
  export function internal_canvas_round_rect(
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
  export function internal_canvas_save(rid: number): void;

  /**
   * The `internal_canvas_restore` function restores the most recently saved canvas state.
   */
  export function internal_canvas_restore(rid: number): void;

  /**
   * The `internal_canvas_get_stroke_style` function gets the current stroke style of the canvas context.
   * Returns the stroke style as a CSS color string.
   */
  export function internal_canvas_get_stroke_style(rid: number): string;

  /**
   * The `internal_canvas_get_line_width` function gets the current line width of the canvas context.
   * Returns the line width as a number.
   */
  export function internal_canvas_get_line_width(rid: number): number;

  /**
   * The `internal_canvas_get_line_width` function creates a gradient along the line connecting two given coordinates.
   */
  export function internal_canvas_create_linear_gradient(
    x0: number,
    y0: number,
    x1: number,
    y1: number,
  ): number;

  /**
   * The `internal_canvas_get_line_width` function creates a radial gradient using the size and coordinates of two circles.
   */
  export function internal_canvas_create_radial_gradient(
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
  export function internal_canvas_create_conic_gradient(
    startAngle: number,
    x: number,
    y: number,
  ): number;

  /**
   * The `internal_canvas_gradient_add_color_stop` adds a new color stop to a given canvas gradient.
   */
  export function internal_canvas_gradient_add_color_stop(
    rid: number,
    offset: number,
    color: string,
  ): void;

  /**
   * The `internal_text_encode` function encodes a string into a byte sequence.
   */
  export function internal_text_encode(input: string): string;

  /**
   * The `internal_text_decode` function decodes a byte sequence into a string.
   */
  export function internal_text_decode(
    bytes: string,
    encoding: string,
    fatal: boolean,
  ): string;

  /**
   * The `internal_text_encode_into` function encodes a string into a byte sequence and writes it to a destination buffer.
   */
  export function internal_text_encode_into(
    source: string,
    dest: string,
    destLen: number,
  ): string;

  export function internal_crypto_getRandomValues<
    T extends Uint8Array | Uint16Array | Uint32Array,
  >(array: T): T;
  export function internal_crypto_randomUUID(): string;
  export function internal_subtle_digest(
    algorithm: AlgorithmIdentifier,
    data: Uint8Array | ArrayBuffer,
  ): ArrayBuffer;
  export function internal_subtle_generateKey(
    algorithm: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): CryptoKey | CryptoKeyPair;
  export function internal_subtle_importKey(
    format: KeyFormat,
    keyData: ArrayBuffer | Uint8Array | object,
    algorithm: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): CryptoKey;
  export function internal_subtle_exportKey(
    format: KeyFormat,
    key: CryptoKey,
  ): ArrayBuffer | object;
  export function internal_subtle_encrypt(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    data: Uint8Array | ArrayBuffer,
  ): ArrayBuffer;
  export function internal_subtle_decrypt(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    data: Uint8Array | ArrayBuffer,
  ): ArrayBuffer;
  export function internal_subtle_sign(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    data: Uint8Array | ArrayBuffer,
  ): ArrayBuffer;
  export function internal_subtle_verify(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    signature: Uint8Array | ArrayBuffer,
    data: Uint8Array | ArrayBuffer,
  ): boolean;

  /**
   * The `internal_performance_now` function returns the current time in milliseconds since the page load.
   */
  export function internal_performance_now(): number;

  /**
   * The `internal_performance_time_origin` function returns the time origin in milliseconds since the Unix epoch.
   * This is the time when the performance timing started for the current page.
   */
  export function internal_performance_time_origin(): number;

  /**
   * The `internal_navigator_user_agent` function returns the user agent string for the browser.
   * This follows the HTML specification for navigator.userAgent.
   */
  export function internal_navigator_user_agent(): string;

  /**
   * The `internal_add_signal_listener` function adds a signal listener for the specified signal.
   * The signal can be a string like "SIGINT", "SIGTERM", etc.
   */
  export function internal_add_signal_listener(
    signal: string,
    handler: () => void,
  ): string | void;

  /**
   * The `internal_remove_signal_listener` function removes a signal listener for the specified signal.
   * The signal can be a string like "SIGINT", "SIGTERM", etc.
   */
  export function internal_remove_signal_listener(
    signal: string,
    handler: () => void,
  ): string | void;

  // localStorage operations
  /**
   * The `localStorage_length` function returns the number of items in localStorage.
   */
  export function localStorage_length(): number;

  /**
   * The `localStorage_key` function returns the key at the specified index in localStorage.
   */
  export function localStorage_key(index: number): string | null;

  /**
   * The `localStorage_getItem` function retrieves an item from localStorage by key.
   */
  export function localStorage_getItem(key: string): string | null;

  /**
   * The `localStorage_setItem` function stores an item in localStorage with the specified key and value.
   */
  export function localStorage_setItem(key: string, value: string): void;

  /**
   * The `localStorage_removeItem` function removes an item from localStorage by key.
   */
  export function localStorage_removeItem(key: string): void;

  /**
   * The `localStorage_clear` function removes all items from localStorage.
   */
  export function localStorage_clear(): void;

  /**
   * The `localStorage_keys` function returns an array of all keys in localStorage.
   */
  export function localStorage_keys(): string[];

  // sessionStorage operations
  /**
   * The `sessionStorage_length` function returns the number of items in sessionStorage.
   */
  export function sessionStorage_length(): number;

  /**
   * The `sessionStorage_key` function returns the key at the specified index in sessionStorage.
   */
  export function sessionStorage_key(index: number): string | null;

  /**
   * The `sessionStorage_getItem` function retrieves an item from sessionStorage by key.
   */
  export function sessionStorage_getItem(key: string): string | null;

  /**
   * The `sessionStorage_setItem` function stores an item in sessionStorage with the specified key and value.
   */
  export function sessionStorage_setItem(key: string, value: string): void;

  /**
   * The `sessionStorage_removeItem` function removes an item from sessionStorage by key.
   */
  export function sessionStorage_removeItem(key: string): void;

  /**
   * The `sessionStorage_clear` function removes all items from sessionStorage.
   */
  export function sessionStorage_clear(): void;

  /**
   * The `sessionStorage_keys` function returns an array of all keys in sessionStorage.
   */
  export function sessionStorage_keys(): string[];

  /**
   * Creates a new storage instance.
   */
  export function storage_new(persistent: boolean): boolean;

  /**
   * Deletes the storage instance.
   */
  export function storage_delete(storageType: boolean): boolean;

  /**
   * Returns the number of items in the storage.
   */
  export function storage_length(storageType: boolean): number;

  /**
   * Returns the key at the specified index in the storage.
   */
  export function storage_key(
    storageType: boolean,
    index: number,
  ): string | null;

  /**
   * Retrieves an item from the storage.
   */
  export function storage_getItem(
    storageType: boolean,
    key: string,
  ): string | null;

  /**
   * Stores an item in the storage.
   */
  export function storage_setItem(
    storageType: boolean,
    key: string,
    value: string,
  ): void;

  /**
   * Removes an item from the storage.
   */
  export function storage_removeItem(storageType: boolean, key: string): void;

  /**
   * Clears the storage.
   */
  export function storage_clear(storageType: boolean): void;

  /**
   * Returns an array of all keys in the storage.
   */
  export function storage_iterate_keys(storageType: boolean): string[];

  /**
   * The `sqlite_database_sync_constructor` function initializes a new SQLite database.
   */
  export function sqlite_database_sync_constructor(
    filename: string,
    options?: DatabaseSyncOptions,
  ): number;

  /**
   * The `sqlite_database_sync_close` function closes a SQLite database.
   */
  export function sqlite_database_sync_close(dbId: number): void;

  /**
   * The `sqlite_database_sync_enable_load_extension` function enables or disables extension loading.
   */
  export function sqlite_database_sync_enable_load_extension(
    dbId: number,
    enabled: boolean,
  ): void;

  /**
   * The `sqlite_database_sync_exec` function executes SQL on a database.
   */
  export function sqlite_database_sync_exec(dbId: number, sql: string): void;

  /**
   * The `sqlite_database_sync_function` function registers a custom function with SQLite.
   */
  export function sqlite_database_sync_function(
    dbId: number,
    name: string,
    // deno-lint-ignore no-explicit-any
    fn: any,
    options?: FunctionOptions,
  ): void;

  /**
   * The `sqlite_database_sync_load_extension` function loads an extension into SQLite.
   */
  export function sqlite_database_sync_load_extension(
    dbId: number,
    path: string,
    entryPoint?: string,
  ): void;

  /**
   * The `sqlite_database_sync_open` function opens a SQLite database.
   */
  export function sqlite_database_sync_open(
    dbId: number,
    filename: string,
    options?: DatabaseSyncOptions,
  ): void;

  /**
   * The `sqlite_database_sync_prepare` function prepares a SQL statement.
   */
  export function sqlite_database_sync_prepare(
    dbId: number,
    sql: string,
  ): number;

  /**
   * The `sqlite_statement_sync_all` function executes a statement and returns all rows.
   */
  export function sqlite_statement_sync_all(
    dbId: number,
    stmtId: number,
    ...params: SQLInputValue[]
  ): unknown[];

  /**
   * The `sqlite_statement_sync_expanded_sql` function returns the expanded SQL of a prepared statement.
   */
  export function sqlite_statement_sync_expanded_sql(stmtId: number): string;

  /**
   * The `sqlite_statement_sync_get` function executes a statement and returns the first row.
   */
  export function sqlite_statement_sync_get(
    dbId: number,
    stmtId: number,
    ...params: SQLInputValue[]
  ): unknown;

  /**
   * The `sqlite_statement_sync_iterate` function executes a statement and returns an iterator of rows.
   */
  export function sqlite_statement_sync_iterate(
    dbId: number,
    stmtId: number,
    ...params: SQLInputValue[]
  ): unknown[];

  /**
   * The `sqlite_statement_sync_run` function executes a statement that modifies the database.
   */
  export function sqlite_statement_sync_run(
    dbId: number,
    stmtId: number,
    ...params: SQLInputValue[]
  ): unknown;

  /**
   * The `sqlite_statement_sync_set_allow_bare_named_parameters` function configures named parameter handling.
   */
  export function sqlite_statement_sync_set_allow_bare_named_parameters(
    stmtId: number,
    allowBare: boolean,
  ): void;

  /**
   * The `sqlite_statement_sync_set_read_bigints` function configures bigint return handling.
   */
  export function sqlite_statement_sync_set_read_bigints(
    stmtId: number,
    readBigInts: boolean,
  ): void;

  /**
   * The `sqlite_statement_sync_source_sql` function returns the original SQL of a prepared statement.
   */
  export function sqlite_statement_sync_source_sql(stmtId: number): string;

  /**
   * The `sqlite_statement_sync_finalize` function finalizes a prepared statement.
   */
  export function sqlite_statement_sync_finalize(stmtId: number): void;

  export const internal_blob_create: (
    parts: string,
    options: string,
  ) => string;
  export const internal_blob_slice: (
    blobId: string,
    start: number,
    end: number,
    contentType: string,
  ) => string;
  export const internal_blob_get_data: (blobId: string) => string;
  export const internal_blob_get_size: (blobId: string) => number;
  export const internal_blob_get_type: (blobId: string) => string;
  export const internal_blob_stream: (blobId: string) => string;
  export const internal_blob_array_buffer: (blobId: string) => string;
  export const internal_blob_text: (blobId: string) => string;

  export const internal_formdata_create: () => string;
  export const internal_formdata_append: (
    formDataId: string,
    name: string,
    value: string,
  ) => string;
  export const internal_formdata_delete: (
    formDataId: string,
    name: string,
  ) => string;
  export const internal_formdata_get: (
    formDataId: string,
    name: string,
  ) => string;
  export const internal_formdata_get_all: (
    formDataId: string,
    name: string,
  ) => string;
  export const internal_formdata_has: (
    formDataId: string,
    name: string,
  ) => number;
  export const internal_formdata_set: (
    formDataId: string,
    name: string,
    value: string,
  ) => string;
  export const internal_formdata_keys: (formDataId: string) => string;
  export const internal_formdata_values: (formDataId: string) => string;
  export const internal_formdata_entries: (formDataId: string) => string;
  export const internal_file_create: (
    parts: string,
    name: string,
    options: string,
    lastModified: number,
  ) => string;

  /**
   * The `internal_readable_stream_create` function creates a new ReadableStream and returns its ID.
   */
  export function internal_readable_stream_create(): string;

  /**
   * The `internal_readable_stream_read` function reads data from a ReadableStream.
   */
  export function internal_readable_stream_read(streamId: string): string;

  /**
   * The `internal_readable_stream_cancel` function cancels a ReadableStream.
   */
  export function internal_readable_stream_cancel(streamId: string): string;

  /**
   * The `internal_readable_stream_close` function closes a ReadableStream.
   */
  export function internal_readable_stream_close(streamId: string): string;

  /**
   * The `internal_readable_stream_enqueue` function enqueues data to a ReadableStream.
   */
  export function internal_readable_stream_enqueue(
    streamId: string,
    chunk: string,
  ): string;

  /**
   * The `internal_writable_stream_create` function creates a new WritableStream and returns its ID.
   */
  export function internal_writable_stream_create(): string;

  /**
   * The `internal_writable_stream_write` function writes data to a WritableStream.
   */
  export function internal_writable_stream_write(
    streamId: string,
    chunk: string,
  ): string;

  /**
   * The `internal_writable_stream_close` function closes a WritableStream.
   */
  export function internal_writable_stream_close(streamId: string): string;

  /**
   * The `internal_writable_stream_abort` function aborts a WritableStream.
   */
  export function internal_writable_stream_abort(streamId: string): string;

  /**
   * The `internal_stream_get_state` function gets the state of a stream.
   */
  export function internal_stream_get_state(streamId: string): string;

  /**
   * The `internal_readable_stream_error` function puts a ReadableStream into an error state.
   */
  export function internal_readable_stream_error(streamId: string, error: string): string;

  /**
   * The `internal_readable_stream_lock` function locks a ReadableStream for exclusive reading.
   */
  export function internal_readable_stream_lock(streamId: string): string;

  /**
   * The `internal_readable_stream_unlock` function unlocks a ReadableStream.
   */
  export function internal_readable_stream_unlock(streamId: string): string;

  /**
   * The `internal_readable_stream_tee` function creates two independent branches of a ReadableStream.
   */
  export function internal_readable_stream_tee(streamId: string): string;

  /**
   * The `internal_writable_stream_error` function puts a WritableStream into an error state.
   */
  export function internal_writable_stream_error(streamId: string, error: string): string;

  /**
   * The `internal_writable_stream_lock` function locks a WritableStream for exclusive writing.
   */
  export function internal_writable_stream_lock(streamId: string): string;

  /**
   * The `internal_writable_stream_unlock` function unlocks a WritableStream.
   */
  export function internal_writable_stream_unlock(streamId: string): string;

  /**
   * The `internal_stream_set_desired_size` function sets the desired size for a stream.
   */
  export function internal_stream_set_desired_size(streamId: string, desiredSize: number): string;

  /**
   * The `internal_stream_get_desired_size` function gets the desired size of a stream.
   */
  export function internal_stream_get_desired_size(streamId: string): string;

  /**
   * The `internal_stream_get_chunk_count` function gets the number of chunks queued in a stream.
   */
  export function internal_stream_get_chunk_count(streamId: string): string;

  /**
   * The `time_start` function starts a timer with the given label.
   */
  export function time_start(label?: string): number;

  /**
   * The `time_log` function logs the current time for a timer.
   */
  export function time_log(label: string, data?: string): string;

  /**
   * The `time_end` function ends a timer and returns the elapsed time.
   */
  export function time_end(label?: string): string;

  /**
   * The `count` function increments a counter for the given label.
   */
  export function count(label?: string): string;

  /**
   * The `count_reset` function resets a counter for the given label.
   */
  export function count_reset(label?: string): string;

  /**
   * The `group_start` function starts a console group.
   */
  export function group_start(label?: string): string;

  /**
   * The `group_end` function ends a console group.
   */
  export function group_end(): void;

  /**
   * The `internal_css_to_ansi` function converts CSS styling to ANSI escape codes.
   */
  export function internal_css_to_ansi(cssText: string): string;

  /**
   * The `get_group_indent` function returns the current console group indentation level.
   */
  export function get_group_indent(): number;

  export function cache_match(
    cacheName: string,
    request: RequestInfo,
    options?: CacheQueryOptions,
  ): Response | undefined;
  export function cache_matchAll(
    cacheName: string,
    request?: RequestInfo,
    options?: CacheQueryOptions,
  ): Response[];
  export function cache_add(cacheName: string, request: RequestInfo): void;
  export function cache_addAll(
    cacheName: string,
    requests: RequestInfo[],
  ): void;
  export function cache_put(
    cacheName: string,
    request: RequestInfo,
    response: Response,
  ): void;
  export function cache_delete(
    cacheName: string,
    request: RequestInfo,
    options?: CacheQueryOptions,
  ): boolean;
  export function cache_keys(
    cacheName: string,
    request?: RequestInfo,
    options?: CacheQueryOptions,
  ): Request[];
  export function cacheStorage_open(cacheName: string): void;
  export function cacheStorage_has(cacheName: string): boolean;
  export function cacheStorage_delete(cacheName: string): boolean;
  export function cacheStorage_keys(): string[];
  export function cacheStorage_match(
    request: RequestInfo,
    options?: CacheQueryOptions,
  ): Response | undefined;

  /**
   * The `cron` function creates a cron job with the specified name, schedule, and handler.
   */
  export function cron(
    name: string,
    schedule: string,
    handler: () => void | Promise<void>,
  ): Promise<void>;

  type Rid = string;

  export function internal_tls_connect(
    host: string,
    port: number,
  ): Promise<Rid>;
  export function internal_tls_close(rid: Rid): Promise<string>;
  export function internal_tls_read(rid: Rid, len: number): Promise<string>;
  export function internal_tls_write(rid: Rid, data: string): Promise<string>;
  export function internal_tls_get_peer_certificate(rid: Rid): Promise<string>;
  export function ffi_dlopen(filename: string, symbols: unknown): number;
  export function ffi_dlopen_get_symbol(
    libId: number,
    name: string,
    definition: unknown,
  ): unknown;
  export function ffi_call_symbol(
    libId: number,
    name: string,
    args: unknown[],
  ): unknown;
  export function ffi_dlclose(libId: number): void;
  export function ffi_create_callback(
    definition: unknown,
    callback: unknown,
  ): number;
  export function ffi_get_callback_pointer(
    callbackId: number,
  ): number | bigint;
  export function ffi_callback_close(callbackId: number): void;
  export function ffi_pointer_create(value: number): unknown;
  export function ffi_pointer_equals(a: unknown, b: unknown): boolean;
  export function ffi_pointer_offset(value: unknown, offset: number): unknown;
  export function ffi_pointer_value(value: unknown): number | bigint;
  export function ffi_pointer_of(value: unknown): unknown;
  export function ffi_read_memory(
    ptr: unknown,
    offset: number,
    size: number,
  ): ArrayBuffer;
  export function ffi_write_memory(
    ptr: unknown,
    offset: unknown,
    data: unknown,
  ): void;
}
