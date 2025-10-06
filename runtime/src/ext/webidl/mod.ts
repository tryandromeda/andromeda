// deno-lint-ignore-file no-unused-vars

function toNumber(value: unknown): number {
  if (typeof value === "bigint") {
    throw new TypeError("Cannot convert a BigInt value to a number");
  }
  return Number(value);
}

function type(V: unknown): string {
  if (V === null) {
    return "Null";
  }
  switch (typeof V) {
    case "undefined":
      return "Undefined";
    case "boolean":
      return "Boolean";
    case "number":
      return "Number";
    case "string":
      return "String";
    case "symbol":
      return "Symbol";
    case "bigint":
      return "BigInt";
    case "object":
    case "function":
    default:
      return "Object";
  }
}

function makeException(
  ErrorType: new(message: string) => Error,
  message: string,
  prefix?: string,
  context?: string,
): Error {
  return new ErrorType(
    `${prefix ? prefix + ": " : ""}${context ? context : "Value"} ${message}`,
  );
}

// Round x to the nearest integer, choosing the even integer if it lies halfway between two.
function evenRound(x: number): number {
  if (
    (x > 0 && x % 1 === +0.5 && (x & 1) === 0) ||
    (x < 0 && x % 1 === -0.5 && (x & 1) === 1)
  ) {
    return censorNegativeZero(Math.floor(x));
  }
  return censorNegativeZero(Math.round(x));
}

function integerPart(n: number): number {
  return censorNegativeZero(Math.trunc(n));
}

function sign(x: number): number {
  return x < 0 ? -1 : 1;
}

function modulo(x: number, y: number): number {
  const signMightNotMatch = x % y;
  if (sign(y) !== sign(signMightNotMatch)) {
    return signMightNotMatch + y;
  }
  return signMightNotMatch;
}

function censorNegativeZero(x: number): number {
  return x === 0 ? 0 : x;
}

interface ConversionOptions {
  enforceRange?: boolean;
  clamp?: boolean;
  treatNullAsEmptyString?: boolean;
  allowShared?: boolean;
}

function createIntegerConversion(
  bitLength: number,
  typeOpts: { unsigned: boolean; },
) {
  const isSigned = !typeOpts.unsigned;

  let lowerBound: number;
  let upperBound: number;
  if (bitLength === 64) {
    upperBound = Number.MAX_SAFE_INTEGER;
    lowerBound = !isSigned ? 0 : Number.MIN_SAFE_INTEGER;
  } else if (!isSigned) {
    lowerBound = 0;
    upperBound = Math.pow(2, bitLength) - 1;
  } else {
    lowerBound = -Math.pow(2, bitLength - 1);
    upperBound = Math.pow(2, bitLength - 1) - 1;
  }

  const twoToTheBitLength = Math.pow(2, bitLength);
  const twoToOneLessThanTheBitLength = Math.pow(2, bitLength - 1);

  return (
    V: unknown,
    prefix?: string,
    context?: string,
    opts: ConversionOptions = {},
  ): number => {
    let x = toNumber(V);
    x = censorNegativeZero(x);

    if (opts.enforceRange) {
      if (!Number.isFinite(x)) {
        throw makeException(
          TypeError,
          "is not a finite number",
          prefix,
          context,
        );
      }

      x = integerPart(x);

      if (x < lowerBound || x > upperBound) {
        throw makeException(
          TypeError,
          `is outside the accepted range of ${lowerBound} to ${upperBound}, inclusive`,
          prefix,
          context,
        );
      }

      return x;
    }

    if (!Number.isNaN(x) && opts.clamp) {
      x = Math.min(Math.max(x, lowerBound), upperBound);
      x = evenRound(x);
      return x;
    }

    if (!Number.isFinite(x) || x === 0) {
      return 0;
    }
    x = integerPart(x);

    if (x >= lowerBound && x <= upperBound) {
      return x;
    }

    x = modulo(x, twoToTheBitLength);
    if (isSigned && x >= twoToOneLessThanTheBitLength) {
      return x - twoToTheBitLength;
    }
    return x;
  };
}

function isByteString(input: string): boolean {
  for (let i = 0; i < input.length; i++) {
    if (input.charCodeAt(i) > 255) {
      return false;
    }
  }
  return true;
}

