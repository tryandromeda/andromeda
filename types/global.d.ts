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

  // Command API

  /**
   * Options for creating a new Command.
   */
  interface CommandOptions {
    /** Arguments to pass to the command. */
    args?: string[];
    /** Working directory for the command. If not specified, the cwd of the parent process is used. */
    cwd?: string;
    /** Environment variables to pass to the subprocess. */
    env?: Record<string, string>;
    /** Clear environmental variables from parent process. */
    clearEnv?: boolean;
    /** How stdin of the spawned process should be handled. Defaults to "null" for output/outputSync and "inherit" for spawn. */
    stdin?: "piped" | "inherit" | "null";
    /** How stdout of the spawned process should be handled. Defaults to "piped". */
    stdout?: "piped" | "inherit" | "null";
    /** How stderr of the spawned process should be handled. Defaults to "piped". */
    stderr?: "piped" | "inherit" | "null";
    /** Sets the child process's user ID (Unix only). */
    uid?: number;
    /** Sets the child process's group ID (Unix only). */
    gid?: number;
    /** Skips quoting and escaping of the arguments on Windows. Ignored on non-Windows platforms. */
    windowsRawArguments?: boolean;
  }

  /**
   * The output of a completed command.
   */
  interface CommandOutput {
    /** Whether the command exited successfully (exit code 0). */
    success: boolean;
    /** The exit code of the command. */
    code: number;
    /** The signal associated with the child process, or null. */
    signal: Signal | null;
    /** The standard output of the command. */
    stdout: string;
    /** The standard error output of the command. */
    stderr: string;
  }

  /**
   * The exit status of a completed command.
   */
  interface CommandStatus {
    /** Whether the command exited successfully (exit code 0). */
    success: boolean;
    /** The exit code of the command. */
    code: number;
    /** The signal associated with the child process, or null. */
    signal: Signal | null;
  }

  /**
   * Represents a spawned child process.
   */
  interface ChildProcess {
    /** The operating system process ID of the child process. */
    readonly pid: number;
    /** A promise that resolves with the exit status when the process completes. */
    readonly status: Promise<CommandStatus>;
    /** Kills the child process with the given signal. Defaults to "SIGTERM". */
    kill(signo?: Signal): void;
    /** Waits for the child to exit completely, returning all its output and status. */
    output(): Promise<CommandOutput>;
  }

  /**
   * A Command is used to configure and spawn subprocesses.
   *
   * @example Run a command and get its output
   * ```ts
   * const command = new Andromeda.Command("echo", { args: ["hello", "world"] });
   * const output = await command.output();
   * console.log(output.stdout); // "hello world\n"
   * ```
   *
   * @example Run a command synchronously
   * ```ts
   * const command = new Andromeda.Command("ls", { args: ["-la"] });
   * const output = command.outputSync();
   * console.log(output.code); // 0
   * ```
   *
   * @example Spawn a long-running process
   * ```ts
   * const command = new Andromeda.Command("sleep", { args: ["10"] });
   * const child = command.spawn();
   * child.kill();
   * ```
   *
   * @example Custom environment and stderr handling
   * ```ts
   * const cmd = new Andromeda.Command("node", {
   *   args: ["-e", "console.error('hi')"],
   *   env: { NODE_ENV: "production" },
   *   stderr: "piped",
   * });
   * const output = await cmd.output();
   * console.log(output.stderr);
   * ```
   */
  class Command {
    constructor(program: string, options?: CommandOptions);
    /** Runs the command and waits for it to complete, returning its output. */
    output(): Promise<CommandOutput>;
    /** Synchronously runs the command and returns its output. */
    outputSync(): CommandOutput;
    /** Spawns the command as a child process without waiting for completion. */
    spawn(): ChildProcess;
  }
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
type CanvasFillRule = "nonzero" | "evenodd";
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
type CanvasPatternRepetition =
  | "repeat"
  | "repeat-x"
  | "repeat-y"
  | "no-repeat";

