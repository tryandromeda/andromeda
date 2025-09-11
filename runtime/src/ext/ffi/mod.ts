// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * All plain number types for interfacing with foreign functions.
 */
type NativeNumberType =
  | "u8"
  | "i8"
  | "u16"
  | "i16"
  | "u32"
  | "i32"
  | "f32"
  | "f64";

/**
 * All BigInt number types for interfacing with foreign functions.
 */
type NativeBigIntType =
  | "u64"
  | "i64"
  | "usize"
  | "isize";

/**
 * The native boolean type for interfacing to foreign functions.
 */
type NativeBooleanType = "bool";

/**
 * The native void type for interfacing with foreign functions.
 */
type NativeVoidType = "void";

/**
 * The native pointer type for interfacing to foreign functions.
 */
type NativePointerType = "pointer";

/**
 * The native buffer type for interfacing to foreign functions.
 */
type NativeBufferType = "buffer";

/**
 * The native function type for interfacing with foreign functions.
 */
type NativeFunctionType = {
  function: UnsafeCallbackDefinition;
};

/**
 * The native struct type for interfacing with foreign functions.
 */
type NativeStructType = {
  struct: readonly NativeType[];
};

/**
 * All supported types for interfacing with foreign functions.
 */
type NativeType =
  | NativeNumberType
  | NativeBigIntType
  | NativeBooleanType
  | NativeVoidType
  | NativePointerType
  | NativeBufferType
  | NativeFunctionType
  | NativeStructType;

type NativeResultType = NativeType | NativeVoidType;

/**
 * Type conversion for foreign symbol parameters and unsafe callback return types.
 */
type ToNativeType<T extends NativeType> = T extends NativeNumberType ? number :
  T extends NativeBigIntType ? number | bigint :
  T extends NativeBooleanType ? boolean :
  T extends NativeVoidType ? void :
  T extends NativePointerType ? PointerValue :
  T extends NativeBufferType ? ArrayBufferView | ArrayBuffer :
  T extends NativeFunctionType ? PointerValue :
  T extends NativeStructType ? Uint8Array :
  unknown;

/**
 * Type conversion for foreign symbol return types.
 */
type FromNativeType<T extends NativeType> = T extends NativeNumberType ?
  number :
  T extends NativeBigIntType ? number | bigint :
  T extends NativeBooleanType ? boolean :
  T extends NativeVoidType ? void :
  T extends NativePointerType ? PointerValue :
  T extends NativeBufferType ? Uint8Array :
  T extends NativeFunctionType ? PointerValue :
  T extends NativeStructType ? Uint8Array :
  unknown;

/**
 * A utility type for conversion of parameter types of foreign functions.
 */
type ToNativeParameterTypes<T extends readonly NativeType[]> = {
  [K in keyof T]: ToNativeType<T[K]>;
};

/**
 * Type conversion for foreign symbol return types and unsafe callback parameters.
 */
type FromNativeParameterTypes<T extends readonly NativeType[]> = {
  [K in keyof T]: FromNativeType<T[K]>;
};

/**
 * Type conversion for foreign symbol return types.
 */
type FromNativeResultType<T extends NativeResultType> = FromNativeType<T>;

/**
 * Type conversion for unsafe callback return types.
 */
type ToNativeResultType<T extends NativeResultType> = ToNativeType<T>;

/**
 * A non-null pointer, represented as an object at runtime. The object's prototype
 * is `null` and cannot be changed. The object cannot be assigned to either and is thus
 * entirely read-only.
 */
interface PointerObject {
  [brand]: void;
}

/**
 * Pointers are represented either with a `PointerObject` object or a `null`
 * if the pointer is null.
 */
type PointerValue = PointerObject | null;

/**
 * The interface for a foreign function as defined by its parameter and result
 * types.
 */
interface ForeignFunction {
  /** The name of the foreign function. */
  name?: string;
  /** The parameters of the foreign function. */
  parameters: readonly NativeType[];
  /** The result (return) type of the foreign function. */
  result: NativeResultType;
  /**
   * When `true`, function calls will run on a dedicated thread and return a Promise.
   */
  nonblocking?: boolean;
  /**
   * When `true`, function calls may return before the C function returns and
   * all parameters and return values are discarded.
   */
  callback?: boolean;
  /** When `true`, dlopen will not fail if the symbol is not found. */
  optional?: boolean;
}

/**
 * A foreign library interface descriptor.
 */
interface ForeignLibraryInterface {
  [name: string]: ForeignFunction;
}

/**
 * A dynamic library resource. Use `Andromeda.dlopen` to load a dynamic library and return this interface.
 */
interface DynamicLibrary<T extends ForeignLibraryInterface> {
  /** All of the registered library along with functions for calling them. */
  symbols: StaticForeignLibraryInterface<T>;
  /** Closes the dynamic library. */
  close(): void;
}

