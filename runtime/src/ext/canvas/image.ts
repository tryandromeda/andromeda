// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// deno-lint-ignore-file no-unused-vars

/**
 * A bitmap image resource.
 */
class ImageBitmap {
  #rid: number;
  /** The width of the image in pixels. */
  width: number;
  /** The height of the image in pixels. */
  height: number;

  constructor(rid: number, width: number, height: number) {
    this.#rid = rid;
    this.width = width;
    this.height = height;
  }
}

/**
 * Creates an ImageBitmap from a file path or URL.
 * @param path The file path or URL to load.
 */
// deno-lint-ignore require-await
async function createImageBitmap(path: string): Promise<ImageBitmap> {
  const rid = internal_image_bitmap_create(path);
  const width = internal_image_bitmap_get_width(rid);
  const height = internal_image_bitmap_get_height(rid);
  return new ImageBitmap(rid, width, height);
}