// Type converters object
// Used by other WebIDL-based modules for type conversion
const converters = {
  any: (V: unknown) => V,

  boolean: (val: unknown): boolean => !!val,

  byte: createIntegerConversion(8, { unsigned: false }),
  octet: createIntegerConversion(8, { unsigned: true }),

  short: createIntegerConversion(16, { unsigned: false }),
  "unsigned short": createIntegerConversion(16, { unsigned: true }),

  long: createIntegerConversion(32, { unsigned: false }),
  "unsigned long": createIntegerConversion(32, { unsigned: true }),

  "long long": createIntegerConversion(64, { unsigned: false }),
  "unsigned long long": createIntegerConversion(64, { unsigned: true }),

  float: (V: unknown, prefix?: string, context?: string): number => {
    const x = toNumber(V);

    if (!Number.isFinite(x)) {
      throw makeException(
        TypeError,
        "is not a finite floating-point value",
        prefix,
        context,
      );
    }

    if (Object.is(x, -0)) {
      return x;
    }

    const y = Math.fround(x);

    if (!Number.isFinite(y)) {
      throw makeException(
        TypeError,
        "is outside the range of a single-precision floating-point value",
        prefix,
        context,
      );
    }

    return y;
  },

  "unrestricted float": (V: unknown): number => {
    const x = toNumber(V);

    if (Number.isNaN(x)) {
      return x;
    }

    if (Object.is(x, -0)) {
      return x;
    }

    return Math.fround(x);
  },

  double: (V: unknown, prefix?: string, context?: string): number => {
    const x = toNumber(V);

    if (!Number.isFinite(x)) {
      throw makeException(
        TypeError,
        "is not a finite floating-point value",
        prefix,
        context,
      );
    }

    return x;
  },

  "unrestricted double": (V: unknown): number => {
    const x = toNumber(V);
    return x;
  },

  DOMString: (
    V: unknown,
    prefix?: string,
    context?: string,
    opts: ConversionOptions = {},
  ): string => {
    if (typeof V === "string") {
      return V;
    } else if (V === null && opts.treatNullAsEmptyString) {
      return "";
    } else if (typeof V === "symbol") {
      throw makeException(
        TypeError,
        "is a symbol, which cannot be converted to a string",
        prefix,
        context,
      );
    }

    return String(V);
  },

  ByteString: (
    V: unknown,
    prefix?: string,
    context?: string,
    opts?: ConversionOptions,
  ): string => {
    const x = converters.DOMString(V, prefix, context, opts);
    if (!isByteString(x)) {
      throw makeException(
        TypeError,
        "is not a valid ByteString",
        prefix,
        context,
      );
    }
    return x;
  },

  USVString: (
    V: unknown,
    prefix?: string,
    context?: string,
    opts?: ConversionOptions,
  ): string => {
    const S = converters.DOMString(V, prefix, context, opts);
    return S.toWellFormed?.() || S;
  },

  object: (V: unknown, prefix?: string, context?: string): object => {
    if (type(V) !== "Object") {
      throw makeException(
        TypeError,
        "is not an object",
        prefix,
        context,
      );
    }
    return V as object;
  },

  // Timestamp converters
  DOMTimeStamp: createIntegerConversion(64, { unsigned: true }),
  DOMHighResTimeStamp: (V: unknown): number => toNumber(V),
  BufferSource: (
    V: unknown,
    prefix?: string,
    context?: string,
    // deno-lint-ignore no-explicit-any
    opts: any = {},
  ): ArrayBuffer | ArrayBufferView => {
    if (webidl.bufferSourceTypes.isArrayBufferView(V)) {
      if (
        opts.allowShared === false &&
        bufferSourceTypes.isSharedArrayBuffer((V as ArrayBufferView).buffer)
      ) {
        throw makeException(
          TypeError,
          "is a view on a SharedArrayBuffer, which is not allowed",
          prefix,
          context,
        );
      }
      return V;
    }

    if (bufferSourceTypes.isArrayBuffer(V)) {
      if (
        opts.allowShared === false && bufferSourceTypes.isSharedArrayBuffer(V)
      ) {
        throw makeException(
          TypeError,
          "is a SharedArrayBuffer, which is not allowed",
          prefix,
          context,
        );
      }
      return V;
    }

    if (
      opts.allowShared !== false && bufferSourceTypes.isSharedArrayBuffer(V)
    ) {
      // deno-lint-ignore no-explicit-any
      return V as any;
    }

    throw makeException(
      TypeError,
      "is not of type BufferSource (ArrayBuffer or ArrayBufferView)",
      prefix,
      context,
    );
  },
};

// Utility function to create nullable converters
// Used by other modules to create nullable type converters
function createNullableConverter<T>(
  converter: (
    V: unknown,
    prefix?: string,
    context?: string,
    opts?: ConversionOptions,
  ) => T,
) {
  return (
    V: unknown,
    prefix?: string,
    context?: string,
    opts: ConversionOptions = {},
  ): T | null => {
    if (V === null || V === undefined) return null;
    return converter(V, prefix, context, opts);
  };
}

