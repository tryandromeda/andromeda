// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

type CanvasFillRule = "nonzero" | "evenodd";

interface DOMMatrix2DInit {
  a?: number;
  b?: number;
  c?: number;
  d?: number;
  e?: number;
  f?: number;
  m11?: number;
  m12?: number;
  m21?: number;
  m22?: number;
  m41?: number;
  m42?: number;
}

/**
 * DOMMatrix (2D subset) — the affine 2D transform class used by Canvas
 * and exposed on `globalThis` per the DOMMatrix spec. Only the 2D fields
 * (a, b, c, d, e, f) are modeled; 3D matrix math is not implemented
 * because canvas 2D never uses it.
 *
 * Mutable. See `DOMMatrixReadOnly` for the immutable variant.
 *
 * Reference: https://drafts.fxtf.org/geometry/#dommatrix
 */
class DOMMatrixReadOnly {
  a: number = 1;
  b: number = 0;
  c: number = 0;
  d: number = 1;
  e: number = 0;
  f: number = 0;
  is2D: boolean = true;

  constructor(init?: number[] | DOMMatrix2DInit | string) {
    const fields = parseMatrixInit(init);
    this.a = fields.a;
    this.b = fields.b;
    this.c = fields.c;
    this.d = fields.d;
    this.e = fields.e;
    this.f = fields.f;
  }

  get m11(): number {
    return this.a;
  }
  get m12(): number {
    return this.b;
  }
  get m21(): number {
    return this.c;
  }
  get m22(): number {
    return this.d;
  }
  get m41(): number {
    return this.e;
  }
  get m42(): number {
    return this.f;
  }

  get isIdentity(): boolean {
    return (
      this.a === 1 &&
      this.b === 0 &&
      this.c === 0 &&
      this.d === 1 &&
      this.e === 0 &&
      this.f === 0
    );
  }

  /** Returns a new DOMMatrix = this * other. */
  multiply(other: DOMMatrix2DInit | DOMMatrixReadOnly): DOMMatrix {
    const o = parseMatrixInit(other as DOMMatrix2DInit);
    return new DOMMatrix([
      this.a * o.a + this.c * o.b,
      this.b * o.a + this.d * o.b,
      this.a * o.c + this.c * o.d,
      this.b * o.c + this.d * o.d,
      this.a * o.e + this.c * o.f + this.e,
      this.b * o.e + this.d * o.f + this.f,
    ]);
  }

  translate(tx: number, ty: number = 0): DOMMatrix {
    return this.multiply({ a: 1, b: 0, c: 0, d: 1, e: tx, f: ty });
  }

  scale(sx: number, sy: number = sx): DOMMatrix {
    return this.multiply({ a: sx, b: 0, c: 0, d: sy, e: 0, f: 0 });
  }

  /** Rotation in degrees per the DOMMatrix spec. */
  rotate(angleDegrees: number): DOMMatrix {
    const rad = (angleDegrees * Math.PI) / 180;
    const cos = Math.cos(rad);
    const sin = Math.sin(rad);
    return this.multiply({ a: cos, b: sin, c: -sin, d: cos, e: 0, f: 0 });
  }

  /** Returns the inverse; a NaN-filled matrix when not invertible, per spec. */
  inverse(): DOMMatrix {
    const det = this.a * this.d - this.b * this.c;
    if (det === 0 || !Number.isFinite(det)) {
      return new DOMMatrix([NaN, NaN, NaN, NaN, NaN, NaN]);
    }
    const inv = 1 / det;
    return new DOMMatrix([
      this.d * inv,
      -this.b * inv,
      -this.c * inv,
      this.a * inv,
      (this.c * this.f - this.d * this.e) * inv,
      (this.b * this.e - this.a * this.f) * inv,
    ]);
  }

  transformPoint(point: { x: number; y: number }): { x: number; y: number } {
    return {
      x: this.a * point.x + this.c * point.y + this.e,
      y: this.b * point.x + this.d * point.y + this.f,
    };
  }

  toFloat32Array(): Float32Array {
    // Canvas 2D uses only the 2D subset — expose the 6 meaningful entries
    // padded into a Float32Array of length 6.
    return new Float32Array([this.a, this.b, this.c, this.d, this.e, this.f]);
  }

  toString(): string {
    return `matrix(${this.a}, ${this.b}, ${this.c}, ${this.d}, ${this.e}, ${this.f})`;
  }
}

class DOMMatrix extends DOMMatrixReadOnly {
  constructor(init?: number[] | DOMMatrix2DInit | string) {
    super(init);
  }

  multiplySelf(other: DOMMatrix2DInit | DOMMatrixReadOnly): DOMMatrix {
    const m = this.multiply(other);
    this.a = m.a;
    this.b = m.b;
    this.c = m.c;
    this.d = m.d;
    this.e = m.e;
    this.f = m.f;
    return this;
  }

  translateSelf(tx: number, ty: number = 0): DOMMatrix {
    return this.multiplySelf({ a: 1, b: 0, c: 0, d: 1, e: tx, f: ty });
  }

  scaleSelf(sx: number, sy: number = sx): DOMMatrix {
    return this.multiplySelf({ a: sx, b: 0, c: 0, d: sy, e: 0, f: 0 });
  }

  rotateSelf(angleDegrees: number): DOMMatrix {
    const rad = (angleDegrees * Math.PI) / 180;
    const cos = Math.cos(rad);
    const sin = Math.sin(rad);
    return this.multiplySelf({ a: cos, b: sin, c: -sin, d: cos, e: 0, f: 0 });
  }

  invertSelf(): DOMMatrix {
    const inv = this.inverse();
    this.a = inv.a;
    this.b = inv.b;
    this.c = inv.c;
    this.d = inv.d;
    this.e = inv.e;
    this.f = inv.f;
    return this;
  }
}