declare class OffscreenCanvas {
  /**
   * Create a new off-screen canvas with the given dimensions.
   */
  constructor(width: number, height: number);
  /** Internal resource id. Used by `Window.presentCanvas`. */
  readonly rid: number;
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
  /** Encode the canvas to image bytes. Supports "image/png" (default) and "image/jpeg". */
  toBuffer(type?: "image/png" | "image/jpeg", quality?: number): Uint8Array;
  /** Encode the canvas as a `data:` URL. Supports "image/png" (default) and "image/jpeg". */
  toDataURL(type?: string, quality?: number): string;
  /** Encode the canvas as a Blob. */
  convertToBlob(
    options?: { type?: string; quality?: number },
  ): Promise<Blob>;
}

/**
 * The 2D rendering context for a Canvas.
 */
declare class CanvasRenderingContext2D {
  /**
   * Gets or sets the current fill style for drawing operations.
   * Accepts a CSS color string, a `CanvasGradient`, or a `CanvasPattern`.
   */
  fillStyle: string | CanvasGradient | CanvasPattern;
  /**
   * Gets or sets the current stroke style for drawing operations.
   * Accepts a CSS color string, a `CanvasGradient`, or a `CanvasPattern`.
   */
  strokeStyle: string | CanvasGradient | CanvasPattern;
  /** Gets or sets the line width for drawing operations. Default 1. */
  lineWidth: number;
  /** Gets or sets the line cap style. Default "butt". */
  lineCap: CanvasLineCap;
  /** Gets or sets the line join style. Default "miter". */
  lineJoin: CanvasLineJoin;
  /** Gets or sets the miter limit. Default 10. */
  miterLimit: number;
  /** Gets or sets the line dash offset. Default 0. */
  lineDashOffset: number;
  /** Gets or sets image smoothing. Default true. */
  imageSmoothingEnabled: boolean;
  /** Gets or sets image smoothing quality. Default "low". */
  imageSmoothingQuality: ImageSmoothingQuality;
  /** Gets or sets a CSS filter string. Default "none" (stored but not applied). */
  filter: string;
  /**
   * Gets or sets the global alpha value (transparency) for drawing
   * operations. Values range from 0.0 (transparent) to 1.0 (opaque).
   */
  globalAlpha: number;
  /**
   * Gets or sets the type of compositing operation to apply when drawing
   * new shapes. Valid values: `source-over`, `source-in`, `source-out`,
   * `source-atop`, `destination-over`, `destination-in`, `destination-out`,
   * `destination-atop`, `lighter`, `copy`, `xor`, `multiply`, `screen`,
   * `overlay`, `darken`, `lighten`, `color-dodge`, `color-burn`,
   * `hard-light`, `soft-light`, `difference`, `exclusion`, `hue`,
   * `saturation`, `color`, `luminosity`. Default `source-over`.
   */
  globalCompositeOperation: string;

  // Shadow properties
  /** Gets or sets the blur radius used for shadows. Default 0. */
  shadowBlur: number;
  /** Gets or sets the shadow color. Default `rgba(0, 0, 0, 0)`. */
  shadowColor: string;
  /** Gets or sets the horizontal shadow offset. Default 0. */
  shadowOffsetX: number;
  /** Gets or sets the vertical shadow offset. Default 0. */
  shadowOffsetY: number;

  // Text properties
  /** Gets or sets the current font, e.g. `"18px sans-serif"`. Default `"10px sans-serif"`. */
  font: string;
  /** Gets or sets text alignment. Default `"start"`. */
  textAlign: CanvasTextAlign;
  /** Gets or sets the text baseline. Default `"alphabetic"`. */
  textBaseline: CanvasTextBaseline;
  /** Gets or sets the text direction. Default `"inherit"`. */
  direction: CanvasDirection;

