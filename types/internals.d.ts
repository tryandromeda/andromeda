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
declare function internal_canvas_save_as_png(rid: number, path: string): boolean;

/**
 * The `internal_canvas_get_fill_style` function gets the current fill style of the canvas context.
 * Returns the fill style as a CSS color string.
 */
declare function internal_canvas_get_fill_style(rid: number): string;

/**
 * The `internal_canvas_set_fill_style` function sets the fill style of the canvas context.
 * Accepts CSS color strings like '#ff0000', 'rgb(255, 0, 0)', 'rgba(255, 0, 0, 0.5)', 'red', etc.
 */
declare function internal_canvas_set_fill_style(rid: number, style: string): void;

/**
 * The `internal_canvas_move_to` function moves the path starting point to the specified coordinates.
 */
declare function internal_canvas_move_to(rid: number, x: number, y: number): void;

/**
 * The `internal_canvas_line_to` function connects the last point in the current sub-path to the specified coordinates with a straight line.
 */
declare function internal_canvas_line_to(rid: number, x: number, y: number): void;

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
declare function internal_canvas_rect(rid: number, x: number, y: number, width: number, height: number): void;

/**
 * The `internal_canvas_set_line_width` function sets the line width for stroking on the canvas.
 */
declare function internal_canvas_set_line_width(rid: number, lineWidth: number): void;

/**
 * The `internal_canvas_set_stroke_style` function sets the stroke style of the canvas context.
 * Accepts CSS color strings like '#ff0000', 'rgb(255, 0, 0)', 'rgba(255, 0, 0, 0.5)', 'red', etc.
 */
declare function internal_canvas_set_stroke_style(rid: number, style: string): void;

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
