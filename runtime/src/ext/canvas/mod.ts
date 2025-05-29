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
   * Get a drawing context (not yet implemented).
   */
  getContext(type: string): never {
    throw new Error("Canvas context not implemented");
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