// Utility function to create sequence converters
// Used by other modules to create sequence type converters
// Per WebIDL spec 3.2.17 - sequence<T> conversion algorithm
function createSequenceConverter<T>(
  converter: (
    V: unknown,
    prefix?: string,
    context?: string,
    opts?: ConversionOptions,
  ) => T,
) {
  return (
    V: unknown,
    prefix?: string,
    context?: string,
    opts: ConversionOptions = {},
  ): T[] => {
    if (type(V) !== "Object") {
      throw makeException(
        TypeError,
        "can not be converted to sequence.",
        prefix,
        context,
      );
    }

    const obj = V as Record<string | symbol, unknown>;
    const iter = obj?.[Symbol.iterator] as
      | (() => Iterator<unknown>)
      | undefined;
    if (typeof iter !== "function") {
      throw makeException(
        TypeError,
        "can not be converted to sequence.",
        prefix,
        context,
      );
    }

    // Call iterator with proper `this` binding per WebIDL spec 3.2.17
    // This ensures Symbol.iterator methods work correctly with their expected context
    const iterator = iter.call(obj);
    const array: T[] = [];
    while (true) {
      const res = iterator.next();
      if (res.done === true) break;
      const val = converter(
        res.value,
        prefix,
        `${context}, index ${array.length}`,
        opts,
      );
      array.push(val);
    }
    return array;
  };
}

// Utility function to create enumeration converters
// Used by other modules to create enum type converters
function createEnumConverter(name: string, values: string[]) {
  const E = new Set(values);

  return (
    V: unknown,
    prefix?: string,
    _context?: string,
  ): string => {
    const S = String(V);

    if (!E.has(S)) {
      throw new TypeError(
        `${
          prefix ? prefix + ": " : ""
        }The provided value '${S}' is not a valid enum value of type ${name}`,
      );
    }

    return S;
  };
}

// Utility function to create dictionary converters
interface DictionaryMember {
  key: string;
  converter: (
    V: unknown,
    prefix?: string,
    context?: string,
    opts?: ConversionOptions,
  ) => unknown;
  required?: boolean;
  defaultValue?: unknown;
}

// Used by other modules to create dictionary type converters
function createDictionaryConverter(
  name: string,
  ...dictionaries: DictionaryMember[][]
) {
  let hasRequiredKey = false;
  const allMembers: DictionaryMember[] = [];

  for (const members of dictionaries) {
    for (const member of members) {
      if (member.required) {
        hasRequiredKey = true;
      }
      allMembers.push(member);
    }
  }

  allMembers.sort((a, b) => {
    if (a.key == b.key) {
      return 0;
    }
    return a.key < b.key ? -1 : 1;
  });

  const defaultValues: Record<string, unknown> = {};
  for (const member of allMembers) {
    if (member.defaultValue !== undefined) {
      const idlMemberValue = member.defaultValue;
      const imvType = typeof idlMemberValue;
      if (
        imvType === "number" || imvType === "boolean" ||
        imvType === "string" || imvType === "bigint" ||
        imvType === "undefined"
      ) {
        defaultValues[member.key] = member.converter(idlMemberValue);
      } else {
        Object.defineProperty(defaultValues, member.key, {
          get() {
            return member.converter(idlMemberValue);
          },
          enumerable: true,
        });
      }
    }
  }

  return (
    V: unknown,
    prefix?: string,
    context?: string,
    opts: ConversionOptions = {},
  ): Record<string, unknown> => {
    const typeV = type(V);
    switch (typeV) {
      case "Undefined":
      case "Null":
      case "Object":
        break;
      default:
        throw makeException(
          TypeError,
          "can not be converted to a dictionary",
          prefix,
          context,
        );
    }

    const esDict = V as Record<string, unknown>;
    const idlDict = Object.assign({}, defaultValues);

    if ((V === undefined || V === null) && !hasRequiredKey) {
      return idlDict;
    }

    for (const member of allMembers) {
      const key = member.key;

      let esMemberValue: unknown;
      if (typeV === "Undefined" || typeV === "Null") {
        esMemberValue = undefined;
      } else {
        esMemberValue = esDict[key];
      }

      if (esMemberValue !== undefined) {
        const memberContext = `'${key}' of '${name}'${
          context ? ` (${context})` : ""
        }`;
        const idlMemberValue = member.converter(
          esMemberValue,
          prefix,
          memberContext,
          opts,
        );
        idlDict[key] = idlMemberValue;
      } else if (member.required) {
        throw makeException(
          TypeError,
          `can not be converted to '${name}' because '${key}' is required in '${name}'`,
          prefix,
          context,
        );
      }
    }

    return idlDict;
  };
}

