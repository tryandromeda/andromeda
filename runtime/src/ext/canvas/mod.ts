// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// deno-lint-ignore-file no-unused-vars

// deno-lint-ignore-file no-unused-vars
// @ts-ignore: provided by Andromeda runtime
declare function internal_canvas_create(width: number, height: number): number;
// @ts-ignore: provided by Andromeda runtime
declare function internal_canvas_get_width(rid: number): number;
// @ts-ignore: provided by Andromeda runtime
declare function internal_canvas_get_height(rid: number): number;
/**
 * A minimal Canvas implementation with internal op bindings.
 */
export class Canvas {
  private rid: number;
  constructor(width: number, height: number) {
    this.rid = internal_canvas_create(width, height);
  }
  /** Get the width of the canvas. */
  getWidth(): number {
    return internal_canvas_get_width(this.rid);
  }
  /** Get the height of the canvas. */
  getHeight(): number {
    return internal_canvas_get_height(this.rid);
  }
  /** Get a drawing context (not yet implemented). */
  getContext(type: string): never {
    throw new Error("Canvas context not implemented");
  }
}

/** Convenience factory to create a Canvas. */
export function createCanvas(width: number, height: number): Canvas {
  return new Canvas(width, height);
}
