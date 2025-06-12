// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Compliant with WHATWG HTML Living Standard
// https://html.spec.whatwg.org/multipage/structured-data.html#dom-structuredclone

// deno-lint-ignore-file no-explicit-any no-unused-vars

/**
 * Options for structuredClone
 */
interface StructuredSerializeOptions {
  /**
   * An array of transferable objects that will be transferred rather than cloned.
   * The objects will be rendered unusable in the sending context after the transfer.
   */
  transfer?: Transferable[];
}

/**
 * Interface for transferable objects
 */
interface Transferable {
  // Marker interface for objects that can be transferred
  readonly __transferable?: never;
}

/**
 * Create a DataCloneError DOMException
 */
function createDataCloneError(message: string): Error {
  const error = new Error(message);
  error.name = "DataCloneError";
  // Set DOMException code for DataCloneError (25)
  (error as any).code = 25;
  return error;
}

/**
 * Check if a value is transferable
 */
function isTransferable(value: any): value is Transferable {
  return value instanceof ArrayBuffer;
}

/**
 * Serialize a value to JSON representation for structured cloning
 */
function structuredSerialize(value: any, transferList: Transferable[] = []): string {
  const memory = new Map();
  const transferSet = new Set(transferList);

  function serializeInternal(val: any): any {
    if (memory.has(val)) {
      return { type: "reference", id: memory.get(val) };
    }

    if (
      val === null ||
      val === undefined ||
      typeof val === "boolean" ||
      typeof val === "number" ||
      typeof val === "string"
    ) {
      return { type: "primitive", value: val };
    }

    if (typeof val === "bigint") {
      return { type: "bigint", value: val.toString() };
    }

    if (typeof val === "symbol") {
      throw createDataCloneError("Symbol values cannot be cloned");
    }

    if (typeof val === "function") {
      throw createDataCloneError("Function objects cannot be cloned");
    }

    const id = memory.size;
    memory.set(val, id);

    if (val instanceof Boolean) {
      return { type: "Boolean", id, value: val.valueOf() };
    }
    if (val instanceof Number) {
      return { type: "Number", id, value: val.valueOf() };
    }
    if (val instanceof String) {
      return { type: "String", id, value: val.valueOf() };
    }
    if (val instanceof BigInt) {
      return { type: "BigInt", id, value: val.toString() };
    }

    if (val instanceof Date) {
      return { type: "Date", id, value: val.getTime() };
    }
    if (val instanceof RegExp) {
      throw createDataCloneError(
        "RegExp objects are not yet supported for cloning in this runtime",
      );
    }
    if (val instanceof ArrayBuffer) {
      if (transferSet.has(val as any)) {
        const transferIndex = transferList.indexOf(val as any);
        return {
          type: "ArrayBuffer",
          id,
          transfer: true,
          transferIndex,
          byteLength: val.byteLength,
        };
      } else {
        try {
          const bytes = Array.from(new Uint8Array(val));
          return { type: "ArrayBuffer", id, transfer: false, bytes };
        } catch (error) {
          throw createDataCloneError(`Failed to serialize ArrayBuffer: ${error}`);
        }
      }
    }

    if (ArrayBuffer.isView(val)) {
      const buffer = serializeInternal(val.buffer);
      if (val instanceof DataView) {
        return {
          type: "DataView",
          id,
          buffer,
          byteOffset: val.byteOffset,
          byteLength: val.byteLength,
        };
      } else {
        // TypedArray
        const typedArray = val as any; // Cast to access length property
        return {
          type: "TypedArray",
          id,
          constructor: val.constructor.name,
          buffer,
          byteOffset: val.byteOffset,
          length: typedArray.length,
        };
      }
    }

    if (val instanceof Map) {
      const entries = [];
      for (const [key, value] of val.entries()) {
        entries.push([serializeInternal(key), serializeInternal(value)]);
      }
      return { type: "Map", id, entries };
    }

    if (val instanceof Set) {
      const values = [];
      for (const value of val.values()) {
        values.push(serializeInternal(value));
      }
      return { type: "Set", id, values };
    }

    if (val instanceof Error) {
      return {
        type: "Error",
        id,
        name: val.name,
        message: val.message,
        stack: val.stack,
      };
    }

    if (Array.isArray(val)) {
      const elements = [];
      for (let i = 0; i < val.length; i++) {
        elements[i] = serializeInternal(val[i]);
      }
      return { type: "Array", id, length: val.length, elements };
    }

    if (val.constructor === Object || val.constructor === undefined) {
      const properties: any = {};
      for (const key in val) {
        if (Object.prototype.hasOwnProperty.call(val, key)) {
          properties[key] = serializeInternal(val[key]);
        }
      }
      return { type: "Object", id, properties };
    }

    throw createDataCloneError(`${val.constructor?.name || "Object"} objects cannot be cloned`);
  }

  try {
    const serialized = serializeInternal(value);
    return JSON.stringify({ root: serialized, transferList: transferList.length });
  } catch (error) {
    if (error instanceof Error && error.name === "DataCloneError") {
      throw error;
    }
    throw createDataCloneError("Failed to serialize value for cloning");
  }
}

/**
 * Deserialize a JSON representation back to JavaScript values
 */
