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
   * Draws a filled rectangle whose starting corner is at (x, y).
   */
  fillRect(x: number, y: number, width: number, height: number): void {
    internal_canvas_fill_rect(this.#rid, x, y, width, height);
  }

  /**
   * Clears the specified rectangular area, making it fully transparent.
   */
  clearRect(x: number, y: number, width: number, height: number): void {
    internal_canvas_clear_rect(this.#rid, x, y, width, height);
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
}
