// deno-lint-ignore-file no-explicit-any
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * The `assert` function tests if a condition is true.
 *
 * @example
 * ```ts
 * assert(1 === 1, "The condition is true!");
 * ```
 */
declare function assert(condition: boolean, message: string): void;

/**
 * The `assertEquals` function tests if two values are equal.
 *
 * @example
 * ```ts
 * assertEquals(1, 1, "The values are equal!");
 * ```
 */
declare function assertEquals<T>(value1: T, value2: T, message: string): void;

/**
 * The `assertNotEquals` function tests if two values are not equal.
 *
 * @example
 * ```ts
 * assertNotEquals(1, 2, "The values are not equal!");
 * ```
 */
declare function assertNotEquals<T>(
  value1: T,
  value2: T,
  message: string,
): void;

/**
 * The `assertThrows` function tests if a function throws an error.
 *
 * @example
 * ```ts
 * assertThrows(() => {
 *  throw new Error("Hello, World!");
 * }, "An error occurred!");
 */
declare function assertThrows(fn: () => void, message: string): void;

/**
 * The Andromeda namespace for the Andromeda runtime.
 */
declare namespace Andromeda {
  /**
   * The `args` property contains the command-line arguments.
   */
  const args: string[];

  // Text file operations
  /**
   * readTextFileSync reads a text file from the file system.
   *
   * @example
   * ```ts
   * const data = Andromeda.readTextFileSync("hello.txt");
   * console.log(data);
   * ```
   */
  function readTextFileSync(path: string): string;

  /**
   * readTextFile asynchronously reads a text file from the file system.
   *
   * @example
   * ```ts
   * const data = await Andromeda.readTextFile("hello.txt");
   * console.log(data);
   * ```
   */
  function readTextFile(path: string): Promise<string>;

  /**
   * writeTextFileSync writes a text file to the file system.
   *
   * @example
   * ```ts
   * Andromeda.writeTextFileSync("hello.txt", "Hello, World!");
   * ```
   */
  function writeTextFileSync(path: string, data: string): void;

  /**
   * writeTextFile asynchronously writes a text file to the file system.
   *
   * @example
   * ```ts
   * await Andromeda.writeTextFile("hello.txt", "Hello, World!");
   * ```
   */
  function writeTextFile(path: string, data: string): Promise<void>;

  // Binary file operations
  /**
   * readFileSync reads a binary file from the file system.
   *
   * @example
   * ```ts
   * const data = Andromeda.readFileSync("image.png");
   * console.log(data);
   * ```
   */
  function readFileSync(path: string): Uint8Array;

  /**
   * readFile asynchronously reads a binary file from the file system.
   *
   * @example
   * ```ts
   * const data = await Andromeda.readFile("image.png");
   * console.log(data);
   * ```
   */
  function readFile(path: string): Promise<Uint8Array>;

  /**
   * writeFileSync writes binary data to a file in the file system.
   *
   * @example
   * ```ts
   * const data = new Uint8Array([72, 101, 108, 108, 111]);
   * Andromeda.writeFileSync("data.bin", data);
   * ```
   */
  function writeFileSync(path: string, data: Uint8Array): void;

  /**
   * writeFile asynchronously writes binary data to a file in the file system.
   *
   * @example
   * ```ts
   * const data = new Uint8Array([72, 101, 108, 108, 111]);
   * await Andromeda.writeFile("data.bin", data);
   * ```
   */
  function writeFile(path: string, data: Uint8Array): Promise<void>;

  // Async file operations
  /**
   * create asynchronously creates a new empty file in the file system.
   *
   * @example
   * ```ts
   * await Andromeda.create("hello.txt");
   * ```
   */
  function create(path: string): Promise<void>;

  /**
   * copyFile asynchronously copies a file in the file system.
   *
   * @example
   * ```ts
   * await Andromeda.copyFile("hello.txt", "world.txt");
   * ```
   */
  function copyFile(source: string, destination: string): Promise<void>;

  /**
   * remove asynchronously removes a file from the file system.
   *
   * @example
   * ```ts
   * await Andromeda.remove("hello.txt");
   * ```
   */
  function remove(path: string): Promise<void>;

  // Sync file operations
  /**
   * createSync creates a new empty file in the file system.
   *
   * @example
   * ```ts
   * Andromeda.createSync("hello.txt");
   * ```
   */
  function createSync(path: string): void;

  /**
   * copyFileSync copies a file in the file system.
   *
   * @example
   * ```ts
   * Andromeda.copyFileSync("hello.txt", "world.txt");
   * ```
   */
  function copyFileSync(source: string, destination: string): void;

  /**
   * removeSync removes a file from the file system.
   *
   * @example
   * ```ts
   * Andromeda.removeSync("hello.txt");
   * ```
   */
  function removeSync(path: string): void;

  /**
   * removeAllSync recursively removes a file or directory from the file system.
   *
   * @example
   * ```ts
   * Andromeda.removeAllSync("my_directory");
   * ```
   */
  function removeAllSync(path: string): void;

  /**
   * removeAll asynchronously removes a file or directory recursively from the file system.
   *
   * @example
   * ```ts
   * await Andromeda.removeAll("my_directory");
   * ```
   */
  function removeAll(path: string): Promise<void>;

  /**
   * renameSync renames/moves a file or directory in the file system.
   *
   * @example
   * ```ts
   * Andromeda.renameSync("old_name.txt", "new_name.txt");
   * ```
   */
  function renameSync(oldPath: string, newPath: string): void;

  /**
   * rename asynchronously renames/moves a file or directory in the file system.
   *
   * @example
   * ```ts
   * await Andromeda.rename("old_name.txt", "new_name.txt");
   * ```
   */
  function rename(oldPath: string, newPath: string): Promise<void>;

  /**
   * existsSync checks if a file or directory exists in the file system.
   *
   * @example
   * ```ts
   * if (Andromeda.existsSync("hello.txt")) {
   *   console.log("File exists!");
   * }
   * ```
   */
  function existsSync(path: string): boolean;

  /**
   * exists asynchronously checks if a file or directory exists in the file system.
   *
   * @example
   * ```ts
   * if (await Andromeda.exists("hello.txt")) {
   *   console.log("File exists!");
   * }
   * ```
   */
  function exists(path: string): Promise<boolean>;

  /**
   * truncateSync truncates a file to a specified length.
   *
   * @example
   * ```ts
   * Andromeda.truncateSync("hello.txt", 100);
   * ```
   */
  function truncateSync(path: string, length: number): void;