// Interface branding
const brand = Symbol("[[webidl.brand]]");

// Used by other modules to create interface type converters
function createInterfaceConverter(name: string, prototype: object) {
  return (V: unknown, prefix?: string, context?: string): unknown => {
    if (
      typeof V !== "object" || V === null ||
      !Object.prototype.isPrototypeOf.call(prototype, V) ||
      (V as Record<symbol, unknown>)[brand] !== brand
    ) {
      throw makeException(
        TypeError,
        `is not of type ${name}`,
        prefix,
        context,
      );
    }
    return V;
  };
}

// Used by other modules to create branded objects
function createBranded<T>(Type: new() => T): T {
  const t = Object.create(Type.prototype);
  (t as Record<symbol, unknown>)[brand] = brand;
  return t;
}

// Used by other modules to assert branded objects
function assertBranded(self: unknown, prototype: object): void {
  if (
    typeof self !== "object" || self === null ||
    !Object.prototype.isPrototypeOf.call(prototype, self) ||
    (self as Record<symbol, unknown>)[brand] !== brand
  ) {
    throw new TypeError("Illegal invocation");
  }
}

// Used by other modules for illegal constructor errors
function illegalConstructor(): never {
  throw new TypeError("Illegal constructor");
}

// Interface configuration
// Used by other modules to configure WebIDL interfaces
function configureInterface(
  interface_: {
    new(...args: unknown[]): unknown;
    prototype: object;
    name: string;
  },
): void {
  configureProperties(interface_);
  configureProperties(interface_.prototype);
  Object.defineProperty(interface_.prototype, Symbol.toStringTag, {
    value: interface_.name,
    enumerable: false,
    configurable: true,
    writable: false,
  });
}

// Used by other modules to configure object properties
function configureProperties(obj: object): void {
  const descriptors = Object.getOwnPropertyDescriptors(obj);
  for (const key in descriptors) {
    if (!Object.hasOwnProperty.call(descriptors, key)) {
      continue;
    }
    if (key === "constructor") continue;
    if (key === "prototype") continue;
    const descriptor = descriptors[key];
    if (
      Reflect.has(descriptor, "value") &&
      typeof descriptor.value === "function"
    ) {
      Object.defineProperty(obj, key, {
        enumerable: true,
        writable: true,
        configurable: true,
      });
    } else if (Reflect.has(descriptor, "get")) {
      Object.defineProperty(obj, key, {
        enumerable: true,
        configurable: true,
      });
    }
  }
}

// Utility function to create record converters
// Used by other modules to create record<K, V> type converters
function createRecordConverter<K extends string, V>(
  keyConverter: (
    V: unknown,
    prefix?: string,
    context?: string,
    opts?: ConversionOptions,
  ) => K,
  valueConverter: (
    V: unknown,
    prefix?: string,
    context?: string,
    opts?: ConversionOptions,
  ) => V,
) {
  return (
    V: unknown,
    prefix?: string,
    context?: string,
    opts: ConversionOptions = {},
  ): Record<K, V> => {
    if (type(V) !== "Object") {
      throw makeException(
        TypeError,
        "can not be converted to a record",
        prefix,
        context,
      );
    }

    const result: Record<K, V> = {} as Record<K, V>;
    const obj = V as Record<string, unknown>;

    // Get own property keys (both string and symbol, but we only care about strings)
    const keys = Object.keys(obj);

    for (const key of keys) {
      const typedKey = keyConverter(key, prefix, `${context}, key`, opts);
      const typedValue = valueConverter(
        obj[key],
        prefix,
        `${context}, value for key "${key}"`,
        opts,
      );
      result[typedKey] = typedValue;
    }

    return result;
  };
}

// Utility function to create union type converters
// Used by other modules to create union type converters
function createUnionConverter<T>(
  types: Array<{
    test: (V: unknown) => boolean;
    convert: (
      V: unknown,
      prefix?: string,
      context?: string,
      opts?: ConversionOptions,
    ) => T;
  }>,
) {
  return (
    V: unknown,
    prefix?: string,
    context?: string,
    opts: ConversionOptions = {},
  ): T => {
    for (const { test, convert } of types) {
      if (test(V)) {
        return convert(V, prefix, context, opts);
      }
    }
    throw makeException(
      TypeError,
      "is not a valid value for this union type",
      prefix,
      context,
    );
  };
}