/**
 * A utility type that infers a foreign library interface.
 */
type StaticForeignLibraryInterface<T extends ForeignLibraryInterface> = {
  [K in keyof T]: StaticForeignSymbol<T[K]>;
};

/**
 * A utility type that infers a foreign symbol.
 */
type StaticForeignSymbol<T extends ForeignFunction> = T extends {
  optional: true;
} ? T["nonblocking"] extends true ? (
      ...args: ToNativeParameterTypes<T["parameters"]>
    ) => Promise<FromNativeResultType<T["result"]>> | null :
  (
    ...args: ToNativeParameterTypes<T["parameters"]>
  ) => FromNativeResultType<T["result"]> | null :
  T["nonblocking"] extends true ? (
      ...args: ToNativeParameterTypes<T["parameters"]>
    ) => Promise<FromNativeResultType<T["result"]>> :
  (
    ...args: ToNativeParameterTypes<T["parameters"]>
  ) => FromNativeResultType<T["result"]>;

/**
 * Definition of a unsafe callback function.
 */
interface UnsafeCallbackDefinition<
  Parameters extends readonly NativeType[] = readonly NativeType[],
  Result extends NativeResultType = NativeResultType,
> {
  /** The parameters of the callbacks. */
  parameters: Parameters;
  /** The result (return) type of the callback. */
  result: Result;
}

/**
 * An unsafe callback function.
 */
type UnsafeCallbackFunction<
  Parameters extends readonly NativeType[] = readonly NativeType[],
  Result extends NativeResultType = NativeResultType,
> = (
  ...args: FromNativeParameterTypes<Parameters>
) => ToNativeResultType<Result>;

/**
 * An unsafe function pointer for passing JavaScript functions as C function
 * pointers to foreign function calls.
 */
interface UnsafeCallback<
  Definition extends UnsafeCallbackDefinition = UnsafeCallbackDefinition,
> {
  /** The callback function. */
  readonly callback: UnsafeCallbackFunction<
    Definition["parameters"],
    Definition["result"]
  >;

  /** The definition of the callback. */
  readonly definition: Definition;

  /** The pointer to the unsafe callback. */
  readonly pointer: PointerValue;

  /**
   * Adds one to this callback's reference counting and returns a new callback
   * with the same function pointer.
   */
  ref(): UnsafeCallback<Definition>;

  /**
   * Removes one from this callback's reference counting and returns a new
   * callback with the same function pointer.
   */
  unref(): UnsafeCallback<Definition>;

  /**
   * Closes the callback.
   */
  close(): void;
}

/**
 * An unsafe pointer to a function, for calling functions that are not present as
 * symbols.
 */
interface UnsafeFnPointer<
  Definition extends ForeignFunction = ForeignFunction,
> {
  /** The definition of the function. */
  readonly definition: Definition;

  /** The pointer to the function. */
  readonly pointer: PointerValue;

  /** Call the foreign function. */
  readonly call: StaticForeignSymbol<Definition>;
}

/**
 * A collection of static functions for interacting with pointer objects.
 */
interface UnsafePointerStatic {
  /**
   * Create a pointer from a numeric value. This one is *really* dangerous!
   */
  create(value: number | bigint): PointerValue;

  /**
   * Returns `true` if the two pointers point to the same address.
   */
  equals(a: PointerValue, b: PointerValue): boolean;

  /**
   * Return the direct memory pointer to the typed array in memory.
   */
  of(value: Uint8Array | ArrayBufferView): PointerValue;

  /**
   * Return a new pointer offset from the original by the given number of bytes.
   */
  offset(value: PointerValue, offset: number): PointerValue;

  /**
   * Get the numeric value of a pointer
   */
  value(value: PointerValue): number | bigint;
}

/**
 * An unsafe pointer view to a memory location as specified by the `pointer`
 * value. The `UnsafePointerView` API follows the standard built in interface
 * `DataView` for accessing the underlying types at a memory location (numbers,
 * strings and raw bytes).
 */
interface UnsafePointerView {
  /** The pointer to the memory location. */
  readonly pointer: PointerValue;

  /** Get an `ArrayBuffer` of length `byteLength` starting at the pointer. */
  getArrayBuffer(byteLength: number, byteOffset?: number): ArrayBuffer;

  /** Get a boolean at the specified byte offset from the pointer. */
  getBool(byteOffset?: number): boolean;

  /** Get a signed 8-bit integer at the specified byte offset from the pointer. */
  getInt8(byteOffset?: number): number;

  /** Get an unsigned 8-bit integer at the specified byte offset from the pointer. */
  getUint8(byteOffset?: number): number;

  /** Get a signed 16-bit integer at the specified byte offset from the pointer. */
  getInt16(byteOffset?: number, littleEndian?: boolean): number;