  /**
   * chmodSync changes the permissions of a file or directory.
   *
   * @example
   * ```ts
   * Andromeda.chmodSync("hello.txt", 0o644);
   * ```
   */
  function chmodSync(path: string, mode: number): void;

  /**
   * openSync opens a file and returns a file descriptor.
   *
   * @example
   * ```ts
   * const fd = Andromeda.openSync("hello.txt", "r");
   * console.log("File descriptor:", fd);
   * ```
   */
  function openSync(path: string, mode: string): number;

  // Directory operations
  /**
   * mkdirSync creates a directory in the file system.
   *
   * @example
   * ```ts
   * Andromeda.mkdirSync("hello");
   * ```
   */
  function mkdirSync(path: string): void;

  /**
   * mkdir asynchronously creates a directory in the file system.
   *
   * @example
   * ```ts
   * await Andromeda.mkdir("hello");
   * ```
   */
  function mkdir(path: string): Promise<void>;

  /**
   * mkdirAllSync creates a directory and all its parent directories.
   *
   * @example
   * ```ts
   * Andromeda.mkdirAllSync("path/to/deep/directory");
   * ```
   */
  function mkdirAllSync(path: string): void;

  /**
   * mkdirAll asynchronously creates a directory and all its parent directories.
   *
   * @example
   * ```ts
   * await Andromeda.mkdirAll("path/to/deep/directory");
   * ```
   */
  function mkdirAll(path: string): Promise<void>;

  // System operations
  /**
   * exit exits the program with an optional exit code.
   *
   * @example
   * ```ts
   * Andromeda.exit(0);
   * ```
   */
  function exit(code?: number): void;

  /**
   * Returns a Promise to be resolved after the specified time in milliseconds.
   *
   * @example
   * ```ts
   * Andromeda.sleep(1000).then(() => {
   *  console.log("Hello, World!");
   * });
   */
  function sleep(duration: number): Promise<void>;

  namespace stdin {
    /**
     * readLine reads a line from standard input.
     *
     * @example
     * ```ts
     * const name = Andromeda.stdin.readLine();
     * console.log(`Hello, ${name}!`);
     * ```
     */
    function readLine(): string;
  }

  /**
   * stdout namespace for writing to standard output.
   */
  namespace stdout {
    /**
     * write writes a string to standard output.
     *
     * @example
     * ```ts
     * Andromeda.stdout.write("Hello, World!");
     * ```
     */
    function write(message: string): void;
  }

  /**
   * env namespace for environment variables.
   */
  namespace env {
    /**
     * get returns the value of an environment variable.
     *
     * @example
     * ```ts
     * const value = Andromeda.env.get("HOME");
     * console.log(value);
     * ```
     */
    function get(key: string): string;

    /**
     * set sets the value of an environment variable.
     *
     * @example
     * ```ts
     * Andromeda.env.set("HOME", "/home/user");
     * ```
     */
    function set(key: string, value: string): void;

    /**
     * remove deletes an environment variable.
     *
     * @example
     * ```ts
     * Andromeda.env.remove("HOME");
     * ```
     */
    function remove(key: string): void;

    /**
     * keys returns the keys of all environment variables.
     *
     * @example
     * ```ts     * const keys = Andromeda.env.keys();
     * console.log(keys);
     * ```
     */
    function keys(): string[];
  }

  // Signal handling types
  type Signal =
    | "SIGABRT"
    | "SIGALRM"
    | "SIGBREAK"
    | "SIGBUS"
    | "SIGCHLD"
    | "SIGCONT"
    | "SIGEMT"
    | "SIGFPE"
    | "SIGHUP"
    | "SIGILL"
    | "SIGINFO"
    | "SIGINT"
    | "SIGIO"
    | "SIGPOLL"
    | "SIGUNUSED"
    | "SIGKILL"
    | "SIGPIPE"
    | "SIGPROF"
    | "SIGPWR"
    | "SIGQUIT"
    | "SIGSEGV"
    | "SIGSTKFLT"
    | "SIGSTOP"
    | "SIGSYS"
    | "SIGTERM"
    | "SIGTRAP"
    | "SIGTSTP"
    | "SIGTTIN"
    | "SIGTTOU"
    | "SIGURG"
    | "SIGUSR1"
    | "SIGUSR2"
    | "SIGVTALRM"
    | "SIGWINCH"
    | "SIGXCPU"
    | "SIGXFSZ";

  /**
   * Registers the given function as a listener of the given signal event.
   *
   * @example
   * ```ts
   * Andromeda.addSignalListener(
   *   "SIGTERM",
   *   () => {
   *     console.log("SIGTERM!")
   *   }
   * );
   * ```
   *
   * Note: On Windows only "SIGINT" (Ctrl+C) and "SIGBREAK" (Ctrl+Break) are supported.
   */
  function addSignalListener(signal: Signal, handler: () => void): void;

  /**
   * Removes the given function as a listener of the given signal event.
   *
   * @example
   * ```ts
   * Andromeda.removeSignalListener("SIGTERM", myHandler);
   * ```
   */
  function removeSignalListener(signal: Signal, handler: () => void): void;

  // Cron API types
  /**
   * CronScheduleExpression defines the different ways to specify a time component in a cron schedule.
   */
  type CronScheduleExpression = number | { exact: number | number[]; } | {
    start?: number;
    end?: number;
    every?: number;
  };

  /**
   * CronSchedule is the interface used for JSON format cron schedule.
   */
  interface CronSchedule {
    minute?: CronScheduleExpression;
    hour?: CronScheduleExpression;
    dayOfMonth?: CronScheduleExpression;
    month?: CronScheduleExpression;
    dayOfWeek?: CronScheduleExpression;
  }

  /**
   * Create a cron job that will periodically execute the provided handler
   * callback based on the specified schedule.
   *
   * ```ts
   * Andromeda.cron("sample cron", "20 * * * *", () => {
   *   console.log("cron job executed");
   * });
   * ```
   *
   * ```ts
   * Andromeda.cron("sample cron", { hour: { every: 6 } }, () => {
   *   console.log("cron job executed");
   * });
   * ```
   *
   * `schedule` can be a string in the Unix cron format or in JSON format
   * as specified by interface {@linkcode CronSchedule}, where time is specified
   * using UTC time zone.
   */
  function cron(
    name: string,
    schedule: string | CronSchedule,
    handler: () => Promise<void> | void,
  ): Promise<void>;

