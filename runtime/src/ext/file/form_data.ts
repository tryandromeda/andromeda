// deno-lint-ignore-file no-unused-vars
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * Implementation of the FormData interface
 * Based on: https://xhr.spec.whatwg.org/#formdata
 * WinterTC Compliance: https://min-common-api.proposal.wintertc.org/
 */

// Minimal HTMLFormElement interface for type compatibility
interface HTMLFormElement {
  readonly tagName: string;
}

type FormDataEntryValue = File | string;

/**
 * FormData provides a way to easily construct a set of key/value pairs representing form fields and their values
 */
class FormData {
  #formDataId: string;
  #entries: Map<string, FormDataEntryValue[]>;

  constructor(form?: HTMLFormElement) {
    // For WinterTC compliance, we ignore the form parameter as there's no DOM
    // Per the spec: Node.js and Deno throw if the first parameter is not undefined
    if (form !== undefined) {
      throw new TypeError(
        "FormData constructor: form parameter is not supported in non-DOM environments",
      );
    }

    this.#formDataId = internal_formdata_create();
    this.#entries = new Map();
  }

  /**
   * Appends a new value onto an existing key inside a FormData object,
   * or adds the key if it does not already exist.
   */
  append(name: string, value: FormDataEntryValue, filename?: string): void {
    const normalizedName = String(name);

    let normalizedValue: FormDataEntryValue;
    if (value instanceof File) {
      normalizedValue = value;
    } else if (typeof value === "object" && value !== null && "type" in value && "size" in value) {
      // Duck-type check for Blob-like object
      const blobValue = value as Blob;
      const fileName = filename ?? "blob";
      normalizedValue = new File([blobValue], fileName, { type: blobValue.type });
    } else {
      normalizedValue = String(value);
    }

    // Add to internal map
    if (!this.#entries.has(normalizedName)) {
      this.#entries.set(normalizedName, []);
    }
    this.#entries.get(normalizedName)!.push(normalizedValue);

    // Call native implementation
    const valueStr = normalizedValue instanceof File ?
      `file:${normalizedValue.name}:${normalizedValue.type}:${normalizedValue.size}` :
      String(normalizedValue);
    internal_formdata_append(this.#formDataId, normalizedName, valueStr);
  }

  /**
   * Deletes a key and all its values from a FormData object.
   */
  delete(name: string): void {
    const normalizedName = String(name);
    this.#entries.delete(normalizedName);
    internal_formdata_delete(this.#formDataId, normalizedName);
  }

  /**
   * Returns the first value associated with a given key from within a FormData object.
   */
  get(name: string): FormDataEntryValue | null {
    const normalizedName = String(name);
    const values = this.#entries.get(normalizedName);
    return values && values.length > 0 ? values[0] : null;
  }

  /**
   * Returns all the values associated with a given key from within a FormData object.
   */
  getAll(name: string): FormDataEntryValue[] {
    const normalizedName = String(name);
    const values = this.#entries.get(normalizedName);
    return values ? [...values] : [];
  }

  /**
   * Returns whether a FormData object contains a certain key.
   */
  has(name: string): boolean {
    const normalizedName = String(name);
    return this.#entries.has(normalizedName);
  }

  /**
   * Sets a new value for an existing key inside a FormData object,
   * or adds the key/value if it does not already exist.
   */
  set(name: string, value: FormDataEntryValue, filename?: string): void {
    const normalizedName = String(name);

    let normalizedValue: FormDataEntryValue;
    if (value instanceof File) {
      normalizedValue = value;
    } else if (typeof value === "object" && value !== null && "type" in value && "size" in value) {
      // Duck-type check for Blob-like object
      const blobValue = value as Blob;
      const fileName = filename ?? "blob";
      normalizedValue = new File([blobValue], fileName, { type: blobValue.type });
    } else {
      normalizedValue = String(value);
    }

    // Set in internal map (replace all existing values)
    this.#entries.set(normalizedName, [normalizedValue]);

    // Call native implementation
    const valueStr = normalizedValue instanceof File ?
      `file:${normalizedValue.name}:${normalizedValue.type}:${normalizedValue.size}` :
      String(normalizedValue);
    internal_formdata_set(this.#formDataId, normalizedName, valueStr);
  }

  /**
   * Returns an iterator of all the keys of the FormData.
   */
  *keys(): IterableIterator<string> {
    for (const key of this.#entries.keys()) {
      yield key;
    }
  }

  /**
   * Returns an iterator of all the values of the FormData.
   */
  *values(): IterableIterator<FormDataEntryValue> {
    for (const values of this.#entries.values()) {
      for (const value of values) {
        yield value;
      }
    }
  }

  /**
   * Returns an iterator of all the key/value pairs of the FormData.
   */
  *entries(): IterableIterator<[string, FormDataEntryValue]> {
    for (const [key, values] of this.#entries.entries()) {
      for (const value of values) {
        yield [key, value];
      }
    }
  }

  /**
   * Executes a provided function once for each key/value pair of the FormData.
   */
  forEach(
    callback: (value: FormDataEntryValue, key: string, parent: FormData) => void,
    thisArg?: unknown,
  ): void {
    for (const [key, value] of this) {
      callback.call(thisArg, value, key, this);
    }
  }

  /**
   * Returns an iterator of all the key/value pairs of the FormData.
   * This makes FormData iterable.
   */
  [Symbol.iterator](): IterableIterator<[string, FormDataEntryValue]> {
    return this.entries();
  }

  /**
   * Returns the string tag for Object.prototype.toString
   */
  get [Symbol.toStringTag]() {
    return "FormData";
  }
}