  /** Get an unsigned 16-bit integer at the specified byte offset from the pointer. */
  getUint16(byteOffset?: number, littleEndian?: boolean): number;

  /** Get a signed 32-bit integer at the specified byte offset from the pointer. */
  getInt32(byteOffset?: number, littleEndian?: boolean): number;

  /** Get an unsigned 32-bit integer at the specified byte offset from the pointer. */
  getUint32(byteOffset?: number, littleEndian?: boolean): number;

  /** Get a signed 64-bit integer at the specified byte offset from the pointer. */
  getBigInt64(byteOffset?: number, littleEndian?: boolean): bigint;

  /** Get an unsigned 64-bit integer at the specified byte offset from the pointer. */
  getBigUint64(byteOffset?: number, littleEndian?: boolean): bigint;

  /** Get a 32-bit float at the specified byte offset from the pointer. */
  getFloat32(byteOffset?: number, littleEndian?: boolean): number;

  /** Get a 64-bit float at the specified byte offset from the pointer. */
  getFloat64(byteOffset?: number, littleEndian?: boolean): number;

  /** Get a pointer at the specified byte offset from the pointer */
  getPointer(byteOffset?: number): PointerValue;

  /**
   * Get a null-terminated C string at the specified byte offset from the pointer.
   */
  getCString(byteOffset?: number): string;

  /**
   * Copy bytes from the pointer into a Uint8Array.
   */
  copyInto(destination: Uint8Array, byteOffset?: number): number;
}

const brand = Symbol("pointer");

class UnsafeCallback<
  Definition extends UnsafeCallbackDefinition = UnsafeCallbackDefinition,