  /**
   * Create a cron job that will periodically execute the provided handler
   * callback based on the specified schedule.
   *
   * ```ts
   * Andromeda.cron("sample cron", "20 * * * *", {
   *   backoffSchedule: [100, 1000, 5000],
   *   signal: abortController.signal,
   * }, () => {
   *   console.log("cron job executed");
   * });
   * ```
   */
  function cron(
    name: string,
    schedule: string | CronSchedule,
    options: { backoffSchedule?: number[]; signal?: AbortSignal; },
    handler: () => Promise<void> | void,
  ): Promise<void>;
}
/**
 * The `prompt` function prompts the user for input.
 *
 * @example
 * ```ts
 * const name = prompt("What is your name?");
 * console.log(`Hello, ${name}!`);
 * ```
 */
declare function prompt(message: string): string;

/**
 * The `confirm` function prompts the user for confirmation.
 */
declare function confirm(message: string): boolean;

// Extension to Navigator interface for Web Locks API
interface Navigator {
  /** The LockManager for Web Locks API */
  readonly locks: LockManager;
}

/**
 * Options for structuredClone function
 */
interface StructuredSerializeOptions {
  /**
   * An array of transferable objects that will be transferred rather than cloned.
   * The objects will be rendered unusable in the sending context after the transfer.
   */
  // @ts-ignore Deno type issues
  transfer?: any[];
}

/**
 * Creates a deep clone of a given value using the structured clone algorithm.
 *
 * The structured clone algorithm copies complex JavaScript objects. It supports many built-in
 * data types and can handle circular references. However, it cannot clone functions, symbols,
 * or certain platform objects.
 *
 * @param value - The object to be cloned
 * @param options - Options for the cloning operation, including transferable objects
 * @returns A deep clone of the original value
 * @throws DataCloneError if the value cannot be cloned
 *
 * @example
 * ```ts
 * // Clone a simple object
 * const original = { a: 1, b: [2, 3] };
 * const cloned = structuredClone(original);
 *
 * // Clone with circular references
 * const circular = { self: null };
 * circular.self = circular;
 * const clonedCircular = structuredClone(circular);
 *
 * // Transfer an ArrayBuffer (makes original unusable)
 * const buffer = new ArrayBuffer(8);
 * const transferred = structuredClone(buffer, { transfer: [buffer] });
 * ```
 */
declare function structuredClone<T = any>(
  value: T,
  options?: StructuredSerializeOptions,
): T;

/**
 * An offscreen Canvas implementation.
 */
declare class OffscreenCanvas {
  /**
   * Create a new off-screen canvas with the given dimensions.
   */
  constructor(width: number, height: number);
  /** Get the width of the canvas. */
  getWidth(): number;
  /** Get the height of the canvas. */
  getHeight(): number;
  /**
   * Returns a 2D rendering context or null if not available.
   */
  getContext(type: "2d"): CanvasRenderingContext2D | null;
  /**
   * Renders the canvas to finalize GPU operations and optionally extract pixel data.
   * Returns true if rendering was successful, false otherwise.
   */
  render(): boolean;
  /**
   * Saves the canvas as a PNG image file.
   * Returns true if save was successful, false otherwise.
   */
  saveAsPng(path: string): boolean;
}

/**
 * The 2D rendering context for a Canvas.
 */
declare class CanvasRenderingContext2D {
  /** Gets or sets the current fill style for drawing operations. */
  fillStyle: string | CanvasGradient;
  /** Gets or sets the current stroke style for drawing operations. */
  strokeStyle: string;
  /** Gets or sets the line width for drawing operations. */
  lineWidth: number;
  /** Gets or sets the global alpha value (transparency) for drawing operations. Values range from 0.0 (transparent) to 1.0 (opaque). */
  globalAlpha: number;
  /** Gets or sets the type of compositing operation to apply when drawing new shapes. Valid values include 'source-over', 'source-in', 'source-out', 'source-atop', 'destination-over', 'destination-in', 'destination-out', 'destination-atop', 'lighter', 'copy', 'xor', 'multiply', 'screen', 'overlay', 'darken', 'lighten', 'color-dodge', 'color-burn', 'hard-light', 'soft-light', 'difference', 'exclusion', 'hue', 'saturation', 'color', and 'luminosity'. Default is 'source-over'. */
  globalCompositeOperation: string;
  /** Creates an arc/curve on the canvas context. */
  arc(
    x: number,
    y: number,
    radius: number,
    startAngle: number,
    endAngle: number,
  ): void;
  /** Creates an arc-to command on the canvas context. */
  arcTo(x1: number, y1: number, x2: number, y2: number, radius: number): void;
  /** Begins a new path on the canvas context. */
  beginPath(): void;
  /** Adds a cubic Bézier curve to the path. */
  bezierCurveTo(
    cp1x: number,
    cp1y: number,
    cp2x: number,
    cp2y: number,
    x: number,
    y: number,
  ): void;
  /** Clears the specified rectangular area, making it fully transparent. */
  clearRect(x: number, y: number, width: number, height: number): void;
  /** Creates a gradient along the line connecting two given coordinates. */
  createLinearGradient(
    x0: number,
    y0: number,
    x1: number,
    y1: number,
  ): CanvasGradient;
  /** Creates a radial gradient using the size and coordinates of two circles. */
  createRadialGradient(
    x0: number,
    y0: number,
    r0: number,
    x1: number,
    y1: number,
    r1: number,
  ): CanvasGradient;
  /** Creates a gradient around a point with given coordinates. */
  createConicGradient(
    startAngle: number,
    x: number,
    y: number,
  ): CanvasGradient;
  /** Closes the current path on the canvas context. */
  closePath(): void;
  /** Draws a filled rectangle whose starting corner is at (x, y). */
  fillRect(x: number, y: number, width: number, height: number): void;
  /** Moves the path starting point to the specified coordinates. */
  moveTo(x: number, y: number): void;
  /** Connects the last point in the current sub-path to the specified coordinates with a straight line. */
  lineTo(
    x: number,
    y: number,
  ): void; /** Fills the current path with the current fill style. */
  fill(): void;
  /** Strokes the current path with the current stroke style. */
  stroke(): void; /** Adds a rectangle to the current path. */
  rect(x: number, y: number, width: number, height: number): void;
  /** Adds a quadratic Bézier curve to the current path. */
  quadraticCurveTo(cpx: number, cpy: number, x: number, y: number): void;
  /** Adds an ellipse to the current path. */
  ellipse(
    x: number,
    y: number,
    radiusX: number,
    radiusY: number,
    rotation: number,
    startAngle: number,
    endAngle: number,
    counterclockwise?: boolean,
  ): void;
  /** Adds a rounded rectangle to the current path. */
  roundRect(
    x: number,
    y: number,
    w: number,
    h: number,
    radii: number | number[],
  ): void;
  /** Saves the current canvas state (styles, transformations, etc.) to a stack. */
  save(): void;
  /** Restores the most recently saved canvas state from the stack. */
  restore(): void;
}

