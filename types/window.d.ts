// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * Andromeda Window API — inspired by `deno-windowing/dwm`, backed by winit.
 */
export interface CreateWindowOptions {
  /** OS window title. Defaults to `"Andromeda"`. */
  title?: string;
  /** Inner width in logical pixels. Defaults to `800`. */
  width?: number;
  /** Inner height in logical pixels. Defaults to `600`. */
  height?: number;
  /** Whether the user can resize the window. Defaults to `true`. */
  resizable?: boolean;
  /** Whether the window is visible on creation. Defaults to `true`. */
  visible?: boolean;
}

export interface ResizeEventDetail {
  width: number;
  height: number;
  scaleFactor: number;
}

/**
 * Detail payload for `keydown` / `keyup` events on an `Andromeda.Window`.
 *
 * Values match the web platform's `KeyboardEvent` contract so code written
 * against the browser's event object carries over 1:1 (modulo the plain-
 * object wrapper — see {@link WindowEvent}).
 */
export interface KeyEventDetail {
  /**
   * The `key` attribute value per the UI Events spec.
   * https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/key
   */
  key: string;

  /**
   * The `code` attribute — physical key identifier independent of layout.
   * https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/code
   */
  code: string;

  /**
   * Deprecated legacy numeric code per the MDN keyCode table.
   * https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode
   */
  keyCode: number;

  /**
   * Alias of {@link keyCode} — always equal. Present only because legacy
   * code sometimes reads `event.which` instead.
   */
  which: number;

  /**
   * Physical key location.
   * https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/location
   */
  location: 0 | 1 | 2 | 3;

  /** True when the Alt / Option modifier is held. */
  altKey: boolean;
  /** True when the Control modifier is held. */
  ctrlKey: boolean;
  /** True when the Meta / Command / Super modifier is held. */
  metaKey: boolean;
  /** True when the Shift modifier is held. */
  shiftKey: boolean;

  /** True when the event is a held-key auto-repeat. */
  repeat: boolean;

  /**
   * Whether the event is part of an IME composition session.
   * https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/isComposing
   */
  isComposing: boolean;
}

export interface MouseEventDetail {
  x: number;
  y: number;
  /** DOM-aligned button index: 0=left, 1=middle, 2=right, 3/4=back/forward, -1 on mousemove. */
  button: number;
  /** Bitmask of currently pressed buttons. */
  buttons: number;
  altKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  shiftKey: boolean;
}

/**
 * A native OS window.
 */
export interface RawWindowHandleData {
  /** Native windowing system for this handle. */
  system: "cocoa" | "win32" | "x11" | "wayland";
  /** Pointer-sized window handle, encoded as a string (convert with `BigInt`). */
  windowHandle: string;
  /** Pointer-sized display handle, encoded as a string (`"0"` when not applicable). */
  displayHandle: string;
  width: number;
  height: number;
}

/** Plain-object event dispatched to window listeners.*/
export interface WindowEvent<D = unknown> {
  type: string;
  detail: D;
  target: Window;
}

export type WindowEventListener<D = unknown> = (event: WindowEvent<D>) => void;

export declare class Window {
  constructor(options?: CreateWindowOptions);
  readonly rid: number;
  readonly title: string;
  readonly width: number;
  readonly height: number;
  readonly closed: boolean;
  close(): void;

  addEventListener(type: string, listener: WindowEventListener): void;
  removeEventListener(type: string, listener: WindowEventListener): void;
  dispatchEvent(event: WindowEvent): boolean;

  /**
   * Return the native window + display handles and current size. Shape-
   * compatible with `Deno.UnsafeWindowSurface` so WebGPU-surface bridges can
   * pass the object through verbatim.
   */
  rawHandle(): RawWindowHandleData;

  setTitle(title: string): void;
  getSize(): { width: number; height: number; scaleFactor: number };
  setSize(width: number, height: number): void;
  setVisible(visible: boolean): void;

  /**
   * Clear the window's swapchain to an RGBA color and present a frame.
   * Channel values are 0..1. Useful as a smoke test of the render loop
   * before wiring richer canvas/WebGPU surface integrations.
   */
  present(r: number, g: number, b: number, a?: number): void;

  /**
   * Present the latest frame of an `OffscreenCanvas` into this window.
   * Flushes any pending canvas 2D draw commands and then blits the
   * rendered texture to the swapchain via a scaled fullscreen pass. The
   * canvas may be any size; it will stretch to fill the window.
   *
   * Requires the runtime to be built with both the `window` and `canvas`
   * features enabled; throws a clear error otherwise.
   */
  presentCanvas(canvas: OffscreenCanvas): void;

  /**
   * Drive the winit event loop, dispatching events to all open windows.
   * Returns once every open window has been closed.
   *
   * @param callback Optional per-frame callback invoked after events are
   *   dispatched. Useful for rendering or state updates.
   */
  static mainloop(callback?: () => void | Promise<void>): Promise<void>;
}

/** Shorthand for `new Andromeda.Window(options)`. */
export declare function createWindow(options?: CreateWindowOptions): Window;

declare global {
  namespace Andromeda {
    export {
      createWindow,
      CreateWindowOptions,
      KeyEventDetail,
      MouseEventDetail,
      RawWindowHandleData,
      ResizeEventDetail,
      Window,
    };
  }
}
