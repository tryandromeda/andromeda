// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// deno-lint-ignore-file no-unused-vars

/**
 * A OffscreenCanvas implementation
 */
class OffscreenCanvas {
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

  /**
   * Renders the canvas to finalize GPU operations and optionally extract pixel data.
   * Returns true if rendering was successful, false otherwise.
   */
  render(): boolean {
    return internal_canvas_render(this.#rid);
  }

  /**
   * Saves the canvas as a PNG image file.
   * Returns true if save was successful, false otherwise.
   */
  saveAsPng(path: string): boolean {
    return this.render() ? internal_canvas_save_as_png(this.#rid, path) : false;
  }
}

/**
 * A 2D rendering context for Canvas
 */
class CanvasRenderingContext2D {
  #rid: number;
  
  constructor(rid: number) {
    this.#rid = rid;
  }  /**
   * Gets or sets the global alpha value (transparency) for drawing operations.
   * Value is in range [0.0, 1.0].
   */
  get globalAlpha(): number {
    // Convert integer representation back to float (divide by 1000)
    return internal_canvas_get_global_alpha(this.#rid) / 1000;
  }
  
  set globalAlpha(value: number) {
    internal_canvas_set_global_alpha(this.#rid, value);
  }

  /**
   * Gets or sets the current fill style for drawing operations.
   * Accepts CSS color strings like '#ff0000', 'rgb(255, 0, 0)', 'rgba(255, 0, 0, 0.5)', 'red', etc.
   */
  get fillStyle(): string {
    return internal_canvas_get_fill_style(this.#rid);
  }

  set fillStyle(value: string) {
    internal_canvas_set_fill_style(this.#rid, value);
  }
  /**
   * Gets or sets the current stroke style for drawing operations.
   * Accepts CSS color strings like '#ff0000', 'rgb(255, 0, 0)', 'rgba(255, 0, 0, 0.5)', 'red', etc.
   */
  get strokeStyle(): string {
    return internal_canvas_get_stroke_style(this.#rid);
  }

  set strokeStyle(value: string) {
    internal_canvas_set_stroke_style(this.#rid, value);
  }
  /**
   * Gets or sets the line width for drawing operations.
   */
  get lineWidth(): number {
    return internal_canvas_get_line_width(this.#rid);
  }

  set lineWidth(value: number) {
    internal_canvas_set_line_width(this.#rid, value);
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

  /**
   * Moves the path starting point to the specified coordinates.
   */
  moveTo(x: number, y: number): void {
    internal_canvas_move_to(this.#rid, x, y);
  }

  /**
   * Connects the last point in the current sub-path to the specified coordinates with a straight line.
   */
  lineTo(x: number, y: number): void {
    internal_canvas_line_to(this.#rid, x, y);
  }

  /**
   * Fills the current path with the current fill style.
   */
  fill(): void {
    internal_canvas_fill(this.#rid);
  }

  /**
   * Strokes the current path with the current stroke style.
   */
  stroke(): void {
    internal_canvas_stroke(this.#rid);
  }

  /**
   * Adds a rectangle to the current path.
   */
  rect(x: number, y: number, width: number, height: number): void {
    internal_canvas_rect(this.#rid, x, y, width, height);
  }

  /**
   * Creates a quadratic Bézier curve to the specified point.
   */
  quadraticCurveTo(cpx: number, cpy: number, x: number, y: number): void {
    internal_canvas_quadratic_curve_to(this.#rid, cpx, cpy, x, y);
  }

  /**
   * Creates an elliptical arc on the canvas.
   */
  ellipse(
    x: number,
    y: number,
    radiusX: number,
    radiusY: number,
    rotation: number,
    startAngle: number,
    endAngle: number,
    counterclockwise?: boolean
  ): void {
    internal_canvas_ellipse(
      this.#rid,
      x,
      y,
      radiusX,
      radiusY,
      rotation,
      startAngle,
      endAngle,
      counterclockwise
    );
  }
  /**
   * Adds a rounded rectangle to the current path.
   */
  roundRect(
    x: number,
    y: number,
    width: number,
    height: number,
    radius: number
  ): void {
    internal_canvas_round_rect(this.#rid, x, y, width, height, radius);
  }

  /**
   * Saves the current canvas state (styles, transformations, etc.) to a stack.
   */
  save(): void {
    internal_canvas_save(this.#rid);
  }

  /**
   * Restores the most recently saved canvas state from the stack.
   */
  restore(): void {
    internal_canvas_restore(this.#rid);
  }
}