declare class CanvasGradient {
  /** Adds a new color stop to a given canvas gradient. */
  addColorStop(offset: number, color: string): void;
}

/**
 * A bitmap image resource.
 */
// @ts-ignore ImageBitmap is available in Deno's scope
declare class ImageBitmap {
  /** The width of the image in pixels. */
  readonly width: number;
  /** The height of the image in pixels. */
  readonly height: number;
}

/**
 * Creates an ImageBitmap from a file path or URL.
 * @param path The file path or URL to load.
 */
declare function createImageBitmap(path: string): Promise<ImageBitmap>;

/**
 * Options for acquiring a Web Lock
 */
interface LockOptions {
  /**
   * The mode of the lock. Default is "exclusive".
   * - "exclusive": Only one holder allowed at a time
   * - "shared": Multiple holders allowed simultaneously
   */
  mode?: "exclusive" | "shared";

  /**
   * If true, the request will fail if the lock cannot be granted immediately.
   * The callback will be invoked with null.
   */
  ifAvailable?: boolean;

  /**
   * If true, any held locks with the same name will be released,
   * and the request will be granted, preempting any queued requests.
   */
  steal?: boolean;

  /**
   * An AbortSignal that can be used to abort the lock request.
   */
  signal?: AbortSignal;
}

/**
 * Information about a lock for query results
 */
interface LockInfo {
  /** The name of the lock */
  name: string;
  /** The mode of the lock */
  mode: "exclusive" | "shared";
  /** An identifier for the client holding or requesting the lock */
  clientId?: string;
}

/**
 * Result of a query operation
 */
interface LockManagerSnapshot {
  /** Currently held locks */
  held: LockInfo[];
  /** Pending lock requests */
  pending: LockInfo[];
}

/**
 * Represents a granted Web Lock
 */
interface Lock {
  /** The name of the lock */
  readonly name: string;
  /** The mode of the lock */
  readonly mode: "exclusive" | "shared";
}

/**
 * The LockManager interface provides methods for requesting locks and querying lock state
 */
interface LockManager {
  /**
   * Request a lock and execute a callback while holding it
   * @param name The name of the lock
   * @param callback The callback to execute while holding the lock
   * @returns A promise that resolves with the return value of the callback
   */
  request<T>(
    name: string,
    callback: (lock: Lock) => T | Promise<T>,
  ): Promise<T>;

  /**
   * Request a lock with options and execute a callback while holding it
   * @param name The name of the lock
   * @param options Options for acquiring the lock
   * @param callback The callback to execute while holding the lock
   * @returns A promise that resolves with the return value of the callback
   */
  request<T>(
    name: string,
    options: LockOptions,
    callback: (lock: Lock | null) => T | Promise<T>,
  ): Promise<T>;

  /**
   * Query the current state of locks
   * @returns A promise that resolves with information about held and pending locks
   */
  query(): Promise<LockManagerSnapshot>;
}

/**
 * TextEncoder interface for encoding strings to UTF-8 bytes
 */
// @ts-ignore Deno type issues
interface TextEncoder {
  /**
   * The encoding name, always "utf-8"
   */
  readonly encoding: string;

  /**
   * Encodes a string into a Uint8Array of UTF-8 bytes
   * @param input The string to encode
   */
  encode(input?: string): Uint8Array;

  /**
   * Encodes a string into a Uint8Array of UTF-8 bytes with streaming support
   * @param source The string to encode
   * @param options Encoding options
   */
  encodeInto(
    source: string,
    destination: Uint8Array,
  ): TextEncoderEncodeIntoResult;
}

/**
 * Result of TextEncoder.encodeInto operation
 */
interface TextEncoderEncodeIntoResult {
  /**
   * Number of UTF-16 code units read from the source
   */
  read: number;

  /**
   * Number of bytes written to the destination
   */
  written: number;
}

/**
 * TextDecoder interface for decoding UTF-8 bytes to strings
 */
// @ts-ignore Deno type issues
interface TextDecoder {
  /**
   * The encoding name
   */
  readonly encoding: string;

  /**
   * Whether the decoder will throw on invalid sequences
   */
  readonly fatal: boolean;

  /**
   * Whether the decoder ignores BOM
   */
  readonly ignoreBOM: boolean;

  /**
   * Decodes a buffer into a string
   * @param input The buffer to decode
   * @param options Decoding options
   */
  decode(input?: BufferSource, options?: TextDecodeOptions): string;
}

/**
 * Options for TextDecoder.decode
 */
interface TextDecodeOptions {
  /**
   * Whether the decoder should continue decoding in subsequent calls
   */
  stream?: boolean;
}

/**
 * Constructor options for TextDecoder
 */
interface TextDecoderOptions {
  /**
   * Whether to throw on invalid sequences
   */
  fatal?: boolean;

  /**
   * Whether to ignore BOM
   */
  ignoreBOM?: boolean;
}

/**
 * TextEncoder constructor
 */
// @ts-ignore Deno type issues
declare const TextEncoder: {
  new(): TextEncoder;
};

/**
 * TextDecoder constructor
 */
// @ts-ignore Deno type issues
declare const TextDecoder: {
  new(label?: string, options?: TextDecoderOptions): TextDecoder;
};

/**
 * Buffer source types for crypto operations
 */
// @ts-ignore Deno type issues
type BufferSource = ArrayBufferView | ArrayBuffer;

/**
 * Key formats supported by the Web Crypto API
 */
// @ts-ignore Deno type issues
type KeyFormat = "raw" | "spki" | "pkcs8" | "jwk";

/**
 * Key types
 */
// @ts-ignore Deno type issues
type KeyType = "public" | "private" | "secret";

/**
 * Key usages
 */
// @ts-ignore Deno type issues
type KeyUsage =
  | "encrypt"
  | "decrypt"
  | "sign"
  | "verify"
  | "deriveKey"
  | "deriveBits"
  | "wrapKey"
  | "unwrapKey";

/**
 * Hash algorithm identifiers
 */
// @ts-ignore Deno type issues
type HashAlgorithmIdentifier =
  | AlgorithmIdentifier
  | "SHA-1"
  | "SHA-256"
  | "SHA-384"
  | "SHA-512";

/**
 * Algorithm identifier
 */
