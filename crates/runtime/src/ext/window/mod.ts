// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// deno-lint-ignore-file no-explicit-any

const __andromeda_window_registry: Map<number, AndromedaWindow> = new Map();

interface AndromedaWindowEvent {
  type: string;
  detail: any;
  target: AndromedaWindow;
}

type AndromedaWindowListener = (event: AndromedaWindowEvent) => void;

class AndromedaWindow {
  #rid: number;
  #closed = false;
  #title: string;
  #width: number;
  #height: number;
  #listeners: Map<string, Set<AndromedaWindowListener>> = new Map();

  constructor(
    options: {
      title?: string;
      width?: number;
      height?: number;
      resizable?: boolean;
      visible?: boolean;
    } = {},
  ) {
    const title = options.title ?? "Andromeda";
    const width = options.width ?? 800;
    const height = options.height ?? 600;
    const payload = JSON.stringify({
      title,
      width,
      height,
      resizable: options.resizable ?? true,
      visible: options.visible ?? true,
    });
    // @ts-ignore - internal op surface
    const ridStr = (globalThis as any).__andromeda__.internal_window_create(
      payload,
    );
    this.#rid = Number(ridStr);
    this.#title = title;
    this.#width = width;
    this.#height = height;
    __andromeda_window_registry.set(this.#rid, this);
    try {
      // @ts-ignore - internal op surface
      const rawSize = (
        globalThis as any
      ).__andromeda__.internal_window_get_size(this.#rid);
      const parsed = JSON.parse(rawSize);
      this.#width = parsed.width;
      this.#height = parsed.height;
    } catch {
      // Leave the requested values as a best-effort fallback.
    }
  }

  addEventListener(type: string, listener: AndromedaWindowListener): void {
    if (typeof listener !== "function") return;
    let set = this.#listeners.get(type);
    if (!set) {
      set = new Set();
      this.#listeners.set(type, set);
    }
    set.add(listener);
  }

  removeEventListener(type: string, listener: AndromedaWindowListener): void {
    const set = this.#listeners.get(type);
    if (set) set.delete(listener);
  }

  dispatchEvent(event: AndromedaWindowEvent): boolean {
    const set = this.#listeners.get(event.type);
    if (!set) return true;
    for (const listener of Array.from(set)) {
      try {
        listener.call(this, event);
      } catch (err) {
        console.error("[Andromeda.Window] listener threw:", err);
      }
    }
    return true;
  }

  get rid(): number {
    return this.#rid;
  }

  get title(): string {
    return this.#title;
  }

  get width(): number {
    return this.#width;
  }

  get height(): number {
    return this.#height;
  }

  get closed(): boolean {
    return this.#closed;
  }

  close(): void {
    if (this.#closed) return;
    this.#closed = true;
    __andromeda_window_registry.delete(this.#rid);
    // @ts-ignore - internal op surface
    (globalThis as any).__andromeda__.internal_window_close(this.#rid);
  }

  /**
   * Return the native window + display handles along with current size. The
   * shape matches `Deno.UnsafeWindowSurface`'s input so a future WebGPU
   * surface bridge can pass it through verbatim. Pointer-sized values come
   * back as strings; cast to `BigInt` on the caller side if needed.
   */
  rawHandle(): {
    system: "cocoa" | "win32" | "x11" | "wayland";
    windowHandle: string;
    displayHandle: string;
    width: number;
    height: number;
  } {
    // @ts-ignore - internal op surface
    const raw = (globalThis as any).__andromeda__.internal_window_raw_handle(
      this.#rid,
    );
    return JSON.parse(raw);
  }

  setTitle(title: string): void {
    // @ts-ignore - internal op surface
    (globalThis as any).__andromeda__.internal_window_set_title(
      this.#rid,
      String(title),
    );
    this.#title = String(title);
  }

  getSize(): { width: number; height: number; scaleFactor: number } {
    // @ts-ignore - internal op surface
    const raw = (globalThis as any).__andromeda__.internal_window_get_size(
      this.#rid,
    );
    const parsed = JSON.parse(raw);
    this.#width = parsed.width;
    this.#height = parsed.height;
    return parsed;
  }

  setSize(width: number, height: number): void {
    // @ts-ignore - internal op surface
    (globalThis as any).__andromeda__.internal_window_set_size(
      this.#rid,
      width | 0,
      height | 0,
    );
    this.#width = width | 0;
    this.#height = height | 0;
  }

  setVisible(visible: boolean): void {
    // @ts-ignore - internal op surface
    (globalThis as any).__andromeda__.internal_window_set_visible(
      this.#rid,
      !!visible,
    );
  }

  /**
   * Clear the window to a solid RGBA color and present. Channel values are
   * in the 0..1 range. Lazily initializes the shared wgpu context and the
   * window's surface on first call.
   */
  present(r: number, g: number, b: number, a: number = 1): void {
    // @ts-ignore - internal op surface
    (globalThis as any).__andromeda__.internal_window_present_color(
      this.#rid,
      Number(r),
      Number(g),
      Number(b),
      Number(a),
    );
  }

  /**
   * Flush any pending draw commands on the given canvas and present its
   * latest frame into this window's swapchain. Requires the runtime to
   * be built with both the `window` and `canvas` features enabled (the
   * op is absent otherwise and this method throws a clear error).
   */
  presentCanvas(canvas: { rid: number }): void {
    // @ts-ignore - internal op surface
    const op = (globalThis as any).__andromeda__.internal_window_present_canvas;
    if (typeof op !== "function") {
      throw new Error(
        "presentCanvas requires both the `window` and `canvas` features to be enabled.",
      );
    }
    if (
      !canvas ||
      typeof canvas.rid !== "number" ||
      !Number.isInteger(canvas.rid) ||
      canvas.rid < 0
    ) {
      throw new TypeError(
        "presentCanvas: expected an object with a non-negative integer `rid` field",
      );
    }
    op(this.#rid, canvas.rid);
  }

  // Internal — used by mainloop to patch cached dimensions after a resize.
  _updateSize(width: number, height: number): void {
    this.#width = width;
    this.#height = height;
  }
}

function __andromeda_window_poll(): Array<{
  rid: number;
  type: string;
  detail: any;
}> {
  // @ts-ignore - internal op surface
  const raw = (globalThis as any).__andromeda__.internal_window_poll_events();
  if (!raw) return [];
  try {
    return JSON.parse(raw);
  } catch {
    return [];
  }
}

async function __andromeda_window_mainloop(
  callback?: () => void | Promise<void>,
): Promise<void> {
  // Run until every registered window is closed. If the user's callback
  // throws, close every open window before propagating so windows don't
  // stay visible on screen after an unhandled rejection.
  try {
    while (__andromeda_window_registry.size > 0) {
      const events = __andromeda_window_poll();
      for (const e of events) {
        const target = __andromeda_window_registry.get(e.rid);
        if (!target) continue;
        if (e.type === "resize" && e.detail) {
          target._updateSize(e.detail.width, e.detail.height);
        }
        target.dispatchEvent({ type: e.type, detail: e.detail, target });
        if (e.type === "close") {
          target.close();
        }
      }
      if (callback) {
        await callback();
      }
      // Yield to the event loop so timers/promises can run.
      // Prefer a shared already-resolved promise to avoid per-frame timer allocation churn.
      await Promise.resolve();
    }
  } finally {
    for (const w of Array.from(__andromeda_window_registry.values())) {
      w.close();
    }
  }
}
// @ts-ignore - cross-file handoff
(globalThis as any).__andromeda_window_class = AndromedaWindow;
// @ts-ignore - cross-file handoff
(globalThis as any).__andromeda_window_mainloop = __andromeda_window_mainloop;