/** Coerce any supported DOMMatrix init form into a canonical 2D-affine struct. */
function parseMatrixInit(
  init?: number[] | DOMMatrix2DInit | DOMMatrixReadOnly | string,
): { a: number; b: number; c: number; d: number; e: number; f: number } {
  const identity = { a: 1, b: 0, c: 0, d: 1, e: 0, f: 0 };
  if (init === undefined || init === null) return identity;
  if (Array.isArray(init)) {
    if (init.length === 6) {
      return {
        a: init[0],
        b: init[1],
        c: init[2],
        d: init[3],
        e: init[4],
        f: init[5],
      };
    }
    if (init.length === 16) {
      return {
        a: init[0],
        b: init[1],
        c: init[4],
        d: init[5],
        e: init[12],
        f: init[13],
      };
    }
    throw new TypeError(
      `DOMMatrix constructor: array must have length 6 or 16, got ${init.length}`,
    );
  }
  if (typeof init === "string") {
    // Minimal parser for `matrix(a,b,c,d,e,f)` — other CSS transform forms
    // fall back to identity.
    const m = init.match(
      /matrix\(\s*([-+.eE0-9]+)\s*,\s*([-+.eE0-9]+)\s*,\s*([-+.eE0-9]+)\s*,\s*([-+.eE0-9]+)\s*,\s*([-+.eE0-9]+)\s*,\s*([-+.eE0-9]+)\s*\)/,
    );
    if (m) {
      return {
        a: parseFloat(m[1]),
        b: parseFloat(m[2]),
        c: parseFloat(m[3]),
        d: parseFloat(m[4]),
        e: parseFloat(m[5]),
        f: parseFloat(m[6]),
      };
    }
    return identity;
  }
  // DOMMatrix2DInit or existing DOMMatrix instance — read a..f with fallbacks
  // through the m11..m42 aliases.
  const obj = init as DOMMatrix2DInit;
  return {
    a: obj.a ?? obj.m11 ?? 1,
    b: obj.b ?? obj.m12 ?? 0,
    c: obj.c ?? obj.m21 ?? 0,
    d: obj.d ?? obj.m22 ?? 1,
    e: obj.e ?? obj.m41 ?? 0,
    f: obj.f ?? obj.m42 ?? 0,
  };
}

/**
 * Per-spec normalization for roundRect's `radii` argument.
 * Accepts a single number, a DOMPointInit-like `{x, y}`, or an array of
 * 1-4 of either. Collapses to a single representative radius (the max
 * corner radius) until the renderer gains per-corner support.
 *
 * Throws `RangeError` for negative or non-finite values per the spec.
 */
/**
 * True if `value` is a Path2D instance. Used to dispatch the union
 * overloads on CanvasRenderingContext2D methods (fill, stroke, clip,
 * isPointInPath, isPointInStroke).
 */
function isPath2D(value: unknown): value is Path2D {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof (value as Path2D).getRid === "function"
  );
}