// Promise utilities
// Used by other modules to create and resolve promises
function createPromiseConverter<T>(
  innerConverter: (
    V: unknown,
    prefix?: string,
    context?: string,
    opts?: ConversionOptions,
  ) => T,
) {
  return (
    V: unknown,
    prefix?: string,
    context?: string,
    opts: ConversionOptions = {},
  ): Promise<T> => {
    return Promise.resolve(V).then((value) =>
      innerConverter(value, prefix, context, opts)
    );
  };
}

// Buffer source type utilities
const bufferSourceTypes = {
  isArrayBuffer: (V: unknown): V is ArrayBuffer => {
    return V instanceof ArrayBuffer;
  },
  isSharedArrayBuffer: (V: unknown): V is SharedArrayBuffer => {
    return typeof SharedArrayBuffer !== "undefined" &&
      V instanceof SharedArrayBuffer;
  },
  isArrayBufferView: (V: unknown): V is ArrayBufferView => {
    return ArrayBuffer.isView(V);
  },
  isTypedArray: (V: unknown): boolean => {
    return V instanceof Int8Array ||
      V instanceof Uint8Array ||
      V instanceof Uint8ClampedArray ||
      V instanceof Int16Array ||
      V instanceof Uint16Array ||
      V instanceof Int32Array ||
      V instanceof Uint32Array ||
      V instanceof Float32Array ||
      V instanceof Float64Array ||
      (typeof BigInt64Array !== "undefined" && V instanceof BigInt64Array) ||
      (typeof BigUint64Array !== "undefined" && V instanceof BigUint64Array);
  },
  isDataView: (V: unknown): V is DataView => {
    return V instanceof DataView;
  },
};

// Callback function converter
function createCallbackConverter(
  name: string,
  allowNonCallable = false,
) {
  return (
    V: unknown,
    prefix?: string,
    context?: string,
    // deno-lint-ignore no-explicit-any
  ): any => {
    if (typeof V !== "function") {
      if (allowNonCallable && typeof V === "object" && V !== null) {
        return V;
      }
      throw makeException(
        TypeError,
        `is not a function`,
        prefix,
        context,
      );
    }
    return V;
  };
}

function createFrozenArrayConverter<T>(
  elementConverter: (
    V: unknown,
    prefix?: string,
    context?: string,
    opts?: ConversionOptions,
  ) => T,
) {
  return (
    V: unknown,
    prefix?: string,
    context?: string,
    opts: ConversionOptions = {},
  ): ReadonlyArray<T> => {
    const sequence = createSequenceConverter(elementConverter)(
      V,
      prefix,
      context,
      opts,
    );
    return Object.freeze(sequence);
  };
}

function convertSymbol(V: unknown, prefix?: string, context?: string): symbol {
  if (typeof V !== "symbol") {
    throw makeException(
      TypeError,
      "is not a symbol",
      prefix,
      context,
    );
  }
  return V;
}

function requiredArguments(
  length: number,
  required: number,
  prefix: string,
): void {
  if (length < required) {
    const errMsg = `${prefix ? prefix + ": " : ""}${required} argument${
      required === 1 ? "" : "s"
    } required, but only ${length} present`;
    throw new TypeError(errMsg);
  }
}

function isPlatformObject(V: unknown, prototype: object): boolean {
  return typeof V === "object" && V !== null &&
    Object.prototype.isPrototypeOf.call(prototype, V);
}

function clampToRange(
  value: number,
  min: number,
  max: number,
): number {
  return Math.min(Math.max(value, min), max);
}

function enforceRange(
  value: number,
  min: number,
  max: number,
  prefix?: string,
  context?: string,
): number {
  if (!Number.isFinite(value)) {
    throw makeException(
      TypeError,
      "is not a finite number",
      prefix,
      context,
    );
  }
  if (value < min || value > max) {
    throw makeException(
      TypeError,
      `is outside the accepted range of ${min} to ${max}, inclusive`,
      prefix,
      context,
    );
  }
  return value;
}

const webidl = {
  converters,
  createNullableConverter,
  createSequenceConverter,
  createRecordConverter,
  createUnionConverter,
  createPromiseConverter,
  createFrozenArrayConverter,
  createCallbackConverter,
  createEnumConverter,
  createDictionaryConverter,
  createInterfaceConverter,
  createBranded,
  assertBranded,
  configureInterface,
  configureProperties,
  requiredArguments,
  isPlatformObject,
  clampToRange,
  enforceRange,
  bufferSourceTypes,
  convertSymbol,
  makeException,
  type,
};

// deno-lint-ignore no-explicit-any
(globalThis as any).webidl = webidl;