  /** Resets the context to its default state per HTML spec. */
  reset(): void;
  /** Returns false — Andromeda never loses the canvas context. */
  isContextLost(): boolean;
  /** Sets the line dash pattern. */
  setLineDash(segments: number[]): void;
  /** Returns the current line dash pattern. */
  getLineDash(): number[];

  // Path construction
  /** Creates an arc on the current path. */
  arc(
    x: number,
    y: number,
    radius: number,
    startAngle: number,
    endAngle: number,
    counterclockwise?: boolean,
  ): void;
  /** Adds an arc-to segment to the current path. */
  arcTo(x1: number, y1: number, x2: number, y2: number, radius: number): void;
  /** Begins a new path. */
  beginPath(): void;
  /** Adds a cubic Bézier curve to the current path. */
  bezierCurveTo(
    cp1x: number,
    cp1y: number,
    cp2x: number,
    cp2y: number,
    x: number,
    y: number,
  ): void;
  /** Closes the current path. */
  closePath(): void;
  /** Moves the path starting point. */
  moveTo(x: number, y: number): void;
  /** Adds a straight line from the last point to (x, y). */
  lineTo(x: number, y: number): void;
  /** Adds a rectangle sub-path. */
  rect(x: number, y: number, width: number, height: number): void;
  /** Adds a quadratic Bézier curve to the current path. */
  quadraticCurveTo(cpx: number, cpy: number, x: number, y: number): void;
  /** Adds an ellipse sub-path. */
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
  /** Adds a rounded rectangle sub-path. */
  roundRect(
    x: number,
    y: number,
    w: number,
    h: number,
    radii?:
      | number
      | { x: number; y: number }
      | Array<number | { x: number; y: number }>,
  ): void;

  // Rectangles
  /** Clears the specified rectangular area. */
  clearRect(x: number, y: number, width: number, height: number): void;
  /** Draws a filled rectangle with the current fill style. */
  fillRect(x: number, y: number, width: number, height: number): void;
  /** Draws a stroked rectangle with the current stroke style. */
  strokeRect(x: number, y: number, width: number, height: number): void;

  // Fill / stroke / clip
  /** Fills the current path (or given `Path2D`) with the current fill style. */
  fill(fillRule?: CanvasFillRule): void;
  fill(path: Path2D, fillRule?: CanvasFillRule): void;
  /** Strokes the current path (or given `Path2D`) with the current stroke style. */
  stroke(path?: Path2D): void;
  /** Turns the current path (or given `Path2D`) into the clipping region. */
  clip(fillRule?: CanvasFillRule): void;
  clip(path: Path2D, fillRule?: CanvasFillRule): void;

  // Hit testing
  /** Returns whether the given point is inside the current path. */
  isPointInPath(x: number, y: number, fillRule?: CanvasFillRule): boolean;
  isPointInPath(
    path: Path2D,
    x: number,
    y: number,
    fillRule?: CanvasFillRule,
  ): boolean;
  /** Returns whether the given point is inside the stroked path. */
  isPointInStroke(x: number, y: number): boolean;
  isPointInStroke(path: Path2D, x: number, y: number): boolean;

  // Gradients & patterns
  /** Creates a linear gradient along the line between two points. */
  createLinearGradient(
    x0: number,
    y0: number,
    x1: number,
    y1: number,
  ): CanvasGradient;
  /** Creates a radial gradient between two circles. */
  createRadialGradient(
    x0: number,
    y0: number,
    r0: number,
    x1: number,
    y1: number,
    r1: number,
  ): CanvasGradient;
  /** Creates a conic gradient around a point. */
  createConicGradient(
    startAngle: number,
    x: number,
    y: number,
  ): CanvasGradient;
  /** Creates a pattern from an image with the given repetition mode. */
  createPattern(
    image: ImageBitmap,
    repetition: CanvasPatternRepetition,
  ): CanvasPattern | null;

  // State
  /** Saves the current canvas state to a stack. */
  save(): void;
  /** Restores the most recently saved canvas state from the stack. */
  restore(): void;

