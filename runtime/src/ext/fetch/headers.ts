// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// deno-lint-ignore-file no-unused-vars

class Headers {
  #guard: HeadersGuard = "none";
  #headerList: HeaderList = [];

  // TODO: this is HeaderList type
  // https://fetch.spec.whatwg.org/#headers-class
  constructor(init = undefined) {
    fillHeaders(this, init);
  }

  clear() {
    this.#headerList = [];
    this.#guard = "none";
  }

  // https://fetch.spec.whatwg.org/#dom-headers-get
  get(name: string) {
    return getHeader(this.#headerList, name);
  }

  // https://fetch.spec.whatwg.org/#dom-headers-getsetcookie
  getSetCookie() {
    const list = this.#headerList;

    const entries = [];
    for (let i = 0; i < list.length; i++) {
      if (byteLowerCase(list[i][0]) === "set-cookie") {
        entries.push(list[i][1]);
      }
    }

    return entries;
  }

  // https://fetch.spec.whatwg.org/#dom-headers-append
  append(name: string, value: string) {
    return appendHeader(this, name, value);
  }

  // https://fetch.spec.whatwg.org/#dom-headers-set
  set(name: string, value: string) {
    value = normalizeHeaderValue(value);

    if (this.#guard === "immutable") {
      throw new TypeError("Cannot change header: headers are immutable");
    }

    const lowercaseName = byteLowerCase(name);
    let found = false;

    // Remove all existing headers with this name and add the new one
    for (let i = this.#headerList.length - 1; i >= 0; i--) {
      if (byteLowerCase(this.#headerList[i][0]) === lowercaseName) {
        if (!found) {
          this.#headerList[i] = [this.#headerList[i][0], value];
          found = true;
        } else {
          this.#headerList.splice(i, 1);
        }
      }
    }

    if (!found) {
      this.#headerList.push([name, value]);
    }
  }

  // https://fetch.spec.whatwg.org/#dom-headers-has
  has(name: string): boolean {
    const lowercaseName = byteLowerCase(name);
    for (const [headerName] of this.#headerList) {
      if (byteLowerCase(headerName) === lowercaseName) {
        return true;
      }
    }
    return false;
  }

  // https://fetch.spec.whatwg.org/#dom-headers-delete
  delete(name: string) {
    if (this.#guard === "immutable") {
      throw new TypeError("Cannot change header: headers are immutable");
    }

    const lowercaseName = byteLowerCase(name);
    for (let i = this.#headerList.length - 1; i >= 0; i--) {
      if (byteLowerCase(this.#headerList[i][0]) === lowercaseName) {
        this.#headerList.splice(i, 1);
      }
    }
  }

  // Iterator methods
  *entries(): IterableIterator<[string, string]> {
    for (const header of this.#headerList) {
      yield [header[0], header[1]];
    }
  }

  *keys(): IterableIterator<string> {
    for (const [name] of this.#headerList) {
      yield name;
    }
  }

  *values(): IterableIterator<string> {
    for (const [, value] of this.#headerList) {
      yield value;
    }
  }

  [Symbol.iterator](): IterableIterator<[string, string]> {
    return this.entries();
  }

  get headerList() {
    return this.#headerList;
  }

  get guard() {
    return this.#guard;
  }

  static getHeadersGuard(o: Headers, guard: HeadersGuard) {
    return o.#guard;
  }

  static setHeadersGuard(o: Headers, guard: HeadersGuard) {
    o.#guard = guard;
  }

  static getHeadersList(target: Headers) {
    return target.#headerList;
  }

  static setHeadersList(target: Headers, list: HeaderList) {
    target.#headerList = list;
  }
}

const { setHeadersList, setHeadersGuard, getHeadersList, getHeadersGuard } =
  Headers;

// deno-lint-ignore no-explicit-any
function fillHeaders(headers: Headers, object: any) {
  if (Array.isArray(object)) {
    for (let i = 0; i < object.length; ++i) {
      const header = object[i];
      if (header.length !== 2) {
        throw new TypeError(
          `Invalid header: length must be 2, but is ${header.length}`,
        );
      }
      appendHeader(headers, header[0], header[1]);
    }
  } else {
    for (const key in object) {
      if (!Object.hasOwn(object, key)) {
        continue;
      }
      appendHeader(headers, key, object[key]);
    }
  }
}

function byteLowerCase(s: string): string {
  // NOTE: correct since all callers convert to ByteString first
  // TODO: Header implementation should be imported from standard library when available
  return s.toLowerCase();
}

//  https://fetch.spec.whatwg.org/#concept-headers-append
function appendHeader(headers: Headers, name: string, value: string) {
  // 1.
  value = normalizeHeaderValue(value);

  // 2. TODO
  // if (!checkHeaderNameForHttpTokenCodePoint(name)) {
  //   throw new TypeError(`Invalid header name: "${name}"`);
  // }
  // if (!checkForInvalidValueChars(value)) {
  //   throw new TypeError(`Invalid header value: "${value}"`);
  // }

  // 3
  if (headers.guard == "immutable") {
    throw new TypeError("Cannot change header: headers are immutable");
  }

  // 7.
  const list = headers.headerList;
  const lowercaseName = byteLowerCase(name);
  for (let i = 0; i < list.length; i++) {
    if (byteLowerCase(list[i][0]) === lowercaseName) {
      name = list[i][0];
      break;
    }
  }
  list.push([name, value]);
}

function normalizeHeaderValue(potentialValue: string): string {
  return httpTrim(potentialValue);
}

// TODO: move to web
function isHttpWhitespace(char: string): boolean {
  switch (char) {
    case "\u0009":
    case "\u000A":
    case "\u000D":
    case "\u0020":
      return true;
    default:
      return false;
  }
}

// const HTTP_BETWEEN_WHITESPACE = new SafeRegExp(
//   `^[${HTTP_WHITESPACE_MATCHER}]*(.*?)[${HTTP_WHITESPACE_MATCHER}]*$`,
// );
// TODO: move to web
function httpTrim(s: string): string {
  if (!isHttpWhitespace(s[0]) && !isHttpWhitespace(s[s.length - 1])) {
    return s;
  }
  // return String.prototype.match(s, HTTP_BETWEEN_WHITESPACE)?.[1] ?? "";
  // TODO: implement to nova RegExp
  return s;
}

//  https://fetch.spec.whatwg.org/#concept-header-list-get
function getHeader(list: [string, string][], name: string): string | null {
  const lowercaseName = byteLowerCase(name);
  const entries = [];
  for (let i = 0; i < list.length; i++) {
    if (byteLowerCase(list[i][0]) === lowercaseName) {
      entries.push(list[i][1]);
    }
  }

  if (entries.length === 0) {
    return null;
  } else {
    return entries.join("\x2C\x20");
  }
}

// Helper function to set a header on a request object
// This handles both Headers objects and plain objects/headersList
function setRequestHeader(request: any, name: string, value: string) {
  if (request.headers instanceof Headers) {
    request.headers.set(name, value);
  } else {
    // Fallback to plain object
    if (!request.headers) {
      request.headers = {};
    }
    request.headers[name] = value;

    // Also update headersList if it exists
    if (request.headersList && Array.isArray(request.headersList)) {
      const lowerName = name.toLowerCase();
      const existingIndex = request.headersList.findIndex(
        ([headerName]) => headerName.toLowerCase() === lowerName,
      );
      if (existingIndex >= 0) {
        request.headersList[existingIndex] = [name, value];
      } else {
        request.headersList.push([name, value]);
      }
    }
  }
}

// Helper function to check if a header exists on a request object
function hasRequestHeader(request: any, name: string): boolean {
  if (request.headers instanceof Headers) {
    return request.headers.has(name);
  } else if (request.headers && typeof request.headers === "object") {
    return name in request.headers || name.toLowerCase() in request.headers;
  }
  return false;
}

// Helper function to get headers as a list from various representations
function getHeadersAsList(headers: any): HeaderList {
  const headersList: HeaderList = [];

  if (headers instanceof Headers) {
    for (const [name, value] of headers.entries()) {
      headersList.push([name, value]);
    }
  } else if (Array.isArray(headers)) {
    headersList.push(...headers);
  } else if (headers && typeof headers === "object") {
    for (const [name, value] of Object.entries(headers)) {
      headersList.push([name, String(value)]);
    }
  }

  return headersList;
}

// Export Headers to globalThis
globalThis.Headers = Headers;
globalThis.setHeadersList = setHeadersList;
globalThis.setHeadersGuard = setHeadersGuard;
globalThis.getHeadersList = getHeadersList;
globalThis.fillHeaders = fillHeaders;