> {
  #id: number;
  #definition: Definition;
  #callback: UnsafeCallbackFunction<
    Definition["parameters"],
    Definition["result"]
  >;
  #pointer: PointerValue;

  constructor(
    definition: Definition,
    callback: UnsafeCallbackFunction<
      Definition["parameters"],
      Definition["result"]
    >,
  ) {
    this.#definition = definition;
    this.#callback = callback;
    this.#id = __andromeda__.ffi_create_callback(definition, callback);

    // Get the callback pointer from the stored callback
    const pointerValue = __andromeda__.ffi_get_callback_pointer(this.#id);
    this.#pointer = UnsafePointer.create(pointerValue);
  }

  get callback() {
    return this.#callback;
  }

  get definition() {
    return this.#definition;
  }

  get pointer() {
    return this.#pointer;
  }

  ref(): UnsafeCallback<Definition> {
    return new UnsafeCallback(
      this.#definition,
      this.#callback,
    ) as UnsafeCallback<Definition>;
  }

  unref(): UnsafeCallback<Definition> {
    return new UnsafeCallback(
      this.#definition,
      this.#callback,
    ) as UnsafeCallback<Definition>;
  }

  close(): void {
    __andromeda__.ffi_callback_close(this.#id);
  }
}

class UnsafeFnPointer<
  Definition extends ForeignFunction = ForeignFunction,
> {
  #pointer: PointerValue;
  #definition: Definition;

  constructor(pointer: PointerValue, definition: Definition) {
    this.#pointer = pointer;
    this.#definition = definition;
  }

  get definition() {
    return this.#definition;
  }

  get pointer() {
    return this.#pointer;
  }

  get call(): StaticForeignSymbol<Definition> {
    return ((..._args: unknown[]) => {
      // TODO: Implement function pointer calling
      return null;
    }) as StaticForeignSymbol<Definition>;
  }
}

class UnsafePointer {
  static create(value: number | bigint): PointerValue {
    const result = __andromeda__.ffi_pointer_create(Number(value));
    return result as PointerValue;
  }

  static equals(a: PointerValue, b: PointerValue): boolean {
    return __andromeda__.ffi_pointer_equals(a, b);
  }

  static of(value: Uint8Array | ArrayBufferView): PointerValue {
    return __andromeda__.ffi_pointer_of(value) as PointerValue;
  }

  static offset(value: PointerValue, offset: number): PointerValue {
    const result = __andromeda__.ffi_pointer_offset(value, offset);
    return result as PointerValue;
  }

  static value(value: PointerValue): number | bigint {
    return __andromeda__.ffi_pointer_value(value) as number | bigint;
  }
}

class UnsafePointerView {
  #pointer: PointerValue;

  constructor(pointer: PointerValue) {
    this.#pointer = pointer;
  }

  get pointer() {
    return this.#pointer;
  }

  getArrayBuffer(byteLength: number, byteOffset = 0): ArrayBuffer {
    return __andromeda__.ffi_read_memory(
      this.#pointer,
      byteOffset,
      byteLength,
    ) as ArrayBuffer;
  }

  getBool(byteOffset = 0): boolean {
    const buffer = this.getArrayBuffer(1, byteOffset);
    return new Uint8Array(buffer)[0] !== 0;
  }

  getInt8(byteOffset = 0): number {
    const buffer = this.getArrayBuffer(1, byteOffset);
    return new Int8Array(buffer)[0];
  }

  getUint8(byteOffset = 0): number {
    const buffer = this.getArrayBuffer(1, byteOffset);
    return new Uint8Array(buffer)[0];
  }

  getInt16(byteOffset = 0, littleEndian = true): number {
    const buffer = this.getArrayBuffer(2, byteOffset);
    return new DataView(buffer).getInt16(0, littleEndian);
  }

  getUint16(byteOffset = 0, littleEndian = true): number {
    const buffer = this.getArrayBuffer(2, byteOffset);
    return new DataView(buffer).getUint16(0, littleEndian);
  }

  getInt32(byteOffset = 0, littleEndian = true): number {
    const buffer = this.getArrayBuffer(4, byteOffset);
    return new DataView(buffer).getInt32(0, littleEndian);
  }

  getUint32(byteOffset = 0, littleEndian = true): number {
    const buffer = this.getArrayBuffer(4, byteOffset);
    return new DataView(buffer).getUint32(0, littleEndian);
  }

  getBigInt64(byteOffset = 0, littleEndian = true): bigint {
    const buffer = this.getArrayBuffer(8, byteOffset);
    return new DataView(buffer).getBigInt64(0, littleEndian);
  }

  getBigUint64(byteOffset = 0, littleEndian = true): bigint {
    const buffer = this.getArrayBuffer(8, byteOffset);
    return new DataView(buffer).getBigUint64(0, littleEndian);
  }

  getFloat32(byteOffset = 0, littleEndian = true): number {
    const buffer = this.getArrayBuffer(4, byteOffset);
    return new DataView(buffer).getFloat32(0, littleEndian);
  }

  getFloat64(byteOffset = 0, littleEndian = true): number {
    const buffer = this.getArrayBuffer(8, byteOffset);
    return new DataView(buffer).getFloat64(0, littleEndian);
  }

  getPointer(byteOffset = 0): PointerValue {
    const buffer = this.getArrayBuffer(8, byteOffset); // Assuming 64-bit pointers
    const value = new DataView(buffer).getBigUint64(0, true);
    return UnsafePointer.create(value);
  }

  getCString(byteOffset = 0): string {
    // TODO: Implement proper C string reading
    const buffer = this.getArrayBuffer(256, byteOffset); // Read up to 256 bytes
    const bytes = new Uint8Array(buffer);
    const nullIndex = bytes.indexOf(0);
    const actualBytes = nullIndex >= 0 ?
      bytes.subarray(0, nullIndex) :
      bytes;
    return new TextDecoder().decode(actualBytes);
  }

  copyInto(destination: Uint8Array, byteOffset = 0): number {
    const buffer = this.getArrayBuffer(destination.length, byteOffset);
    const sourceBytes = new Uint8Array(buffer);
    destination.set(sourceBytes);
    return sourceBytes.length;
  }
}

class DynamicLibrary<T extends ForeignLibraryInterface>
  implements DynamicLibrary<T>
{
  #id: number;
  #symbols: StaticForeignLibraryInterface<T>;

  constructor(id: number, symbols: StaticForeignLibraryInterface<T>) {
    this.#id = id;
    this.#symbols = symbols;
  }

  get symbols() {
    return this.#symbols;
  }

  close() {
    __andromeda__.ffi_dlclose(this.#id);
  }
}

function dlopen<T extends ForeignLibraryInterface>(
  filename: string | URL,
  symbols: T,
): DynamicLibrary<T> {
  const path = typeof filename === "string" ? filename : filename.pathname;
  const libId = __andromeda__.ffi_dlopen(path, symbols);

  const callableSymbols = {} as StaticForeignLibraryInterface<T>;

  for (const [name, def] of Object.entries(symbols)) {
    const _symbolPointer = __andromeda__.ffi_dlopen_get_symbol(
      libId,
      name,
      def,
    );

    callableSymbols[name as keyof T] = ((...args: unknown[]) => {
      return __andromeda__.ffi_call_symbol(libId, name, args);
    }) as StaticForeignSymbol<T[keyof T]>;
  }

  return new DynamicLibrary(libId, callableSymbols);
}

// @ts-ignore globalThis is not readonly
globalThis.dlopen = dlopen;
// @ts-ignore globalThis is not readonly
globalThis.UnsafeCallback = UnsafeCallback;
// @ts-ignore globalThis is not readonly
globalThis.UnsafeFnPointer = UnsafeFnPointer;
// @ts-ignore globalThis is not readonly
globalThis.UnsafePointer = UnsafePointer;
// @ts-ignore globalThis is not readonly
globalThis.UnsafePointerView = UnsafePointerView;