  // Transforms
  /** Adds a rotation (in radians) to the current transform. */
  rotate(angle: number): void;
  /** Adds a scaling transformation. */
  scale(x: number, y: number): void;
  /** Adds a translation. */
  translate(x: number, y: number): void;
  /** Multiplies the current transform by the given 2×3 matrix. */
  transform(
    a: number,
    b: number,
    c: number,
    d: number,
    e: number,
    f: number,
  ): void;
  /** Resets the current transform, then applies the given matrix. */
  setTransform(
    a: number,
    b: number,
    c: number,
    d: number,
    e: number,
    f: number,
  ): void;
  setTransform(matrix?: DOMMatrix2DInit | DOMMatrixReadOnly | null): void;
  /** Resets the current transform to identity. */
  resetTransform(): void;
  /** Returns the current transformation matrix. */
  getTransform(): DOMMatrix;

  // Text
  /** Returns a `TextMetrics` object for the given text in the current font. */
  measureText(text: string): TextMetrics;
  /** Draws filled text at (x, y). */
  fillText(text: string, x: number, y: number, maxWidth?: number): void;
  /** Draws stroked (outlined) text at (x, y). */
  strokeText(text: string, x: number, y: number, maxWidth?: number): void;

  // Image drawing
  /** Draws an image at (dx, dy). */
  drawImage(image: ImageBitmap, dx: number, dy: number): void;
  /** Draws an image at (dx, dy) scaled to (dWidth, dHeight). */
  drawImage(
    image: ImageBitmap,
    dx: number,
    dy: number,
    dWidth: number,
    dHeight: number,
  ): void;
  /** Draws a sub-rect of an image at (dx, dy) scaled to (dWidth, dHeight). */
  drawImage(
    image: ImageBitmap,
    sx: number,
    sy: number,
    sWidth: number,
    sHeight: number,
    dx: number,
    dy: number,
    dWidth: number,
    dHeight: number,
  ): void;

  // ImageData
  /** Creates a new blank `ImageData` with the given dimensions. */
  createImageData(width: number, height: number): ImageData;
  /** Creates a new `ImageData` matching the dimensions of another. */
  createImageData(imageData: ImageData): ImageData;
  /** Returns pixel data for the given rectangle. */
  getImageData(sx: number, sy: number, sw: number, sh: number): ImageData;
  /** Paints pixel data at (dx, dy). */
  putImageData(imageData: ImageData, dx: number, dy: number): void;
  /** Paints a dirty sub-rect of pixel data at (dx, dy). */
  putImageData(
    imageData: ImageData,
    dx: number,
    dy: number,
    dirtyX: number,
    dirtyY: number,
    dirtyWidth: number,
    dirtyHeight: number,
  ): void;
}

/**
 * A reusable path description. Can be constructed from another `Path2D`,
 * an SVG path string (`"M10 10 L20 20"`), or empty.
 */
declare class Path2D {
  constructor(path?: Path2D | string);
  addPath(path: Path2D, transform?: DOMMatrix2DInit | DOMMatrixReadOnly): void;
  moveTo(x: number, y: number): void;
  lineTo(x: number, y: number): void;
  closePath(): void;
  arc(
    x: number,
    y: number,
    radius: number,
    startAngle: number,
    endAngle: number,
    counterclockwise?: boolean,
  ): void;
  arcTo(x1: number, y1: number, x2: number, y2: number, radius: number): void;
  bezierCurveTo(
    cp1x: number,
    cp1y: number,
    cp2x: number,
    cp2y: number,
    x: number,
    y: number,
  ): void;
  quadraticCurveTo(cpx: number, cpy: number, x: number, y: number): void;
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
  rect(x: number, y: number, w: number, h: number): void;
  roundRect(
    x: number,
    y: number,
    w: number,
    h: number,
    radii?:
      | number
      | { x: number; y: number }
      | Array<number | { x: number; y: number }>,
  ): void;
  isPointInPath(x: number, y: number, fillRule?: CanvasFillRule): boolean;
  isPointInStroke(x: number, y: number, lineWidth?: number): boolean;
}