function structuredDeserialize(serializedData: string, transferredValues: any[] = []): any {
  const data = JSON.parse(serializedData);
  const memory = new Map();

  function deserializeInternal(serialized: any): any {
    if (!serialized || typeof serialized !== "object") {
      return serialized;
    }

    if (serialized.type === "reference") {
      if (!memory.has(serialized.id)) {
        throw createDataCloneError("Invalid reference in serialized data");
      }
      return memory.get(serialized.id);
    }

    if (serialized.type === "primitive") {
      return serialized.value;
    }

    if (serialized.type === "bigint") {
      return BigInt(serialized.value);
    }

    // Handle objects with IDs
    if (typeof serialized.id === "number") {
      let result: any;

      switch (serialized.type) {
        case "Boolean":
          result = new Boolean(serialized.value);
          break;
        case "Number":
          result = new Number(serialized.value);
          break;
        case "String":
          result = new String(serialized.value);
          break;
        case "BigInt":
          result = Object(BigInt(serialized.value));
          break;
        case "Date":
          result = new Date(serialized.value);
          break;
        case "ArrayBuffer":
          if (serialized.transfer) {
            // Get from transferred values using the correct index
            const transferIndex = serialized.transferIndex;
            if (transferIndex < transferredValues.length) {
              result = transferredValues[transferIndex];
            } else {
              throw createDataCloneError("Missing transferred ArrayBuffer");
            }
          } else {
            try {
              result = new ArrayBuffer(serialized.bytes.length);
              const view = new Uint8Array(result);
              // Use manual copy since TypedArray.set is not implemented in Nova
              for (let i = 0; i < serialized.bytes.length; i++) {
                view[i] = serialized.bytes[i];
              }
            } catch (error) {
              throw createDataCloneError(`Failed to deserialize ArrayBuffer: ${error}`);
            }
          }
          break;
        case "DataView": {
          const buffer = deserializeInternal(serialized.buffer);
          result = new DataView(buffer, serialized.byteOffset, serialized.byteLength);
          break;
        }
        case "TypedArray": {
          const arrayBuffer = deserializeInternal(serialized.buffer);
          const Constructor = globalThis[serialized.constructor as keyof typeof globalThis] as any;
          result = new Constructor(arrayBuffer, serialized.byteOffset, serialized.length);
          break;
        }
        case "Map":
          result = new Map();
          memory.set(serialized.id, result);
          for (const [key, value] of serialized.entries) {
            result.set(deserializeInternal(key), deserializeInternal(value));
          }
          return result;
        case "Set":
          result = new Set();
          memory.set(serialized.id, result);
          for (const value of serialized.values) {
            result.add(deserializeInternal(value));
          }
          return result;
        case "Error": {
          const ErrorConstructor =
            globalThis[serialized.name as keyof typeof globalThis] as ErrorConstructor || Error;
          result = new ErrorConstructor(serialized.message);
          result.name = serialized.name;
          if (serialized.stack) {
            result.stack = serialized.stack;
          }
          break;
        }
        case "Array":
          result = new Array(serialized.length);
          memory.set(serialized.id, result);
          for (let i = 0; i < serialized.elements.length; i++) {
            result[i] = deserializeInternal(serialized.elements[i]);
          }
          return result;
        case "Object":
          result = {};
          memory.set(serialized.id, result);
          for (const key in serialized.properties) {
            result[key] = deserializeInternal(serialized.properties[key]);
          }
          return result;
        default:
          throw createDataCloneError(`Unknown serialized type: ${serialized.type}`);
      }

      memory.set(serialized.id, result);
      return result;
    }

    throw createDataCloneError("Invalid serialized data format");
  }

  return deserializeInternal(data.root);
}

/**
 * The structuredClone() method creates a deep clone of a given value using the structured clone algorithm.
 */
function structuredClone<T = any>(value: T, options: StructuredSerializeOptions = {}): T {
  const transferList = options.transfer || [];

  for (const transferable of transferList) {
    if (!isTransferable(transferable)) {
      throw createDataCloneError("Value in transfer list is not transferable");
    }
  }

  const transferSet = new Set(transferList);
  if (transferSet.size !== transferList.length) {
    throw createDataCloneError("Transfer list contains duplicate values");
  }
  try {
    if (transferList.length > 0) {
      const serialized = structuredSerialize(value, transferList);

      const transferredValues: any[] = [];
      for (let i = 0; i < transferList.length; i++) {
        const transferable = transferList[i];
        if (transferable instanceof ArrayBuffer) {
          const transferred = new ArrayBuffer(transferable.byteLength);
          const transferredView = new Uint8Array(transferred);
          const originalView = new Uint8Array(transferable);

          for (let j = 0; j < originalView.length; j++) {
            transferredView[j] = originalView[j];
          }

          transferredValues.push(transferred);

        }
      }

      return structuredDeserialize(serialized, transferredValues) as T;
    } else {
      const serialized = structuredSerialize(value);
      return structuredDeserialize(serialized) as T;
    }
  } catch (error) {
    if (error instanceof Error && error.name === "DataCloneError") {
      throw error;
    }
    throw createDataCloneError("Failed to clone value");
  }
}