function normalizeRoundRectRadii(
  radii?:
    | number
    | { x: number; y: number }
    | Array<number | { x: number; y: number }>,
): [number, number, number, number] {
  const coerce = (r: number | { x: number; y: number } | undefined): number => {
    if (r === undefined) return 0;
    if (typeof r === "number") {
      if (!Number.isFinite(r) || r < 0) {
        throw new RangeError(
          `The radius provided (${r}) is negative or non-finite.`,
        );
      }
      return r;
    }
    const rx = typeof r.x === "number" ? r.x : 0;
    const ry = typeof r.y === "number" ? r.y : 0;
    if (!Number.isFinite(rx) || !Number.isFinite(ry) || rx < 0 || ry < 0) {
      throw new RangeError(
        `A radius provided (${rx},${ry}) is negative or non-finite.`,
      );
    }
    // DOMPointInit {x, y} collapses to max(x, y) for this pass; true
    // elliptical per-corner radii are a renderer follow-up.
    return Math.max(rx, ry);
  };
  if (radii === undefined || radii === null) return [0, 0, 0, 0];
  if (!Array.isArray(radii)) {
    const v = coerce(radii);
    return [v, v, v, v];
  }
  if (radii.length === 0) return [0, 0, 0, 0];
  if (radii.length > 4) {
    throw new RangeError(
      `roundRect accepts at most 4 radii, received ${radii.length}.`,
    );
  }
  const c = radii.map(coerce);
  // Spec distribution rules: 1 → all four; 2 → (tl/br, tr/bl);
  // 3 → (tl, tr/bl, br); 4 → (tl, tr, br, bl).
  switch (c.length) {
    case 1:
      return [c[0], c[0], c[0], c[0]];
    case 2:
      return [c[0], c[1], c[0], c[1]];
    case 3:
      return [c[0], c[1], c[2], c[1]];
    default:
      return [c[0], c[1], c[2], c[3]];
  }
}

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
   * Adds a path to the current path, optionally transformed by a
   * `DOMMatrix2DInit` / `DOMMatrix`. The transform is applied point-by-
   * point to the source path's subpaths as they're copied in.
   */
  addPath(
    path: Path2D,
    transform?: DOMMatrix2DInit | DOMMatrixReadOnly | null,
  ): void {
    const m = parseMatrixInit(transform ?? undefined);
    __andromeda__.internal_path2d_add_path(
      this.#rid,
      path.getRid(),
      m.a,
      m.b,
      m.c,
      m.d,
      m.e,
      m.f,
    );
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
   * Adds a rounded rectangle to the path per the HTML Canvas spec.
   * `radii` accepts `number`, `{x, y}`, or an array of 1-4 of those.
   */
  roundRect(
    x: number,
    y: number,
    w: number,
    h: number,
    radii?:
      | number
      | { x: number; y: number }
      | Array<number | { x: number; y: number }>,
  ): void {
    const corners = normalizeRoundRectRadii(radii);
    // Path2D's Rust op accepts an array of radii and handles all four
    // distribution patterns internally via round_rect_web_api; pass the
    // fully-distributed [tl, tr, br, bl] to avoid double-normalization.
    __andromeda__.internal_path2d_round_rect(this.#rid, x, y, w, h, corners);
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
    return this.render()
      ? __andromeda__.internal_canvas_save_as_png(this.#rid, path)
      : false;
  }

  /**
   * Encode the canvas as a `Uint8Array` of image bytes.
   *
   * Supported types: `"image/png"` (default) and `"image/jpeg"`.
   * `quality` is in [0, 1] and only applies to JPEG (default 0.92).
   */
  toBuffer(type: string = "image/png", quality: number = 0.92): Uint8Array {
    this.render();
    const mime = (type ?? "image/png").toLowerCase();
    let csv: string;
    if (mime === "image/jpeg" || mime === "image/jpg") {
      const q = Number.isFinite(quality)
        ? Math.min(1, Math.max(0, quality))
        : 0.92;
      csv = __andromeda__.internal_canvas_encode_jpeg(this.#rid, q);
    } else if (mime === "image/png") {
      csv = __andromeda__.internal_canvas_encode_png(this.#rid);
    } else {
      throw new TypeError(
        `toBuffer: unsupported type "${type}" (only "image/png" and "image/jpeg" are supported).`,
      );
    }
    return decodeCsvBytes(csv);
  }

  /**
   * Encode the canvas as a `data:<mime>;base64,<payload>` URL string.
   *
   * Per the HTML spec, unsupported MIME types silently fall back to PNG.
   * `quality` in [0, 1] applies only to JPEG; default 0.92.
   */
  toDataURL(type: string = "image/png", quality: number = 0.92): string {
    this.render();
    const mime = (type ?? "image/png").toLowerCase();
    const q = Number.isFinite(quality)
      ? Math.min(1, Math.max(0, quality))
      : 0.92;
    return __andromeda__.internal_canvas_encode_data_url(this.#rid, mime, q);
  }

  /**
   * Encode the canvas as a Blob of image bytes.
   * Spec: returns a Promise even though encoding is synchronous here.
   */
  convertToBlob(options?: { type?: string; quality?: number }): Promise<Blob> {
    const type = options?.type ?? "image/png";
    const quality = options?.quality ?? 0.92;
    return new Promise((resolve, reject) => {
      try {
        const bytes = this.toBuffer(type, quality);
        resolve(new Blob([bytes], { type }));
      } catch (e) {
        reject(e);
      }
    });
  }
}

/**
 * Decode the comma-separated-decimal byte string used by the Rust-side
 * encode ops back into a `Uint8Array`.
 */
function decodeCsvBytes(csv: string): Uint8Array {
  if (typeof csv !== "string" || csv.length === 0) return new Uint8Array(0);
  const parts = csv.split(",");
  const out = new Uint8Array(parts.length);
  for (let i = 0; i < parts.length; i++) {
    const n = parseInt(parts[i], 10);
    out[i] = Number.isNaN(n) ? 0 : n;
  }
  return out;
}

/**
 * A 2D rendering context for Canvas
 */
type CanvasLineCap = "butt" | "round" | "square";
type CanvasLineJoin = "miter" | "round" | "bevel";
type CanvasTextAlign = "start" | "end" | "left" | "right" | "center";
type CanvasTextBaseline =
  | "top"
  | "hanging"
  | "middle"
  | "alphabetic"
  | "ideographic"
  | "bottom";
type CanvasDirection = "ltr" | "rtl" | "inherit";
type ImageSmoothingQuality = "low" | "medium" | "high";

class CanvasRenderingContext2D {
  #rid: number;
  #lineDashOffset: number = 0;
  #imageSmoothingEnabled: boolean = true;
  #imageSmoothingQuality: ImageSmoothingQuality = "low";
  #filter: string = "none";
  // Per-context bookkeeping so `fillStyle = grad; fillStyle === grad` is
  // true for both `CanvasGradient` and `CanvasPattern`. The Rust side
  // stores gradients and patterns as opaque numeric rids — without this
  // cache, the TS getter has no way to distinguish a gradient rid from a
  // pattern rid (the source of the collision bug at the old `mod.ts:300`
  // TODO). Keyed by `rid`, value is the JS instance that was set.
  #fillStyleInstances: Map<number, CanvasGradient | CanvasPattern> = new Map();
  #strokeStyleInstances: Map<number, CanvasGradient | CanvasPattern> =
    new Map();

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
   * Gets or sets the type of compositing operation to apply when drawing new shapes.
   * Valid values include: 'source-over', 'source-in', 'source-out', 'source-atop',
   * 'destination-over', 'destination-in', 'destination-out', 'destination-atop',
   * 'lighter', 'copy', 'xor', 'multiply', 'screen', 'overlay', 'darken', 'lighten',
   * 'color-dodge', 'color-burn', 'hard-light', 'soft-light', 'difference', 'exclusion',
   * 'hue', 'saturation', 'color', 'luminosity'.
   * Default is 'source-over'.
   */
  get globalCompositeOperation(): string {
    return __andromeda__.internal_canvas_get_global_composite_operation(
      this.#rid,
    );
  }

  set globalCompositeOperation(value: string) {
    __andromeda__.internal_canvas_set_global_composite_operation(
      this.#rid,
      value,
    );
  }

  /**
   * Gets or sets the current fill style for drawing operations.
   * Accepts CSS color strings like '#ff0000', 'rgb(255, 0, 0)', 'rgba(255, 0, 0, 0.5)', 'red', etc.,
   * or CanvasGradient/CanvasPattern objects.
   */
  get fillStyle(): string | CanvasGradient | CanvasPattern {
    const fillStyle = __andromeda__.internal_canvas_get_fill_style(this.#rid);
    if (typeof fillStyle === "number") {
      // Resolve to the JS instance that was set via the setter. Falls
      // back to a fresh CanvasGradient only if the rid has no recorded
      // owner — preserves referential equality per the HTML spec and
      // avoids the old gradient-vs-pattern collision.
      const cached = this.#fillStyleInstances.get(fillStyle);
      if (cached !== undefined) return cached;
      return new CanvasGradient(fillStyle);
    }
    return fillStyle as string;
  }

  set fillStyle(value: string | CanvasGradient | CanvasPattern) {
    if (typeof value === "string") {
      __andromeda__.internal_canvas_set_fill_style(this.#rid, value);
      return;
    }
    const rid = value[_fillId];
    this.#fillStyleInstances.set(rid, value);
    __andromeda__.internal_canvas_set_fill_style(this.#rid, rid);
  }
  /**
   * Gets or sets the current stroke style for drawing operations.
   * Accepts CSS color strings like '#ff0000', 'rgb(255, 0, 0)', 'rgba(255, 0, 0, 0.5)', 'red', etc.,
   * or CanvasGradient/CanvasPattern objects.
   */
  get strokeStyle(): string | CanvasGradient | CanvasPattern {
    const strokeStyle = __andromeda__.internal_canvas_get_stroke_style(
      this.#rid,
    );
    if (typeof strokeStyle === "number") {
      const cached = this.#strokeStyleInstances.get(strokeStyle);
      if (cached !== undefined) return cached;
      return new CanvasGradient(strokeStyle);
    }
    return strokeStyle as string;
  }

  set strokeStyle(value: string | CanvasGradient | CanvasPattern) {
    if (typeof value === "string") {
      __andromeda__.internal_canvas_set_stroke_style(this.#rid, value);
      return;
    }
    const rid = value[_fillId];
    this.#strokeStyleInstances.set(rid, value);
    // @ts-ignore - internal_canvas_set_stroke_style accepts numbers for gradients/patterns
    __andromeda__.internal_canvas_set_stroke_style(this.#rid, rid);
  }
  /**
   * Gets or sets the line width for drawing operations.
   * Default is 1.
   */
  get lineWidth(): number {
    return __andromeda__.internal_canvas_get_line_width(this.#rid);
  }

  set lineWidth(value: number) {
    if (!Number.isFinite(value) || value <= 0) return;
    __andromeda__.internal_canvas_set_line_width(this.#rid, value);
  }

  /**
   * Gets or sets the line cap style. One of "butt", "round", "square".
   * Default is "butt".
   */
  get lineCap(): CanvasLineCap {
    return __andromeda__.internal_canvas_get_line_cap(
      this.#rid,
    ) as CanvasLineCap;
  }

  set lineCap(value: CanvasLineCap) {
    if (value !== "butt" && value !== "round" && value !== "square") return;
    __andromeda__.internal_canvas_set_line_cap(this.#rid, value);
  }

  /**
   * Gets or sets the line join style. One of "miter", "round", "bevel".
   * Default is "miter".
   */
  get lineJoin(): CanvasLineJoin {
    return __andromeda__.internal_canvas_get_line_join(
      this.#rid,
    ) as CanvasLineJoin;
  }

  set lineJoin(value: CanvasLineJoin) {
    if (value !== "miter" && value !== "round" && value !== "bevel") return;
    __andromeda__.internal_canvas_set_line_join(this.#rid, value);
  }

  /**
   * Gets or sets the miter limit ratio.
   * Default is 10.
   */
  get miterLimit(): number {
    return __andromeda__.internal_canvas_get_miter_limit(this.#rid);
  }

  set miterLimit(value: number) {
    if (!Number.isFinite(value) || value <= 0) return;
    __andromeda__.internal_canvas_set_miter_limit(this.#rid, value);
  }

  /**
   * Sets the line dash pattern per the HTML Canvas spec: a single sequence
   * of non-negative, finite numbers. If an odd number of segments is given,
   * the sequence is duplicated per spec.
   *
   * `lineDashOffset` is preserved across calls.
   */
  setLineDash(segments: number[]): void {
    if (!Array.isArray(segments)) return;
    for (const n of segments) {
      if (typeof n !== "number" || !Number.isFinite(n) || n < 0) return;
    }
    const normalized =
      segments.length % 2 === 1 ? segments.concat(segments) : segments;
    __andromeda__.internal_canvas_set_line_dash(
      this.#rid,
      normalized,
      this.#lineDashOffset,
    );
  }

  /**
   * Returns the current line dash pattern as a sequence of numbers per spec.
   */
  getLineDash(): number[] {
    const json = __andromeda__.internal_canvas_get_line_dash(this.#rid);
    try {
      const info = JSON.parse(json);
      if (Array.isArray(info)) return info.slice();
      return Array.isArray(info?.dash) ? info.dash.slice() : [];
    } catch (_e) {
      if (typeof json === "string" && json.indexOf(",") !== -1) {
        return json
          .split(",")
          .map((s) => parseFloat(s.trim()))
          .filter((n) => !Number.isNaN(n));
      }
      return [];
    }
  }

  get lineDashOffset(): number {
    return this.#lineDashOffset;
  }

  set lineDashOffset(value: number) {
    if (!Number.isFinite(value)) return;
    this.#lineDashOffset = value;
    // Re-apply the current segments with the new offset so existing renderer
    // code (which reads offset from the same setter) stays in sync.
    const segments = this.getLineDash();
    __andromeda__.internal_canvas_set_line_dash(this.#rid, segments, value);
  }

  /**
   * Gets or sets whether image smoothing is enabled.
   * Default is true.
   *
   * Currently stored JS-side only; wiring to the GPU sampler is a follow-up.
   */
  get imageSmoothingEnabled(): boolean {
    return this.#imageSmoothingEnabled;
  }

  set imageSmoothingEnabled(value: boolean) {
    this.#imageSmoothingEnabled = !!value;
  }

  /**
   * Gets or sets image smoothing quality. One of "low", "medium", "high".
   * Default is "low".
   */
  get imageSmoothingQuality(): ImageSmoothingQuality {
    return this.#imageSmoothingQuality;
  }

  set imageSmoothingQuality(value: ImageSmoothingQuality) {
    if (value !== "low" && value !== "medium" && value !== "high") return;
    this.#imageSmoothingQuality = value;
  }

  /**
   * Gets or sets the current CSS filter string.
   * Default is "none". Currently stored but not applied in the renderer.
   */
  get filter(): string {
    return this.#filter;
  }

  set filter(value: string) {
    this.#filter = typeof value === "string" ? value : "none";
  }

  /**
   * Gets or sets the shadow blur amount.
   */
  get shadowBlur(): number {
    return __andromeda__.internal_canvas_get_shadow_blur(this.#rid);
  }

  set shadowBlur(value: number) {
    __andromeda__.internal_canvas_set_shadow_blur(this.#rid, value);
  }

  /**
   * Gets or sets the shadow color.
   * Accepts CSS color strings like '#000000', 'rgba(0, 0, 0, 0.5)', 'black', etc.
   */
  get shadowColor(): string {
    return __andromeda__.internal_canvas_get_shadow_color(this.#rid);
  }

  set shadowColor(value: string) {
    __andromeda__.internal_canvas_set_shadow_color(this.#rid, value);
  }

  /**
   * Gets or sets the shadow offset in the X direction.
   */
  get shadowOffsetX(): number {
    return __andromeda__.internal_canvas_get_shadow_offset_x(this.#rid);
  }

  set shadowOffsetX(value: number) {
    __andromeda__.internal_canvas_set_shadow_offset_x(this.#rid, value);
  }

  /**
   * Gets or sets the shadow offset in the Y direction.
   */
  get shadowOffsetY(): number {
    return __andromeda__.internal_canvas_get_shadow_offset_y(this.#rid);
  }

  set shadowOffsetY(value: number) {
    __andromeda__.internal_canvas_set_shadow_offset_y(this.#rid, value);
  }

  /**
   * Adds a circular arc to the current path.
   */
  arc(
    x: number,
    y: number,
    radius: number,
    startAngle: number,
    endAngle: number,
    counterclockwise?: boolean,
  ): void {
    if (
      !Number.isFinite(x) ||
      !Number.isFinite(y) ||
      !Number.isFinite(radius) ||
      !Number.isFinite(startAngle) ||
      !Number.isFinite(endAngle)
    ) {
      return;
    }
    if (radius < 0) {
      throw new DOMException(
        `The radius provided (${radius}) is negative.`,
        "IndexSizeError",
      );
    }
    __andromeda__.internal_canvas_arc(
      this.#rid,
      x,
      y,
      radius,
      startAngle,
      endAngle,
      !!counterclockwise,
    );
  }

  /**
   * Creates an arc to the canvas.
   */
  arcTo(x1: number, y1: number, x2: number, y2: number, radius: number): void {
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
   * Creates a pattern using the specified image and repetition.
   * @param image An ImageBitmap to use as the pattern's image.
   * @param repetition A string indicating how to repeat the pattern's image.
   *   Possible values: "repeat" (both directions), "repeat-x" (horizontal only),
   *   "repeat-y" (vertical only), "no-repeat" (neither direction).
   *   If null or empty string, defaults to "repeat".
   * @returns A CanvasPattern object, or null if the image is not valid.
   */
  createPattern(
    image: ImageBitmap,
    repetition: string | null,
  ): CanvasPattern | null {
    const imageRid =
      typeof (image as unknown as { __getRid?: () => number }).__getRid ===
      "function"
        ? (image as unknown as { __getRid: () => number }).__getRid()
        : (image["#rid" as keyof ImageBitmap] as number);
    if (typeof imageRid !== "number") {
      return null;
    }

    // Normalize repetition parameter
    const rep =
      repetition === null || repetition === "" ? "repeat" : repetition;

    // Validate repetition value
    if (!["repeat", "repeat-x", "repeat-y", "no-repeat"].includes(rep)) {
      throw new TypeError(`Invalid repetition value: ${rep}`);
    }

    const patternRid = __andromeda__.internal_canvas_create_pattern(
      imageRid,
      rep,
    );
    return new CanvasPattern(patternRid);
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
   * Draws a rectangle that is stroked (outlined) according to the current strokeStyle.
   */
  strokeRect(x: number, y: number, width: number, height: number): void {
    __andromeda__.internal_canvas_stroke_rect(this.#rid, x, y, width, height);
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
   * Fills the current path or the given Path2D.
   */
  fill(pathOrRule?: Path2D | CanvasFillRule, _fillRule?: CanvasFillRule): void {
    if (isPath2D(pathOrRule)) {
      __andromeda__.internal_canvas_fill_path2d(
        this.#rid,
        (pathOrRule as Path2D).getRid(),
      );
      return;
    }
    __andromeda__.internal_canvas_fill(this.#rid);
  }

  /**
   * Strokes the current path or the given Path2D.
   */
  stroke(path?: Path2D): void {
    if (isPath2D(path)) {
      __andromeda__.internal_canvas_stroke_path2d(
        this.#rid,
        (path as Path2D).getRid(),
      );
      return;
    }
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
    radii?:
      | number
      | { x: number; y: number }
      | Array<number | { x: number; y: number }>,
  ): void {
    const [tl, tr, br, bl] = normalizeRoundRectRadii(radii);
    __andromeda__.internal_canvas_round_rect(
      this.#rid,
      x,
      y,
      width,
      height,
      tl,
      tr,
      br,
      bl,
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
   * Resets the rendering context to its default state per the Offscreen Canvas
   */
  reset(): void {
    __andromeda__.internal_canvas_reset_bitmap(this.#rid);
    __andromeda__.internal_canvas_reset_state_stack(this.#rid);
    this.resetTransform();
    this.beginPath();
    __andromeda__.internal_canvas_set_fill_style(this.#rid, "#000000");
    __andromeda__.internal_canvas_set_stroke_style(this.#rid, "#000000");
    __andromeda__.internal_canvas_set_line_width(this.#rid, 1);
    __andromeda__.internal_canvas_set_line_cap(this.#rid, "butt");
    __andromeda__.internal_canvas_set_line_join(this.#rid, "miter");
    __andromeda__.internal_canvas_set_miter_limit(this.#rid, 10);
    __andromeda__.internal_canvas_set_line_dash(this.#rid, [], 0);
    __andromeda__.internal_canvas_set_global_alpha(this.#rid, 1);
    __andromeda__.internal_canvas_set_global_composite_operation(
      this.#rid,
      "source-over",
    );
    __andromeda__.internal_canvas_set_shadow_blur(this.#rid, 0);
    __andromeda__.internal_canvas_set_shadow_color(
      this.#rid,
      "rgba(0, 0, 0, 0)",
    );
    __andromeda__.internal_canvas_set_shadow_offset_x(this.#rid, 0);
    __andromeda__.internal_canvas_set_shadow_offset_y(this.#rid, 0);
    __andromeda__.internal_canvas_set_font(this.#rid, "10px sans-serif");
    __andromeda__.internal_canvas_set_text_align(this.#rid, "start");
    __andromeda__.internal_canvas_set_text_baseline(this.#rid, "alphabetic");
    __andromeda__.internal_canvas_set_direction(this.#rid, "inherit");
    this.#lineDashOffset = 0;
    this.#imageSmoothingEnabled = true;
    this.#imageSmoothingQuality = "low";
    this.#filter = "none";
  }

  /**
   * Returns whether the context is lost. Andromeda is a headless runtime
   * and never loses its canvas context, so this always returns false.
   */
  isContextLost(): boolean {
    return false;
  }

  /**
   * Adds a rotation to the transformation matrix.
   * @param angle The rotation angle, clockwise in radians.
   */
  rotate(angle: number): void {
    __andromeda__.internal_canvas_rotate(this.#rid, angle);
  }

  /**
   * Adds a scaling transformation to the canvas units horizontally and/or vertically.
   * @param x Scaling factor in the horizontal direction.
   * @param y Scaling factor in the vertical direction.
   */
  scale(x: number, y: number): void {
    __andromeda__.internal_canvas_scale(this.#rid, x, y);
  }

  /**
   * Adds a translation transformation to the current matrix.
   * @param x Distance to move in the horizontal direction.
   * @param y Distance to move in the vertical direction.
   */
  translate(x: number, y: number): void {
    __andromeda__.internal_canvas_translate(this.#rid, x, y);
  }

  /**
   * Multiplies the current transformation with the matrix described by the arguments.
   */
  transform(
    a: number,
    b: number,
    c: number,
    d: number,
    e: number,
    f: number,
  ): void {
    __andromeda__.internal_canvas_transform(this.#rid, a, b, c, d, e, f);
  }

  /**
   * Resets the current transformation to the identity matrix, then multiplies
   * it by the given matrix. Per the HTML Canvas spec, accepts either six
   * floats or a `DOMMatrix2DInit` dictionary / `DOMMatrix` instance. Calling
   * with no arguments resets to identity.
   */
  setTransform(
    aOrMatrix?: number | DOMMatrix2DInit | DOMMatrixReadOnly | null,
    b?: number,
    c?: number,
    d?: number,
    e?: number,
    f?: number,
  ): void {
    if (aOrMatrix === undefined || aOrMatrix === null) {
      __andromeda__.internal_canvas_reset_transform(this.#rid);
      return;
    }
    if (typeof aOrMatrix === "number") {
      __andromeda__.internal_canvas_set_transform(
        this.#rid,
        aOrMatrix,
        b!,
        c!,
        d!,
        e!,
        f!,
      );
      return;
    }
    const m = parseMatrixInit(aOrMatrix);
    __andromeda__.internal_canvas_set_transform(
      this.#rid,
      m.a,
      m.b,
      m.c,
      m.d,
      m.e,
      m.f,
    );
  }

  /**
   * Resets the current transform to the identity matrix.
   */
  resetTransform(): void {
    __andromeda__.internal_canvas_reset_transform(this.#rid);
  }

  /**
   * Returns the current transformation matrix as a `DOMMatrix`
   */
  getTransform(): DOMMatrix {
    const json = __andromeda__.internal_canvas_get_transform(this.#rid);
    const parsed = JSON.parse(json) as DOMMatrix2DInit;
    return new DOMMatrix([
      parsed.a ?? 1,
      parsed.b ?? 0,
      parsed.c ?? 0,
      parsed.d ?? 1,
      parsed.e ?? 0,
      parsed.f ?? 0,
    ]);
  }

  /**
   * Gets or sets the current text font.
   */
  get font(): string {
    return __andromeda__.internal_canvas_get_font(this.#rid);
  }

  set font(value: string) {
    __andromeda__.internal_canvas_set_font(this.#rid, value);
  }

  /**
   * Gets or sets the text alignment.
   */
  get textAlign(): string {
    return __andromeda__.internal_canvas_get_text_align(this.#rid);
  }

  set textAlign(value: string) {
    __andromeda__.internal_canvas_set_text_align(this.#rid, value);
  }

  /**
   * Gets or sets the text baseline.
   */
  get textBaseline(): string {
    return __andromeda__.internal_canvas_get_text_baseline(this.#rid);
  }

  set textBaseline(value: string) {
    __andromeda__.internal_canvas_set_text_baseline(this.#rid, value);
  }

  /**
   * Gets or sets the text direction.
   */
  get direction(): string {
    return __andromeda__.internal_canvas_get_direction(this.#rid);
  }

  set direction(value: string) {
    __andromeda__.internal_canvas_set_direction(this.#rid, value);
  }

  /**
   * Returns a TextMetrics object containing the measured dimensions of the specified text.
   */
  measureText(text: string): TextMetrics {
    const json = __andromeda__.internal_canvas_measure_text(this.#rid, text);
    const data = JSON.parse(json);
    return new TextMetrics(
      data.width,
      data.actualBoundingBoxLeft,
      data.actualBoundingBoxRight,
      data.fontBoundingBoxAscent,
      data.fontBoundingBoxDescent,
      data.actualBoundingBoxAscent,
      data.actualBoundingBoxDescent,
      data.emHeightAscent,
      data.emHeightDescent,
      data.hangingBaseline,
      data.alphabeticBaseline,
      data.ideographicBaseline,
    );
  }

  /**
   * Fills a given text at the given (x,y) position.
   */
  fillText(text: string, x: number, y: number, maxWidth?: number): void {
    if (maxWidth !== undefined) {
      __andromeda__.internal_canvas_fill_text(this.#rid, text, x, y, maxWidth);
    } else {
      __andromeda__.internal_canvas_fill_text(this.#rid, text, x, y);
    }
  }

  /**
   * Strokes the outlines of a given text at the given (x,y) position.
   */
  strokeText(text: string, x: number, y: number, maxWidth?: number): void {
    if (maxWidth !== undefined) {
      __andromeda__.internal_canvas_stroke_text(
        this.#rid,
        text,
        x,
        y,
        maxWidth,
      );
    } else {
      __andromeda__.internal_canvas_stroke_text(this.#rid, text, x, y);
    }
  }

  /**
   * Determines whether the specified point is in the current subpath
   * or in the given Path2D.
   */
  isPointInPath(
    pathOrX: Path2D | number,
    xOrY: number,
    yOrRule?: number | CanvasFillRule,
    fillRule?: CanvasFillRule,
  ): boolean {
    if (isPath2D(pathOrX)) {
      const rule =
        (typeof yOrRule === "string" ? yOrRule : fillRule) || "nonzero";
      return __andromeda__.internal_canvas_is_point_in_path(
        (pathOrX as Path2D).getRid(),
        xOrY,
        (yOrRule as number) ?? 0,
        rule,
      );
    }
    const rule = (
      typeof yOrRule === "string" ? yOrRule : "nonzero"
    ) as CanvasFillRule;
    return __andromeda__.internal_canvas_is_point_in_current_path(
      this.#rid,
      pathOrX as number,
      xOrY,
      rule,
    );
  }

  /**
   * Determines whether the specified point is inside the stroked area
   * of the current subpath or of the given Path2D.
   */
  isPointInStroke(pathOrX: Path2D | number, xOrY: number, y?: number): boolean {
    if (isPath2D(pathOrX)) {
      return __andromeda__.internal_canvas_is_point_in_stroke(
        (pathOrX as Path2D).getRid(),
        xOrY,
        y ?? 0,
        this.lineWidth,
      );
    }
    return __andromeda__.internal_canvas_is_point_in_current_stroke(
      this.#rid,
      pathOrX as number,
      xOrY,
      this.lineWidth,
    );
  }

  /**
   * Draws an image onto the canvas.
   * Supports three overload patterns:
   * - drawImage(image, dx, dy)
   * - drawImage(image, dx, dy, dWidth, dHeight)
   * - drawImage(image, sx, sy, sWidth, sHeight, dx, dy, dWidth, dHeight)
   */
  drawImage(image: ImageBitmap, ...args: number[]): void {
    const imageRid = image["#rid" as keyof ImageBitmap] as number;

    if (args.length === 2) {
      // drawImage(image, dx, dy)
      const [dx, dy] = args;
      __andromeda__.internal_canvas_draw_image(
        this.#rid,
        imageRid,
        0,
        0,
        image.width,
        image.height,
        dx,
        dy,
        image.width,
        image.height,
      );
    } else if (args.length === 4) {
      // drawImage(image, dx, dy, dWidth, dHeight)
      const [dx, dy, dWidth, dHeight] = args;
      __andromeda__.internal_canvas_draw_image(
        this.#rid,
        imageRid,
        0,
        0,
        image.width,
        image.height,
        dx,
        dy,
        dWidth,
        dHeight,
      );
    } else if (args.length === 8) {
      // drawImage(image, sx, sy, sWidth, sHeight, dx, dy, dWidth, dHeight)
      const [sx, sy, sWidth, sHeight, dx, dy, dWidth, dHeight] = args;
      __andromeda__.internal_canvas_draw_image(
        this.#rid,
        imageRid,
        sx,
        sy,
        sWidth,
        sHeight,
        dx,
        dy,
        dWidth,
        dHeight,
      );
    } else {
      throw new TypeError(
        `Invalid number of arguments to drawImage: ${args.length}`,
      );
    }
  }

  /**
   * Creates a new blank ImageData object per the HTML Canvas spec.
   */
  createImageData(
    widthOrImageData: number | ImageData,
    height?: number,
  ): ImageData {
    if (typeof widthOrImageData === "number") {
      if (height === undefined || !Number.isFinite(height)) {
        throw new TypeError(
          "createImageData: height is required when width is a number",
        );
      }
      const rid = __andromeda__.internal_canvas_create_image_data(
        widthOrImageData,
        height,
      );
      return new ImageData(
        _internalImageDataCtor,
        rid,
        widthOrImageData,
        height,
      );
    }
    if (!(widthOrImageData instanceof ImageData)) {
      throw new TypeError(
        "createImageData: expected a number or ImageData instance.",
      );
    }
    const src = widthOrImageData;
    const rid = __andromeda__.internal_canvas_create_image_data(
      src.width,
      src.height,
    );
    return new ImageData(_internalImageDataCtor, rid, src.width, src.height);
  }

  /**
   * Returns an ImageData object representing the pixel data for a region of the canvas.
   */
  getImageData(sx: number, sy: number, sw: number, sh: number): ImageData {
    const rid = __andromeda__.internal_canvas_get_image_data(
      this.#rid,
      sx,
      sy,
      sw,
      sh,
    );
    return new ImageData(_internalImageDataCtor, rid, sw, sh);
  }

  /**
   * Paints data from an ImageData object onto the canvas.
   */
  putImageData(
    imageData: ImageData,
    dx: number,
    dy: number,
    dirtyX?: number,
    dirtyY?: number,
    dirtyWidth?: number,
    dirtyHeight?: number,
  ): void {
    const imageRid = (
      imageData as unknown as { __syncAndGetRid(): number }
    ).__syncAndGetRid();
    if (dirtyX !== undefined) {
      // Normalize: negative widths flip and reanchor per spec.
      let dx0 = dirtyX;
      let dy0 = dirtyY ?? 0;
      let dw = dirtyWidth ?? imageData.width;
      let dh = dirtyHeight ?? imageData.height;
      if (dw < 0) {
        dx0 += dw;
        dw = -dw;
      }
      if (dh < 0) {
        dy0 += dh;
        dh = -dh;
      }
      // Clamp to the image's own bounds.
      if (dx0 < 0) {
        dw += dx0;
        dx0 = 0;
      }
      if (dy0 < 0) {
        dh += dy0;
        dy0 = 0;
      }
      if (dx0 + dw > imageData.width) dw = imageData.width - dx0;
      if (dy0 + dh > imageData.height) dh = imageData.height - dy0;
      if (dw <= 0 || dh <= 0) return;
      const coversWhole =
        dx0 === 0 &&
        dy0 === 0 &&
        dw === imageData.width &&
        dh === imageData.height;
      if (!coversWhole) {
        throw new DOMException(
          "putImageData dirty-rect form is not yet supported by the Andromeda renderer; omit the dirty-rect arguments or pass the full image bounds.",
          "NotSupportedError",
        );
      }
    }
    __andromeda__.internal_canvas_put_image_data(this.#rid, imageRid, dx, dy);
  }

  /**
   * Turns the current path (or the given Path2D) into the clipping region.
   */
  clip(pathOrRule?: Path2D | CanvasFillRule, _fillRule?: CanvasFillRule): void {
    if (isPath2D(pathOrRule)) {
      __andromeda__.internal_canvas_clip_path2d(
        this.#rid,
        (pathOrRule as Path2D).getRid(),
      );
      return;
    }
    __andromeda__.internal_canvas_clip_current(this.#rid);
  }
}

const _fillId = Symbol("[[fillId]]");

class CanvasGradient {
  [_fillId]: number;

  constructor(rid: number) {
    this[_fillId] = rid;
  }
  /**
   * Adds a new color stop to the gradient.
   * Throws `IndexSizeError` if offset is not in the range [0, 1] per spec.
   */
  addColorStop(offset: number, color: string) {
    if (!Number.isFinite(offset) || offset < 0 || offset > 1) {
      throw new DOMException(
        `The offset provided (${offset}) is outside the range [0, 1].`,
        "IndexSizeError",
      );
    }
    if (typeof color !== "string") {
      throw new DOMException(
        `The color provided is not a string.`,
        "SyntaxError",
      );
    }
    __andromeda__.internal_canvas_gradient_add_color_stop(
      this[_fillId],
      offset,
      color,
    );
  }
}

/**
 * Represents a pattern created from an image.
 */
class CanvasPattern {
  [_fillId]: number;

  constructor(rid: number) {
    this[_fillId] = rid;
  }

  /**
   * Sets the transformation matrix that will be used when rendering the pattern.
   * Accepts `DOMMatrix`, `DOMMatrix2DInit`, or omitted (resets to identity).
   */
  setTransform(transform?: DOMMatrix2DInit | DOMMatrixReadOnly | null): void {
    if (transform === undefined || transform === null) {
      this.#transform = undefined;
      return;
    }
    const m = parseMatrixInit(transform);
    this.#transform = m;
  }

  #transform:
    | {
        a: number;
        b: number;
        c: number;
        d: number;
        e: number;
        f: number;
      }
    | undefined;
}

/**
 * Represents the underlying pixel data of an area of a canvas element.
 */
const _internalImageDataCtor = Symbol("[[internalImageDataCtor]]");

class ImageData {
  #rid: number;
  #width: number;
  #height: number;
  #userData: Uint8ClampedArray | undefined;
  #cached: Uint8ClampedArray | undefined;

  /**
   * Constructs an ImageData object.
   */
  constructor(
    firstArg: number | Uint8ClampedArray | typeof _internalImageDataCtor,
    widthOrHeightOrRid?: number,
    heightOrWidth?: number,
    height?: number,
  ) {
    if (firstArg === _internalImageDataCtor) {
      this.#rid = widthOrHeightOrRid as number;
      this.#width = heightOrWidth as number;
      this.#height = height as number;
      return;
    }
    if (firstArg instanceof Uint8ClampedArray) {
      const data = firstArg;
      const w = widthOrHeightOrRid as number;
      const h = heightOrWidth ?? data.length / (w * 4);
      if (!Number.isInteger(w) || w <= 0 || !Number.isInteger(h) || h <= 0) {
        throw new DOMException(
          "ImageData constructor: width and height must be positive integers.",
          "IndexSizeError",
        );
      }
      if (data.length !== w * h * 4) {
        throw new DOMException(
          `ImageData constructor: buffer length ${data.length} does not match width*height*4 = ${w * h * 4}.`,
          "InvalidStateError",
        );
      }
      const rid = __andromeda__.internal_canvas_create_image_data(w, h);
      this.#rid = rid;
      this.#width = w;
      this.#height = h;
      this.#userData = data;
      syncImageDataToRust(rid, data);
      return;
    }
    if (
      typeof firstArg === "number" &&
      typeof widthOrHeightOrRid === "number"
    ) {
      if (heightOrWidth !== undefined) {
        throw new TypeError(
          "ImageData constructor: too many arguments for the (width, height) form.",
        );
      }
      const width = firstArg;
      const h = widthOrHeightOrRid;
      if (
        !Number.isInteger(width) ||
        width <= 0 ||
        !Number.isInteger(h) ||
        h <= 0
      ) {
        throw new DOMException(
          "ImageData constructor: width and height must be positive integers.",
          "IndexSizeError",
        );
      }
      const rid = __andromeda__.internal_canvas_create_image_data(width, h);
      this.#rid = rid;
      this.#width = width;
      this.#height = h;
      return;
    }
    throw new TypeError(
      "ImageData constructor: expected (width, height) or (Uint8ClampedArray, width, height?).",
    );
  }

  /**
   * The width in pixels of the ImageData.
   */
  get width(): number {
    return this.#width;
  }

  /**
   * The height in pixels of the ImageData.
   */
  get height(): number {
    return this.#height;
  }

  /**
   * Pixel bytes in RGBA order, one byte per channel per pixel.
   */
  get data(): Uint8ClampedArray {
    if (this.#userData !== undefined) return this.#userData;
    if (this.#cached === undefined) {
      this.#cached = decodeImageDataBytes(
        __andromeda__.internal_image_data_get_data(this.#rid),
        this.#width * this.#height * 4,
      );
    }
    return this.#cached;
  }

  __syncAndGetRid(): number {
    if (this.#userData !== undefined) {
      syncImageDataToRust(this.#rid, this.#userData);
    } else if (this.#cached !== undefined) {
      syncImageDataToRust(this.#rid, this.#cached);
    }
    return this.#rid;
  }
}

// TODO: implement into nova's JS API a more efficient way to transfer large binary data buffers into Rust, to avoid this O(n) stringification step on every sync.
function syncImageDataToRust(rid: number, data: Uint8ClampedArray): void {
  const parts = new Array<string>(data.length);
  for (let i = 0; i < data.length; i++) parts[i] = data[i].toString();
  __andromeda__.internal_image_data_set_data(rid, parts.join(","));
}

/**
 * Parse the byte representation returned by `internal_image_data_get_data`
 * into a fixed-length `Uint8ClampedArray`.
 */
function decodeImageDataBytes(
  raw: unknown,
  expectedLength: number,
): Uint8ClampedArray {
  const out = new Uint8ClampedArray(expectedLength);
  if (typeof raw !== "string" || raw.length === 0) return out;
  try {
    const parsed = JSON.parse(raw);
    if (Array.isArray(parsed)) {
      const n = Math.min(parsed.length, expectedLength);
      for (let i = 0; i < n; i++) out[i] = parsed[i] | 0;
      return out;
    }
  } catch (_) {
    // Fall through to CSV parsing below.
  }
  const stripped =
    raw.startsWith("[") && raw.endsWith("]") ? raw.slice(1, -1) : raw;
  let i = 0;
  for (const piece of stripped.split(",")) {
    if (i >= expectedLength) break;
    const n = parseInt(piece, 10);
    if (!Number.isNaN(n)) out[i++] = n;
  }
  return out;
}

/**
 * Represents the dimensions of a piece of text in the canvas.
 * Returned by CanvasRenderingContext2D.measureText().
 */
class TextMetrics {
  readonly width: number;
  readonly actualBoundingBoxLeft: number;
  readonly actualBoundingBoxRight: number;
  readonly fontBoundingBoxAscent: number;
  readonly fontBoundingBoxDescent: number;
  readonly actualBoundingBoxAscent: number;
  readonly actualBoundingBoxDescent: number;
  readonly emHeightAscent: number;
  readonly emHeightDescent: number;
  readonly hangingBaseline: number;
  readonly alphabeticBaseline: number;
  readonly ideographicBaseline: number;

  constructor(
    width: number,
    actualBoundingBoxLeft: number,
    actualBoundingBoxRight: number,
    fontBoundingBoxAscent: number,
    fontBoundingBoxDescent: number,
    actualBoundingBoxAscent: number,
    actualBoundingBoxDescent: number,
    emHeightAscent: number,
    emHeightDescent: number,
    hangingBaseline: number,
    alphabeticBaseline: number,
    ideographicBaseline: number,
  ) {
    this.width = width;
    this.actualBoundingBoxLeft = actualBoundingBoxLeft;
    this.actualBoundingBoxRight = actualBoundingBoxRight;
    this.fontBoundingBoxAscent = fontBoundingBoxAscent;
    this.fontBoundingBoxDescent = fontBoundingBoxDescent;
    this.actualBoundingBoxAscent = actualBoundingBoxAscent;
    this.actualBoundingBoxDescent = actualBoundingBoxDescent;
    this.emHeightAscent = emHeightAscent;
    this.emHeightDescent = emHeightDescent;
    this.hangingBaseline = hangingBaseline;
    this.alphabeticBaseline = alphabeticBaseline;
    this.ideographicBaseline = ideographicBaseline;
  }
}

// Export classes to global scope
Object.assign(globalThis, {
  Path2D,
  OffscreenCanvas,
  ImageData,
  CanvasPattern,
  CanvasGradient,
  TextMetrics,
  DOMMatrix,
  DOMMatrixReadOnly,
});