// @ts-ignore Deno type issues
type AlgorithmIdentifier = string | Algorithm;

/**
 * Base algorithm interface
 */
interface Algorithm {
  name: string;
}

/**
 * JSON Web Key interface
 */
interface JsonWebKey {
  alg?: string;
  crv?: string;
  d?: string;
  dp?: string;
  dq?: string;
  e?: string;
  ext?: boolean;
  k?: string;
  key_ops?: string[];
  kty?: string;
  n?: string;
  oth?: RsaOtherPrimesInfo[];
  p?: string;
  q?: string;
  qi?: string;
  use?: string;
  x?: string;
  y?: string;
}

/**
 * RSA other primes info
 */
interface RsaOtherPrimesInfo {
  d?: string;
  r?: string;
  t?: string;
}

/**
 * CryptoKey interface
 */
interface CryptoKey {
  readonly algorithm: KeyAlgorithm;
  readonly extractable: boolean;
  readonly type: KeyType;
  readonly usages: KeyUsage[];
}

/**
 * CryptoKeyPair interface
 */
interface CryptoKeyPair {
  // @ts-ignore Deno type issues
  readonly privateKey: CryptoKey;
  // @ts-ignore Deno type issues
  readonly publicKey: CryptoKey;
}

/**
 * Key algorithm interface
 */
interface KeyAlgorithm {
  name: string;
}

/**
 * AES key generation parameters
 */
interface AesKeyGenParams extends Algorithm {
  name: "AES-CTR" | "AES-CBC" | "AES-GCM";
  // @ts-ignore Deno type issues
  length: 128 | 192 | 256;
}

/**
 * AES-CTR parameters
 */
interface AesCtrParams extends Algorithm {
  name: "AES-CTR";
  counter: BufferSource;
  length: number;
}

/**
 * AES-CBC parameters
 */
interface AesCbcParams extends Algorithm {
  name: "AES-CBC";
  iv: BufferSource;
}

/**
 * AES-GCM parameters
 */
interface AesGcmParams extends Algorithm {
  name: "AES-GCM";
  iv: BufferSource;
  additionalData?: BufferSource;
  tagLength?: number;
}

/**
 * RSA hashed key generation parameters
 */
interface RsaHashedKeyGenParams extends Algorithm {
  name: "RSASSA-PKCS1-v1_5" | "RSA-PSS" | "RSA-OAEP";
  modulusLength: number;
  publicExponent: Uint8Array;
  hash: HashAlgorithmIdentifier;
}

/**
 * RSA PKCS#1 v1.5 parameters
 */
interface RsaPkcs1v15Params extends Algorithm {
  name: "RSASSA-PKCS1-v1_5";
}

/**
 * RSA-PSS parameters
 */
interface RsaPssParams extends Algorithm {
  name: "RSA-PSS";
  saltLength: number;
}

/**
 * RSA-OAEP parameters
 */
interface RsaOaepParams extends Algorithm {
  name: "RSA-OAEP";
  // @ts-ignore Deno type issues
  label?: BufferSource;
}

/**
 * HMAC key generation parameters
 */
interface HmacKeyGenParams extends Algorithm {
  name: "HMAC";
  hash: HashAlgorithmIdentifier;
  length?: number;
}

/**
 * HMAC parameters
 */
interface HmacParams extends Algorithm {
  name: "HMAC";
}

/**
 * EC key generation parameters
 */
interface EcKeyGenParams extends Algorithm {
  name: "ECDSA" | "ECDH";
  namedCurve: string;
}

/**
 * ECDSA parameters
 */
interface EcdsaParams extends Algorithm {
  name: "ECDSA";
  hash: HashAlgorithmIdentifier;
}

/**
 * ECDH key derivation parameters
 */
interface EcdhKeyDeriveParams extends Algorithm {
  name: "ECDH";
  public: CryptoKey;
}

/**
 * PBKDF2 parameters
 */
interface Pbkdf2Params extends Algorithm {
  name: "PBKDF2";
  salt: BufferSource;
  iterations: number;
  hash: HashAlgorithmIdentifier;
}

/**
 * SubtleCrypto interface providing low-level cryptographic primitives
 * following the W3C Web Crypto API specification
 */
interface SubtleCrypto {
  /**
   * Decrypts data using the specified algorithm and key
   */
  decrypt(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    data: BufferSource,
  ): Promise<ArrayBuffer>;