declare class CanvasGradient {
  /** Adds a new color stop. `offset` must be in [0, 1]. */
  addColorStop(offset: number, color: string): void;
}

/**
 * A pattern created from an `ImageBitmap` with a given repetition mode.
 */
declare class CanvasPattern {
  /** Sets the transformation matrix applied when rendering this pattern. */
  setTransform(transform?: DOMMatrix2DInit | DOMMatrixReadOnly | null): void;
}

/**
 * Dimensions of measured text returned by `measureText`.
 */
declare class TextMetrics {
  /** Pen advance width of the text. */
  readonly width: number;
  readonly actualBoundingBoxLeft: number;
  readonly actualBoundingBoxRight: number;
  /** Font ascent (em-ascent) above the alphabetic baseline, positive. */
  readonly fontBoundingBoxAscent: number;
  /** Font descent (em-descent) below the alphabetic baseline, positive. */
  readonly fontBoundingBoxDescent: number;
  readonly actualBoundingBoxAscent: number;
  readonly actualBoundingBoxDescent: number;
  readonly emHeightAscent: number;
  readonly emHeightDescent: number;
  /** Hanging baseline offset above alphabetic baseline. */
  readonly hangingBaseline: number;
  /** Alphabetic baseline offset (always 0). */
  readonly alphabeticBaseline: number;
  /** Ideographic baseline offset below alphabetic baseline (negative). */
  readonly ideographicBaseline: number;
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
 * ImageData represents the underlying pixel data of a canvas area.
 */
declare class ImageData {
  /** Creates a new blank `ImageData` with the given dimensions. */
  constructor(width: number, height: number);
  /** Creates an `ImageData` wrapping an existing `Uint8ClampedArray`. */
  constructor(data: Uint8ClampedArray, width: number, height?: number);

  /** The width of the ImageData in pixels. */
  readonly width: number;
  /** The height of the ImageData in pixels. */
  readonly height: number;
  /** The one-dimensional array containing the pixel data in RGBA order. */
  readonly data: Uint8ClampedArray;
}

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

declare class DOMMatrixReadOnly {
  constructor(init?: number[] | DOMMatrix2DInit | string);
  readonly a: number;
  readonly b: number;
  readonly c: number;
  readonly d: number;
  readonly e: number;
  readonly f: number;
  readonly m11: number;
  readonly m12: number;
  readonly m21: number;
  readonly m22: number;
  readonly m41: number;
  readonly m42: number;
  readonly is2D: boolean;
  readonly isIdentity: boolean;
  multiply(other: DOMMatrix2DInit | DOMMatrixReadOnly): DOMMatrix;
  translate(tx: number, ty?: number): DOMMatrix;
  scale(sx: number, sy?: number): DOMMatrix;
  rotate(angleDegrees: number): DOMMatrix;
  inverse(): DOMMatrix;
  transformPoint(point: { x: number; y: number }): { x: number; y: number };
  toFloat32Array(): Float32Array;
  toString(): string;
}

declare class DOMMatrix extends DOMMatrixReadOnly {
  constructor(init?: number[] | DOMMatrix2DInit | string);
  a: number;
  b: number;
  c: number;
  d: number;
  e: number;
  f: number;
  multiplySelf(other: DOMMatrix2DInit | DOMMatrixReadOnly): DOMMatrix;
  translateSelf(tx: number, ty?: number): DOMMatrix;
  scaleSelf(sx: number, sy?: number): DOMMatrix;
  rotateSelf(angleDegrees: number): DOMMatrix;
  invertSelf(): DOMMatrix;
}

/**
 * Creates an ImageBitmap from a file path or URL.
 * @param path The file path or URL to load.
 */
declare function createImageBitmap(path: string): ImageBitmap;

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
