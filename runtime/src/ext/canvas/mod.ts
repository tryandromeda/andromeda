// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

type CanvasFillRule = "nonzero" | "evenodd";

/**
 * A Path2D implementation for representing vector paths
 */
class Path2D {
  #rid: number;

  constructor(path?: Path2D | string) {
    if (path && typeof path === "object" && typeof path.getRid === "function") {
      // Create from another path
      this.#rid = __andromeda__.internal_path2d_create_from_path(path.getRid());
    } else if (typeof path === "string") {
      // Create from SVG path data
      this.#rid = __andromeda__.internal_path2d_create_from_svg(path);
    } else {
      // Create empty path
      this.#rid = __andromeda__.internal_path2d_create();
    }
  }

  /**
   * Gets the internal resource ID (for internal use by canvas operations)
   */
  getRid(): number {
    return this.#rid;
  }

  /**
   * Adds a path to the current path.
   */
  addPath(path: Path2D, _transform?: object): void {
    // TODO: Add transformation matrix support
    __andromeda__.internal_path2d_add_path(this.#rid, path.getRid());
  }

  /**
   * Adds a circular arc to the path.
   */
  arc(
    x: number,
    y: number,
    radius: number,
    startAngle: number,
    endAngle: number,
    counterclockwise?: boolean,
  ): void {
    __andromeda__.internal_path2d_arc(
      this.#rid,
      x,
      y,
      radius,
      startAngle,
      endAngle,
      counterclockwise || false,
    );
  }

  /**
   * Adds an elliptical arc to the path.
   */
  arcTo(x1: number, y1: number, x2: number, y2: number, radius: number): void {
    __andromeda__.internal_path2d_arc_to(this.#rid, x1, y1, x2, y2, radius);
  }

  /**
   * Adds a cubic Bézier curve to the path.
   */
  bezierCurveTo(
    cp1x: number,
    cp1y: number,
    cp2x: number,
    cp2y: number,
    x: number,
    y: number,
  ): void {
    __andromeda__.internal_path2d_bezier_curve_to(
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
   * Causes the point of the pen to move back to the start of the current sub-path.
   */
  closePath(): void {
    __andromeda__.internal_path2d_close_path(this.#rid);
  }

  /**
   * Adds an ellipse to the path.
   */
  ellipse(
    x: number,
    y: number,
    radiusX: number,
    radiusY: number,
    rotation: number,
    startAngle: number,
    endAngle: number,
    counterclockwise?: boolean,
  ): void {
    __andromeda__.internal_path2d_ellipse(
      this.#rid,
      x,
      y,
      radiusX,
      radiusY,
      rotation,
      startAngle,
      endAngle,
      counterclockwise || false,
    );
  }

  /**
   * Adds a straight line to the path.
   */
  lineTo(x: number, y: number): void {
    __andromeda__.internal_path2d_line_to(this.#rid, x, y);
  }

  /**
   * Moves the starting point of a new sub-path to the specified coordinates.
   */
  moveTo(x: number, y: number): void {
    __andromeda__.internal_path2d_move_to(this.#rid, x, y);
  }

  /**
   * Adds a quadratic Bézier curve to the path.
   */
  quadraticCurveTo(cpx: number, cpy: number, x: number, y: number): void {
    __andromeda__.internal_path2d_quadratic_curve_to(this.#rid, cpx, cpy, x, y);
  }

  /**
   * Adds a rectangle to the path.
   */
  rect(x: number, y: number, w: number, h: number): void {
    __andromeda__.internal_path2d_rect(this.#rid, x, y, w, h);
  }

  /**
   * Adds a rounded rectangle to the path.
   */
  roundRect(
    x: number,
    y: number,
    w: number,
    h: number,
    radii?: number | number[],
  ): void {
    const radiiArray = Array.isArray(radii) ?
      radii :
      (typeof radii === "number" ? [radii] : [0]);
    __andromeda__.internal_path2d_round_rect(this.#rid, x, y, w, h, radiiArray);
  }

  /**
   * Determines whether the specified point is contained in the current path.
   */
  isPointInPath(x: number, y: number, fillRule?: CanvasFillRule): boolean {
    const rule = fillRule || "nonzero";
    return __andromeda__.internal_canvas_is_point_in_path(
      this.#rid,
      x,
      y,
      rule,
    );
  }

  /**
   * Determines whether the specified point is inside the area contained by the stroking of the current path.
   */
  isPointInStroke(x: number, y: number, lineWidth?: number): boolean {
    const width = lineWidth || 1.0;
    return __andromeda__.internal_canvas_is_point_in_stroke(
      this.#rid,
      x,
      y,
      width,
    );
  }
}

/**
 * A OffscreenCanvas implementation
 */
class OffscreenCanvas {
  #rid: number;
  constructor(width: number, height: number) {
    this.#rid = __andromeda__.internal_canvas_create(width, height);
  }

  /**
   * Get the width of the canvas.
   */
  getWidth(): number {
    return __andromeda__.internal_canvas_get_width(this.#rid);
  }

  /**
   * Get the height of the canvas.
   */
  getHeight(): number {
    return __andromeda__.internal_canvas_get_height(this.#rid);
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
    return __andromeda__.internal_canvas_render(this.#rid);
  }

  /**
   * Saves the canvas as a PNG image file.
   * Returns true if save was successful, false otherwise.
   */
  saveAsPng(path: string): boolean {
    return this.render() ?
      __andromeda__.internal_canvas_save_as_png(this.#rid, path) :
      false;
  }
}

/**
 * A 2D rendering context for Canvas
 */
class CanvasRenderingContext2D {
  #rid: number;

  constructor(rid: number) {
    this.#rid = rid;
  } /**
   * Gets or sets the global alpha value (transparency) for drawing operations.
   * Value is in range [0.0, 1.0].
   */

  get globalAlpha(): number {
    return __andromeda__.internal_canvas_get_global_alpha(this.#rid);
  }

  set globalAlpha(value: number) {
    __andromeda__.internal_canvas_set_global_alpha(this.#rid, value);
  }

  /**
   * Gets or sets the current fill style for drawing operations.
   * Accepts CSS color strings like '#ff0000', 'rgb(255, 0, 0)', 'rgba(255, 0, 0, 0.5)', 'red', etc.
   */
  get fillStyle(): string | CanvasGradient {
    const fillStyle = __andromeda__.internal_canvas_get_fill_style(this.#rid);
    if (typeof fillStyle == "number") {
      return new CanvasGradient(fillStyle);
    } else {
      return fillStyle;
    }
  }

  set fillStyle(value: string | CanvasGradient) {
    if (typeof value == "string") {
      __andromeda__.internal_canvas_set_fill_style(this.#rid, value);
    } else {
      __andromeda__.internal_canvas_set_fill_style(this.#rid, value[_fillId]);
    }
  }
  /**
   * Gets or sets the current stroke style for drawing operations.
   * Accepts CSS color strings like '#ff0000', 'rgb(255, 0, 0)', 'rgba(255, 0, 0, 0.5)', 'red', etc.
   */
  get strokeStyle(): string {
    return __andromeda__.internal_canvas_get_stroke_style(this.#rid);
  }

  set strokeStyle(value: string) {
    __andromeda__.internal_canvas_set_stroke_style(this.#rid, value);
  }
  /**
   * Gets or sets the line width for drawing operations.
   */
  get lineWidth(): number {
    return __andromeda__.internal_canvas_get_line_width(this.#rid);
  }

  set lineWidth(value: number) {
    __andromeda__.internal_canvas_set_line_width(this.#rid, value);
  }

  /**
   * Sets the line dash pattern. Accepts an array of numbers or a JSON string.
   */
  setLineDash(segments: number[] | string, offset?: number): void {
    __andromeda__.internal_canvas_set_line_dash(
      this.#rid,
      segments,
      offset ?? 0,
    );
  }

  /**
   * Gets the line dash pattern as [segments, offset].
   * The runtime returns a JSON string; parse it here and return a tuple.
   */
  getLineDash(): [number[], number] {
    const json = __andromeda__.internal_canvas_get_line_dash(this.#rid);
    try {
      const info = JSON.parse(json);
      return [info.dash || [], info.offset || 0];
    } catch (_e) {
      if (typeof json === "string" && json.indexOf(",") !== -1) {
        const parts = json.split(",").map(s => parseFloat(s.trim())).filter(n =>
          !Number.isNaN(n)
        );
        return [parts, 0];
      }
      return [[], 0];
    }
  }

  get lineDashOffset(): number {
    const info = this.getLineDash();
    return info[1];
  }

  set lineDashOffset(value: number) {
    const info = this.getLineDash();
    this.setLineDash(info[0], value);
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
    __andromeda__.internal_canvas_arc(
      this.#rid,
      x,
      y,
      radius,
      startAngle,
      endAngle,
    );
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
    __andromeda__.internal_canvas_arc_to(this.#rid, x1, y1, x2, y2, radius);
  }

  /**
   * Begin a new path on the canvas.
   */
  beginPath(): void {
    __andromeda__.internal_canvas_begin_path(this.#rid);
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
    __andromeda__.internal_canvas_bezier_curve_to(
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
    __andromeda__.internal_canvas_clear_rect(this.#rid, x, y, width, height);
  }

  /**
   * Creates a gradient along the line connecting two given coordinates.
   */
  createLinearGradient(
    x0: number,
    y0: number,
    x1: number,
    y1: number,
  ): CanvasGradient {
    const rid = __andromeda__.internal_canvas_create_linear_gradient(
      x0,
      y0,
      x1,
      y1,
    );
    return new CanvasGradient(rid);
  }

  /**
   * Creates a radial gradient using the size and coordinates of two circles.
   */
  createRadialGradient(
    x0: number,
    y0: number,
    r0: number,
    x1: number,
    y1: number,
    r1: number,
  ): CanvasGradient {
    const rid = __andromeda__.internal_canvas_create_radial_gradient(
      x0,
      y0,
      r0,
      x1,
      y1,
      r1,
    );
    return new CanvasGradient(rid);
  }

  /**
   * Creates a gradient around a point with given coordinates.
   */
  createConicGradient(
    startAngle: number,
    x: number,
    y: number,
  ): CanvasGradient {
    const rid = __andromeda__.internal_canvas_create_conic_gradient(
      startAngle,
      x,
      y,
    );
    return new CanvasGradient(rid);
  }

  /**
   * Closes the current path on the canvas.
   */
  closePath(): void {
    __andromeda__.internal_canvas_close_path(this.#rid);
  }

  /**
   * Draws a filled rectangle whose starting corner is at (x, y).
   */
  fillRect(x: number, y: number, width: number, height: number): void {
    __andromeda__.internal_canvas_fill_rect(this.#rid, x, y, width, height);
  }

  /**
   * Moves the path starting point to the specified coordinates.
   */
  moveTo(x: number, y: number): void {
    __andromeda__.internal_canvas_move_to(this.#rid, x, y);
  }

  /**
   * Connects the last point in the current sub-path to the specified coordinates with a straight line.
   */
  lineTo(x: number, y: number): void {
    __andromeda__.internal_canvas_line_to(this.#rid, x, y);
  }

  /**
   * Fills the current path with the current fill style.
   */
  fill(): void {
    __andromeda__.internal_canvas_fill(this.#rid);
  }

  /**
   * Strokes the current path with the current stroke style.
   */
  stroke(): void {
    __andromeda__.internal_canvas_stroke(this.#rid);
  }

  /**
   * Adds a rectangle to the current path.
   */
  rect(x: number, y: number, width: number, height: number): void {
    __andromeda__.internal_canvas_rect(this.#rid, x, y, width, height);
  }

  /**
   * Creates a quadratic Bézier curve to the specified point.
   */
  quadraticCurveTo(cpx: number, cpy: number, x: number, y: number): void {
    __andromeda__.internal_canvas_quadratic_curve_to(this.#rid, cpx, cpy, x, y);
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
    counterclockwise?: boolean,
  ): void {
    __andromeda__.internal_canvas_ellipse(
      this.#rid,
      x,
      y,
      radiusX,
      radiusY,
      rotation,
      startAngle,
      endAngle,
      counterclockwise,
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
    radius: number,
  ): void {
    __andromeda__.internal_canvas_round_rect(
      this.#rid,
      x,
      y,
      width,
      height,
      radius,
    );
  }

  /**
   * Saves the current canvas state (styles, transformations, etc.) to a stack.
   */
  save(): void {
    __andromeda__.internal_canvas_save(this.#rid);
  }

  /**
   * Restores the most recently saved canvas state from the stack.
   */
  restore(): void {
    __andromeda__.internal_canvas_restore(this.#rid);
  }

  /**
   * Determines whether the specified point is contained in the given path.
   */
  isPointInPath(
    path: Path2D,
    x: number,
    y: number,
    fillRule?: CanvasFillRule,
  ): boolean {
    const rule = fillRule || "nonzero";
    return __andromeda__.internal_canvas_is_point_in_path(
      path.getRid(),
      x,
      y,
      rule,
    );
  }

  /**
   * Determines whether the specified point is inside the area contained by the stroking of a path.
   */
  isPointInStroke(path: Path2D, x: number, y: number): boolean {
    // Use current line width
    const lineWidth = this.lineWidth;
    return __andromeda__.internal_canvas_is_point_in_stroke(
      path.getRid(),
      x,
      y,
      lineWidth,
    );
  }

  /**
   * Turns the given path into the current clipping region.
   */
  clip(path: Path2D, fillRule?: CanvasFillRule): void {
    const _rule = fillRule || "nonzero";
    __andromeda__.internal_canvas_clip(this.#rid, path.getRid());
  }
}

const _fillId = Symbol("[[fillId]]");

class CanvasGradient {
  [_fillId]: number;

  constructor(rid: number) {
    this[_fillId] = rid;
  }
  /**
   * Adds a new color stop to a given canvas gradient.
   */
  addColorStop(offset: number, color: string) {
    __andromeda__.internal_canvas_gradient_add_color_stop(
      this[_fillId],
      offset,
      color,
    );
  }
}

// Export classes to global scope
Object.assign(globalThis, { Path2D, OffscreenCanvas });