  /**
   * Derives a key from a base key using the specified algorithm
   */
  deriveKey(
    algorithm: AlgorithmIdentifier,
    baseKey: CryptoKey,
    derivedKeyType: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;

  /**
   * Derives bits from a base key using the specified algorithm
   */
  deriveBits(
    algorithm: AlgorithmIdentifier,
    baseKey: CryptoKey,
    length: number,
  ): Promise<ArrayBuffer>;

  /**
   * Computes a digest of the given data using the specified algorithm
   */
  digest(
    algorithm: AlgorithmIdentifier,
    data: BufferSource,
  ): Promise<ArrayBuffer>;

  /**
   * Encrypts data using the specified algorithm and key
   */
  encrypt(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    data: BufferSource,
  ): Promise<ArrayBuffer>;

  /**
   * Exports a key in the specified format
   */
  exportKey(format: "jwk", key: CryptoKey): Promise<JsonWebKey>;
  exportKey(
    format: Exclude<KeyFormat, "jwk">,
    key: CryptoKey,
  ): Promise<ArrayBuffer>;
  exportKey(
    format: KeyFormat,
    key: CryptoKey,
  ): Promise<JsonWebKey | ArrayBuffer>;

  /**
   * Generates a key or key pair using the specified algorithm
   */
  generateKey(
    algorithm: RsaHashedKeyGenParams | EcKeyGenParams,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKeyPair>;
  generateKey(
    algorithm: AesKeyGenParams | HmacKeyGenParams | Pbkdf2Params,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;
  generateKey(
    algorithm: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKeyPair | CryptoKey>;

  /**
   * Imports a key from external data
   */
  importKey(
    format: "jwk",
    keyData: JsonWebKey,
    algorithm: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;
  importKey(
    format: Exclude<KeyFormat, "jwk">,
    keyData: BufferSource,
    algorithm: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;
  importKey(
    format: KeyFormat,
    keyData: JsonWebKey | BufferSource,
    algorithm: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;

  /**
   * Signs data using the specified algorithm and key
   */
  sign(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    data: BufferSource,
  ): Promise<ArrayBuffer>;

  /**
   * Unwraps a wrapped key
   */
  unwrapKey(
    format: KeyFormat,
    wrappedKey: BufferSource,
    unwrappingKey: CryptoKey,
    unwrapAlgorithm: AlgorithmIdentifier,
    unwrappedKeyAlgorithm: AlgorithmIdentifier,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;

  /**
   * Verifies a signature using the specified algorithm and key
   */
  verify(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    signature: BufferSource,
    data: BufferSource,
  ): Promise<boolean>;

  /**
   * Wraps a key using the specified algorithm
   */
  wrapKey(
    format: KeyFormat,
    key: CryptoKey,
    wrappingKey: CryptoKey,
    wrapAlgorithm: AlgorithmIdentifier,
  ): Promise<ArrayBuffer>;
}

/**
 * Crypto interface following the W3C Web Crypto API specification
 */
interface Crypto {
  /**
   * The SubtleCrypto interface provides access to common cryptographic primitives
   */
  readonly subtle: SubtleCrypto;

  /**
   * Returns a cryptographically secure random UUID string
   */
  randomUUID(): string;

  /**
   * Fills the given typed array with cryptographically secure random values
   */
  getRandomValues<
    T extends
      | Int8Array
      | Uint8Array
      | Uint8ClampedArray
      | Int16Array
      | Uint16Array
      | Int32Array
      | Uint32Array
      | BigInt64Array
      | BigUint64Array,
  >(array: T): T;
}

/**
 * Global crypto instance following the W3C Web Crypto API specification
 */
// @ts-ignore Deno type issues
declare const crypto: Crypto;

/**
 * Andromeda Performance Entry interface
 * Base interface for all performance timeline entries
 */
interface AndromedaPerformanceEntry {
  readonly name: string;
  readonly entryType: string;
  readonly startTime: number;
  readonly duration: number;
}

/**
 * Andromeda Performance Mark interface
 * Represents a named timestamp in the performance timeline
 */
interface AndromedaPerformanceMark extends AndromedaPerformanceEntry {
  readonly entryType: "mark";
  readonly duration: 0;
  readonly detail?: unknown;
}

/**
 * Andromeda Performance Measure interface
 * Represents a time measurement between two marks or timestamps
 */
interface AndromedaPerformanceMeasure extends AndromedaPerformanceEntry {
  readonly entryType: "measure";
  readonly detail?: unknown;
}

/**
 * Andromeda Performance mark options
 */
interface AndromedaPerformanceMarkOptions {
  detail?: unknown;
  startTime?: number;
}

/**
 * Andromeda Performance measure options
 */
interface AndromedaPerformanceMeasureOptions {
  start?: string | number;
  end?: string | number;
  detail?: unknown;
  duration?: number;
}

/**
 * Andromeda Performance interface
 * Provides high-resolution time measurements and performance monitoring
 */
interface AndromedaPerformance {
  /**
   * Returns a high-resolution timestamp in milliseconds
   */
  now(): number;

  /**
   * Returns the time origin (when the performance measurement started)
   */
  readonly timeOrigin: number;

  /**
   * Creates a named timestamp in the performance timeline
   */
  mark(
    markName: string,
    markOptions?: AndromedaPerformanceMarkOptions,
  ): AndromedaPerformanceMark;

  /**
   * Creates a named timestamp between two marks or times
   */
  measure(
    measureName: string,
    startOrMeasureOptions?: string | AndromedaPerformanceMeasureOptions,
    endMark?: string,
  ): AndromedaPerformanceMeasure;

  /**
   * Removes performance marks from the timeline
   */
  clearMarks(markName?: string): void;

  /**
   * Removes performance measures from the timeline
   */
  clearMeasures(measureName?: string): void;

  /**
   * Returns a list of all performance entries
   */
  getEntries(): AndromedaPerformanceEntry[];

  /**
   * Returns a list of performance entries by type
   */
  getEntriesByType(type: string): AndromedaPerformanceEntry[];

  /**
   * Returns a list of performance entries by name
   */
  getEntriesByName(name: string, type?: string): AndromedaPerformanceEntry[];

  /**
   * Converts the Performance object to a JSON representation
   */
  toJSON(): object;
}

/**
 * Global performance instance following the W3C High Resolution Time API
 */
// @ts-ignore Deno type issues
declare const performance: AndromedaPerformance;

/**
 * AbortSignal interface following the WHATWG DOM Standard
 * https://dom.spec.whatwg.org/#interface-abortsignal
 */
// @ts-ignore Deno type issues
interface AbortSignal extends EventTarget {
  /** Returns true if the signal has been aborted */
  readonly aborted: boolean;

  /** Returns the abort reason if the signal has been aborted */
  readonly reason: any;

  /** Throws the abort reason if the signal has been aborted */
  throwIfAborted(): void;

  /** Event handler for 'abort' events */
  onabort: ((this: AbortSignal, ev: Event) => any) | null;
}

interface AbortSignalEventMap {
  "abort": Event;
}

// @ts-ignore Deno type issues
interface AbortSignal {
  addEventListener<K extends keyof AbortSignalEventMap>(
    type: K,
    listener: (this: AbortSignal, ev: AbortSignalEventMap[K]) => any,
    options?: boolean | AddEventListenerOptions,
  ): void;
  addEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject,
    options?: boolean | AddEventListenerOptions,
  ): void;
  removeEventListener<K extends keyof AbortSignalEventMap>(
    type: K,
    listener: (this: AbortSignal, ev: AbortSignalEventMap[K]) => any,
    options?: boolean | EventListenerOptions,
  ): void;
  removeEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject,
    options?: boolean | EventListenerOptions,
  ): void;
}

// @ts-ignore Deno type issues
declare const AbortSignal: {
  prototype: AbortSignal;
  new(): AbortSignal;

  /** Creates an already aborted AbortSignal */
  abort(reason?: any): AbortSignal;

  /** Creates an AbortSignal that will be aborted after the specified timeout */
  timeout(milliseconds: number): AbortSignal;

  /** Creates an AbortSignal that will be aborted when any of the provided signals are aborted */
  any(signals: AbortSignal[]): AbortSignal;
};

/**
 * AbortController interface following the WHATWG DOM Standard
 * https://dom.spec.whatwg.org/#interface-abortcontroller
 */
// @ts-ignore Deno type issues
interface AbortController {
  /** The AbortSignal associated with this controller */
  readonly signal: AbortSignal;

  /** Aborts the associated signal */
  abort(reason?: any): void;
}

// @ts-ignore Deno type issues
declare const AbortController: {
  prototype: AbortController;
  new(): AbortController;
};

/**
 * Options for addEventListener that includes AbortSignal support
 */
interface AddEventListenerOptions extends EventListenerOptions {
  once?: boolean;
  passive?: boolean;
  signal?: AbortSignal;
}

/**
 * Brand information for User-Agent Client Hints
 */
interface UADataValues {
  brand: string;
  version: string;
}

/**
 * High entropy values for User-Agent Client Hints
 */
interface UAHighEntropyValues {
  architecture?: string;
  bitness?: string;
  brands?: UADataValues[];
  fullVersionList?: UADataValues[];
  mobile?: boolean;
  model?: string;
  platform?: string;
  platformVersion?: string;
  wow64?: boolean;
  formFactor?: string;
}

/**
 * NavigatorUAData interface for User-Agent Client Hints
 * https://developer.mozilla.org/en-US/docs/Web/API/NavigatorUAData
 */
interface NavigatorUAData {
  /** Returns an array of brand information containing the browser name and version */
  readonly brands: UADataValues[];

  /** Returns true if the user-agent is running on a mobile device */
  readonly mobile: boolean;

  /** Returns the platform brand the user-agent is running on */
  readonly platform: string;

  /** Returns a Promise that resolves with high entropy values */
  getHighEntropyValues(hints: string[]): Promise<UAHighEntropyValues>;

  /** Returns a JSON representation of the low entropy properties */
  toJSON(): { brands: UADataValues[]; mobile: boolean; platform: string; };
}

/**
 * Navigator interface following the HTML specification
 * https://html.spec.whatwg.org/multipage/system-state.html#the-navigator-object
 */
interface Navigator {
  /** Returns the complete User-Agent header */
  readonly userAgent: string;

  /** Returns the string "Mozilla" for compatibility */
  readonly appCodeName: string;

  /** Returns the string "Netscape" for compatibility */
  readonly appName: string;

  /** Returns the version of the browser */
  readonly appVersion: string;

  /** Returns the name of the platform */
  readonly platform: string;

  /** Returns the string "Gecko" for compatibility */
  readonly product: string;

  /** Returns the product sub-version */
  readonly productSub: string;

  /** Returns the vendor string */
  readonly vendor: string;

  /** Returns the vendor sub-version */
  readonly vendorSub: string;

  /** Returns a NavigatorUAData object for User-Agent Client Hints */
  readonly userAgentData: NavigatorUAData;
}

/**
 * Global clientInformation instance (alias for navigator)
 */
declare const clientInformation: Navigator;

// SQLite Extension Types
// Matching the Deno/Node.js SQLite API

/** Type for SQLite input values (parameters) */
type SQLInputValue = null | number | bigint | string | Uint8Array | boolean;

/** Type for SQLite output values (results) */
type SQLOutputValue = null | number | bigint | string | Uint8Array | boolean;

/** Options for opening a SQLite database */
interface DatabaseSyncOptions {
  readonly open?: boolean;
  readonly readOnly?: boolean;
  readonly allowExtension?: boolean;
  readonly enableForeignKeyConstraints?: boolean;
  readonly enableDoubleQuotedStringLiterals?: boolean;
}

/** Options for creating custom SQLite functions */
interface FunctionOptions {
  readonly varargs?: boolean;
  readonly deterministic?: boolean;
  readonly directOnly?: boolean;
  readonly useBigIntArguments?: boolean;
}

/** Result type for SQLite operations that modify data */
interface StatementResultingChanges {
  readonly changes: number;
  readonly lastInsertRowid: number | bigint;
}

/** Options for applying changesets */
interface ApplyChangesetOptions {
  readonly filter?: (tableName: string) => boolean;
  readonly onConflict?: number;
}

/** Options for creating sessions */
interface CreateSessionOptions {
  readonly db?: string;
  readonly table?: string;
}

/** SQLite session interface */
interface Session {
  changeset(): Uint8Array;
  patchset(): Uint8Array;
  close(): void;
}

/** Type for custom SQLite functions */
type SqliteFunction = (...args: SQLInputValue[]) => SQLOutputValue;

/**
 * SQLite prepared statement class
 * Provides methods for executing SQL statements with parameters
 */
declare class StatementSync {
  constructor(stmtId: number, dbId: number);

  /**
   * Execute the statement and return all results
   * @param params Parameters to bind to the statement
   * @returns Array of result objects
   */
  all(...params: SQLInputValue[]): unknown[];

  /**
   * Get the expanded SQL with parameters substituted
   */
  get expandedSQL(): string;

  /**
   * Execute the statement and return the first result
   * @param params Parameters to bind to the statement
   * @returns First result object or undefined
   */
  get(...params: SQLInputValue[]): unknown;

  /**
   * Execute the statement and return an iterator over results
   * @param params Parameters to bind to the statement
   * @returns Iterator over result objects
   */
  iterate(...params: SQLInputValue[]): IterableIterator<unknown>;

  /**
   * Execute the statement and return change information
   * @param params Parameters to bind to the statement
   * @returns Information about changes made
   */
  run(...params: SQLInputValue[]): StatementResultingChanges;

  /**
   * Set whether to allow bare named parameters
   * @param allowBare Whether to allow bare parameters
   * @returns This statement for chaining
   */
  setAllowBareNamedParameters(allowBare: boolean): this;

  /**
   * Set whether to read big integers
   * @param readBigInts Whether to read as big integers
   * @returns This statement for chaining
   */
  setReadBigInts(readBigInts: boolean): this;

  /**
   * Get the original SQL source
   */
  get sourceSQL(): string;

  /**
   * Finalize the statement and free resources
   */
  finalize(): void;
}

/**
 * SQLite database class
 * Provides methods for managing SQLite databases
 */
declare class DatabaseSync {
  constructor(filename: string, options?: DatabaseSyncOptions);

  /**
   * Apply a changeset to the database
   * @param changeset The changeset to apply
   * @param options Options for applying the changeset
   */
  applyChangeset(changeset: Uint8Array, options?: ApplyChangesetOptions): void;

  /**
   * Close the database connection
   */
  close(): void;

  /**
   * Create a session for tracking changes
   * @param options Options for the session
   * @returns A new session object
   */
  createSession(options?: CreateSessionOptions): Session;

  /**
   * Enable or disable loading extensions
   * @param enabled Whether to enable extension loading
   */
  enableLoadExtension(enabled: boolean): void;

  /**
   * Execute SQL statements without returning results
   * @param sql The SQL to execute
   */
  exec(sql: string): void;

  /**
   * Register a custom function
   * @param name The function name
   * @param fn The function implementation
   * @param options Function options
   */
  function(name: string, fn: SqliteFunction, options?: FunctionOptions): void;

  /**
   * Load an extension
   * @param path Path to the extension
   * @param entryPoint Optional entry point name
   */
  loadExtension(path: string, entryPoint?: string): void;

  /**
   * Open a database file
   * @param filename The database filename
   * @param options Options for opening
   */
  open(filename: string, options?: DatabaseSyncOptions): void;

  /**
   * Prepare a SQL statement
   * @param sql The SQL statement to prepare
   * @returns A prepared statement
   */
  prepare(sql: string): StatementSync;
}

/**
 * Database constructor alias for compatibility
 */
declare const Database: typeof DatabaseSync;

/**
 * SQLite constants matching the Deno API
 */
declare const constants: {
  readonly SQLITE_CHANGESET_ABORT: 2;
  readonly SQLITE_CHANGESET_CONFLICT: 3;
  readonly SQLITE_CHANGESET_DATA: 4;
  readonly SQLITE_CHANGESET_FOREIGN_KEY: 5;
  readonly SQLITE_CHANGESET_NOTFOUND: 1;
  readonly SQLITE_CHANGESET_OMIT: 0;
  readonly SQLITE_CHANGESET_REPLACE: 1;
};

/**
 * SQLite module containing all SQLite functionality
 */
declare const sqlite: {
  DatabaseSync: typeof DatabaseSync;
  StatementSync: typeof StatementSync;
  Database: typeof DatabaseSync;
  constants: typeof constants;
};

/**
 * Queuing strategy interface
 */
interface QueuingStrategy<T = any> {
  highWaterMark?: number;
  size?(chunk: T): number;
}

/**
 * CountQueuingStrategy implementation
 * Measures the size of a chunk as 1
 */
declare class CountQueuingStrategy implements QueuingStrategy {
  readonly highWaterMark: number;
  constructor(init: { highWaterMark: number; });
  size(chunk?: any): number;
}

/**
 * ByteLengthQueuingStrategy implementation
 * Measures the size of a chunk by its byteLength property
 */
declare class ByteLengthQueuingStrategy implements QueuingStrategy {
  readonly highWaterMark: number;
  constructor(init: { highWaterMark: number; });
  size(chunk?: any): number;
}

/**
 * ReadableStream default controller interface
 */
interface ReadableStreamDefaultController<R = any> {
  readonly desiredSize: number | null;
  close(): void;
  enqueue(chunk?: R): void;
  error(e?: any): void;
}

/**
 * ReadableStreamDefaultReader interface
 */
interface ReadableStreamDefaultReader<R = any> {
  readonly closed: Promise<undefined>;
  cancel(reason?: any): Promise<void>;
  read(): Promise<ReadableStreamReadResult<R>>;
  releaseLock(): void;
}

/**
 * ReadableStream read result interface
 */
interface ReadableStreamReadResult<T> {
  done: boolean;
  value: T;
}

/**
 * ReadableStream underlying source interface
 */
interface ReadableStreamUnderlyingSource<R = any> {
  start?(controller: ReadableStreamDefaultController<R>): void | Promise<void>;
  pull?(controller: ReadableStreamDefaultController<R>): void | Promise<void>;
  cancel?(reason?: any): void | Promise<void>;
  type?: undefined;
}

/**
 * Pipe options interface
 */
interface PipeOptions {
  preventClose?: boolean;
  preventAbort?: boolean;
  preventCancel?: boolean;
  signal?: AbortSignal;
}

/**
 * ReadableStream implementation
 */
declare class ReadableStream<R = any> {
  constructor(
    underlyingSource?: ReadableStreamUnderlyingSource<R>,
    strategy?: QueuingStrategy<R>,
  );

  readonly locked: boolean;
  cancel(reason?: any): Promise<void>;
  getReader(): ReadableStreamDefaultReader<R>;
  pipeThrough<T>(
    transform: { readable: ReadableStream<T>; writable: WritableStream<R>; },
    options?: PipeOptions,
  ): ReadableStream<T>;
  pipeTo(destination: WritableStream<R>, options?: PipeOptions): Promise<void>;
  tee(): [ReadableStream<R>, ReadableStream<R>];
  [Symbol.asyncIterator](): AsyncIterator<R>;
}

/**
 * WritableStreamDefaultController interface
 */
interface WritableStreamDefaultController {
  error(e?: any): void;
}

/**
 * WritableStreamDefaultWriter interface
 */
interface WritableStreamDefaultWriter<W = any> {
  readonly closed: Promise<undefined>;
  readonly desiredSize: number | null;
  readonly ready: Promise<undefined>;
  abort(reason?: any): Promise<void>;
  close(): Promise<void>;
  releaseLock(): void;
  write(chunk: W): Promise<void>;
}

/**
 * WritableStream underlying sink interface
 */
interface WritableStreamUnderlyingSink<W = any> {
  start?(controller: WritableStreamDefaultController): void | Promise<void>;
  write?(
    chunk: W,
    controller: WritableStreamDefaultController,
  ): void | Promise<void>;
  close?(): void | Promise<void>;
  abort?(reason?: any): void | Promise<void>;
  type?: undefined;
}

/**
 * WritableStream implementation
 */
declare class WritableStream<W = any> {
  constructor(
    underlyingSink?: WritableStreamUnderlyingSink<W>,
    strategy?: QueuingStrategy<W>,
  );

  readonly locked: boolean;
  abort(reason?: any): Promise<void>;
  close(): Promise<void>;
  getWriter(): WritableStreamDefaultWriter<W>;
}

/**
 * TransformStreamDefaultController interface
 */
interface TransformStreamDefaultController<O = any> {
  readonly desiredSize: number | null;
  enqueue(chunk?: O): void;
  error(reason?: any): void;
  terminate(): void;
}

/**
 * Transformer interface
 */
interface Transformer<I = any, O = any> {
  start?(
    controller: TransformStreamDefaultController<O>,
  ): void | Promise<void>;
  transform?(
    chunk: I,
    controller: TransformStreamDefaultController<O>,
  ): void | Promise<void>;
  flush?(
    controller: TransformStreamDefaultController<O>,
  ): void | Promise<void>;
  readableType?: undefined;
  writableType?: undefined;
}

/**
 * TransformStream implementation
 */
declare class TransformStream<I = any, O = any> {
  constructor(
    transformer?: Transformer<I, O>,
    writableStrategy?: QueuingStrategy<I>,
    readableStrategy?: QueuingStrategy<O>,
  );

  readonly readable: ReadableStream<O>;
  readonly writable: WritableStream<I>;
}
