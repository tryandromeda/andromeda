// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// deno-lint-ignore-file no-unused-vars

const _width = Symbol("[[width]]");
const _height = Symbol("[[height]]");
const _detached = Symbol("[[detached]]");

/**
 * A bitmap image resource.
 */
class ImageBitmap {
  #rid: number;
  [_width]: number;
  [_height]: number;
  [_detached]: boolean = false;
  constructor(rid: number, width: number, height: number) {
    this.#rid = rid;
    this[_width] = width;
    this[_height] = height;
  }

  /**
   * Gets the width of the image bitmap.
   */
  get width(): number {
    if (this[_detached]) {
      throw new Error("ImageBitmap has been detached.");
    }
    return this[_width];
  }

  /**
   * Gets the height of the image bitmap.
   */
  get height(): number {
    if (this[_detached]) {
      throw new Error("ImageBitmap has been detached.");
    }
    return this[_height];
  }
}

/**
 * Creates an ImageBitmap from a file path or URL.
 * @param path The file path or URL to load.
 */
function createImageBitmap(path: string): ImageBitmap {
  const rid = __andromeda__.internal_image_bitmap_create(path);
  const width = __andromeda__.internal_image_bitmap_get_width(rid);
  const height = __andromeda__.internal_image_bitmap_get_height(rid);
  return new ImageBitmap(rid, width, height);
}
