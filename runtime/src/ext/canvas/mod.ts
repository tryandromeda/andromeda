// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// deno-lint-ignore-file no-unused-vars
/**
 * A minimal Canvas implementation
 */
class Canvas {
  #rid: number;
  constructor(width: number, height: number) {
    this.#rid = internal_canvas_create(width, height);
  }

  /**
   * Get the width of the canvas.
   */
  getWidth(): number {
    return internal_canvas_get_width(this.#rid);
  }

  /**
   * Get the height of the canvas.
   */
  getHeight(): number {
    return internal_canvas_get_height(this.#rid);
  }

  /**
   * Get a drawing context.
   */
  getContext(type: string): CanvasRenderingContext2D | null {
    if (type === "2d") {
      return new CanvasRenderingContext2D(this.#rid);
    }
    return null;
  }
}

/**
 * Creates a new Canvas instance with the specified width and height.
 *
 * @example
 * ```typescript
 * const canvas = createCanvas(800, 600);
 * ```
 */
function createCanvas(width: number, height: number): Canvas {
  return new Canvas(width, height);
}

/**
 * A 2D rendering context for Canvas
 */
class CanvasRenderingContext2D {
  #rid: number;
  constructor(rid: number) {
    this.#rid = rid;
  }

  /**
   * Creates an arc on the canvas.
   */
  arc(
    x: number,
    y: number,
    radius: number,
    startAngle: number,
    endAngle: number,
  ): void {
    internal_canvas_arc(this.#rid, x, y, radius, startAngle, endAngle);
  }

  /**
   * Creates an arc to the canvas.
   */
  arcTo(
    x1: number,
    y1: number,
    x2: number,
    y2: number,
    radius: number,
  ): void {
    internal_canvas_arc_to(this.#rid, x1, y1, x2, y2, radius);
  }

  /**
   * Begin a new path on the canvas.
   */
  beginPath(): void {
    internal_canvas_begin_path(this.#rid);
  }

  /**
   * Bezier curve to the canvas.
   */
  bezierCurveTo(
    cp1x: number,
    cp1y: number,
    cp2x: number,
    cp2y: number,
    x: number,
    y: number,
  ): void {
    internal_canvas_bezier_curve_to(
      this.#rid,
      cp1x,
      cp1y,
      cp2x,
      cp2y,
      x,
      y,
    );
  }
  /**
   * Clears the specified rectangular area, making it fully transparent.
   */
  clearRect(x: number, y: number, width: number, height: number): void {
    internal_canvas_clear_rect(this.#rid, x, y, width, height);
  }

  /**
   * Closes the current path on the canvas.
   */
  closePath(): void {
    internal_canvas_close_path(this.#rid);
  }

  /**
   * Draws a filled rectangle whose starting corner is at (x, y).
   */
  fillRect(x: number, y: number, width: number, height: number): void {
    internal_canvas_fill_rect(this.#rid, x, y, width, height);
  }
}
