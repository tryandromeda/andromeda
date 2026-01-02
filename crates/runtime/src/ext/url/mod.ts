// deno-lint-ignore-file no-explicit-any
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

const webidl = (globalThis as any).webidl;

const _list = Symbol("[[query]]");
const _urlObject = Symbol("[[url object]]");
const _updateUrlSearch = Symbol("updateUrlSearch");

function encodeURIComponent(s: string): string {
  const encoder = new TextEncoder();
  const bytes = encoder.encode(s);
  let out = "";
  for (const b of bytes) {
    // unreserved characters A-Z a-z 0-9 - _ . ~
    if (
      (b >= 0x30 && b <= 0x39) || (b >= 0x41 && b <= 0x5A) ||
      (b >= 0x61 && b <= 0x7A) || b === 0x2D || b === 0x5F || b === 0x2E ||
      b === 0x7E
    ) {
      out += String.fromCharCode(b);
    } else {
      out += "%" + b.toString(16).toUpperCase().padStart(2, "0");
    }
  }
  return out;
}

function decodeURIComponent(s: string): string {
  const bytes: number[] = [];
  for (let i = 0; i < s.length; i++) {
    const ch = s[i];
    if (ch === "%") {
      const hex = s.slice(i + 1, i + 3);
      const val = parseInt(hex, 16);
      if (!Number.isNaN(val)) {
        bytes.push(val);
        i += 2;
        continue;
      }
    }
    bytes.push(s.charCodeAt(i));
  }
  const decoder = new TextDecoder();
  return decoder.decode(new Uint8Array(bytes));
}

function encodeURI(s: string): string {
  const encoded = encodeURIComponent(s);
  return encoded.replace(/%3B/g, ";")
    .replace(/%2C/g, ",")
    .replace(/%2F/g, "/")
    .replace(/%3F/g, "?")
    .replace(/%3A/g, ":")
    .replace(/%40/g, "@")
    .replace(/%26/g, "&")
    .replace(/%3D/g, "=")
    .replace(/%2B/g, "+")
    .replace(/%24/g, "$");
}

function decodeURI(s: string): string {
  return decodeURIComponent(s);
}

// URLSearchParams implementation with WebIDL
class URLSearchParams {
  [_list]: Array<[string, string]> = [];
  [_urlObject]: URL | null = null;

  constructor(
    init:
      | string
      | Array<[string, string]>
      | Record<string, string>
      | URLSearchParams = "",
  ) {
    const prefix = "Failed to construct 'URLSearchParams'";

    // Create branded object first
    const self = webidl.createBranded(URLSearchParams);
    Object.setPrototypeOf(this, self);

    // Handle different init types manually (union converter might not be ready yet)
    if (init === "" || init === null || init === undefined) {
      this[_list] = [];
      return;
    }

    if (typeof init === "string") {
      // If init is a string and starts with "?", remove it
      if (init[0] === "?") {
        init = init.slice(1);
      }
      this.#parse(init);
    } else if (Array.isArray(init)) {
      // Sequence of sequences
      for (let i = 0; i < init.length; i++) {
        const pair = init[i];
        if (pair.length !== 2) {
          throw new TypeError(
            `${prefix}: Item ${i} in the parameter list does not have length 2 exactly`,
          );
        }
        this[_list].push([String(pair[0]), String(pair[1])]);
      }
    } else if (typeof init === "object") {
      // Record
      for (const key of Object.keys(init)) {
        this[_list].push([key, String((init as Record<string, string>)[key])]);
      }
    }
  }

  #parse(input: string): void {
    this[_list] = [];
    if (!input) return;

    for (const part of input.split("&")) {
      if (part === "") continue;
      const idx = part.indexOf("=");
      if (idx === -1) {
        this[_list].push([decodeURIComponent(part), ""]);
      } else {
        const k = decodeURIComponent(part.slice(0, idx));
        const v = decodeURIComponent(part.slice(idx + 1));
        this[_list].push([k, v]);
      }
    }
  }

  #updateUrlSearch(): void {
    const url = this[_urlObject];
    if (url === null) {
      return;
    }
    (url as any)[_updateUrlSearch](this.toString());
  }

  append(name: string, value: string): void {
    webidl.assertBranded(this, URLSearchParams.prototype);
    const prefix = "Failed to execute 'append' on 'URLSearchParams'";
    webidl.requiredArguments(arguments.length, 2, prefix);
    name = webidl.converters.USVString(name, prefix, "Argument 1");
    value = webidl.converters.USVString(value, prefix, "Argument 2");

    this[_list].push([name, value]);
    this.#updateUrlSearch();
  }

  delete(name: string, value?: string): void {
    webidl.assertBranded(this, URLSearchParams.prototype);
    const prefix = "Failed to execute 'delete' on 'URLSearchParams'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    name = webidl.converters.USVString(name, prefix, "Argument 1");

    const list = this[_list];
    let i = 0;
    if (value === undefined) {
      while (i < list.length) {
        if (list[i][0] === name) {
          list.splice(i, 1);
        } else {
          i++;
        }
      }
    } else {
      value = webidl.converters.USVString(value, prefix, "Argument 2");
      while (i < list.length) {
        if (list[i][0] === name && list[i][1] === value) {
          list.splice(i, 1);
        } else {
          i++;
        }
      }
    }
    this.#updateUrlSearch();
  }

  get(name: string): string | null {
    webidl.assertBranded(this, URLSearchParams.prototype);
    const prefix = "Failed to execute 'get' on 'URLSearchParams'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    name = webidl.converters.USVString(name, prefix, "Argument 1");

    for (const [k, v] of this[_list]) {
      if (k === name) return v;
    }
    return null;
  }

  getAll(name: string): string[] {
    webidl.assertBranded(this, URLSearchParams.prototype);
    const prefix = "Failed to execute 'getAll' on 'URLSearchParams'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    name = webidl.converters.USVString(name, prefix, "Argument 1");

    const values: string[] = [];
    for (const [k, v] of this[_list]) {
      if (k === name) {
        values.push(v);
      }
    }
    return values;
  }

  has(name: string, value?: string): boolean {
    webidl.assertBranded(this, URLSearchParams.prototype);
    const prefix = "Failed to execute 'has' on 'URLSearchParams'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    name = webidl.converters.USVString(name, prefix, "Argument 1");

    if (value !== undefined) {
      value = webidl.converters.USVString(value, prefix, "Argument 2");
      return this[_list].some((entry) =>
        entry[0] === name && entry[1] === value
      );
    }
    return this[_list].some((entry) => entry[0] === name);
  }

  set(name: string, value: string): void {
    webidl.assertBranded(this, URLSearchParams.prototype);
    const prefix = "Failed to execute 'set' on 'URLSearchParams'";
    webidl.requiredArguments(arguments.length, 2, prefix);
    name = webidl.converters.USVString(name, prefix, "Argument 1");
    value = webidl.converters.USVString(value, prefix, "Argument 2");

    const list = this[_list];
    let found = false;
    let i = 0;
    while (i < list.length) {
      if (list[i][0] === name) {
        if (!found) {
          list[i][1] = value;
          found = true;
          i++;
        } else {
          list.splice(i, 1);
        }
      } else {
        i++;
      }
    }
    if (!found) {
      list.push([name, value]);
    }
    this.#updateUrlSearch();
  }

  sort(): void {
    webidl.assertBranded(this, URLSearchParams.prototype);
    this[_list].sort((a, b) => {
      if (a[0] < b[0]) return -1;
      if (a[0] > b[0]) return 1;
      return 0;
    });
    this.#updateUrlSearch();
  }

  toString(): string {
    webidl.assertBranded(this, URLSearchParams.prototype);
    return this[_list]
      .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)
      .join("&");
  }

  forEach(
    callbackfn: (value: string, key: string, parent: URLSearchParams) => void,
    thisArg?: any,
  ): void {
    webidl.assertBranded(this, URLSearchParams.prototype);
    const prefix = "Failed to execute 'forEach' on 'URLSearchParams'";
    webidl.requiredArguments(arguments.length, 1, prefix);

    if (typeof callbackfn !== "function") {
      throw new TypeError(`${prefix}: Argument 1 is not a function`);
    }

    for (const [key, value] of this[_list]) {
      callbackfn.call(thisArg, value, key, this);
    }
  }

  *keys(): IterableIterator<string> {
    webidl.assertBranded(this, URLSearchParams.prototype);
    for (const [k] of this[_list]) {
      yield k;
    }
  }

  *values(): IterableIterator<string> {
    webidl.assertBranded(this, URLSearchParams.prototype);
    for (const [, v] of this[_list]) {
      yield v;
    }
  }

  *entries(): IterableIterator<[string, string]> {
    webidl.assertBranded(this, URLSearchParams.prototype);
    for (const pair of this[_list]) {
      yield pair;
    }
  }

  [Symbol.iterator](): IterableIterator<[string, string]> {
    return this.entries();
  }

  get size(): number {
    webidl.assertBranded(this, URLSearchParams.prototype);
    return this[_list].length;
  }
}

// Configure URLSearchParams interface with WebIDL
webidl.configureInterface(URLSearchParams);

// URL implementation with WebIDL
class URL {
  #serialized: string = "";
  #queryObject: URLSearchParams | null = null;

  constructor(url: string, base?: string) {
    const prefix = "Failed to construct 'URL'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    url = webidl.converters.USVString(url, prefix, "Argument 1");
    if (base !== undefined) {
      base = webidl.converters.USVString(base, prefix, "Argument 2");
    }

    // Create branded object
    const self = webidl.createBranded(URL);
    Object.setPrototypeOf(this, self);

    // Parse the URL
    const parsed = base ?
      __andromeda__.internal_url_parse(url, base) :
      __andromeda__.internal_url_parse_no_base(url);

    if (parsed.startsWith("Error:")) {
      throw new TypeError("Invalid URL");
    }
    this.#serialized = parsed;
  }

  [_updateUrlSearch](value: string): void {
    this.#serialized = __andromeda__.internal_url_set_search(
      this.#serialized,
      value,
    );
  }

  static parse(url: string, base?: string): URL | null {
    const prefix = "Failed to execute 'URL.parse'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    url = webidl.converters.USVString(url, prefix, "Argument 1");
    if (base !== undefined) {
      base = webidl.converters.USVString(base, prefix, "Argument 2");
    }

    try {
      return new URL(url, base);
    } catch {
      return null;
    }
  }

  static canParse(url: string, base?: string): boolean {
    const prefix = "Failed to execute 'URL.canParse'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    url = webidl.converters.USVString(url, prefix, "Argument 1");
    if (base !== undefined) {
      base = webidl.converters.USVString(base, prefix, "Argument 2");
    }

    try {
      new URL(url, base);
      return true;
    } catch {
      return false;
    }
  }

  get href(): string {
    webidl.assertBranded(this, URL.prototype);
    return this.#serialized;
  }

  set href(value: string) {
    webidl.assertBranded(this, URL.prototype);
    const prefix = "Failed to set 'href' on 'URL'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    value = webidl.converters.USVString(value, prefix, "Argument 1");

    const parsed = __andromeda__.internal_url_parse_no_base(value);
    if (parsed.startsWith("Error:")) {
      throw new TypeError("Invalid URL");
    }
    this.#serialized = parsed;
    this.#updateSearchParams();
  }

  get origin(): string {
    webidl.assertBranded(this, URL.prototype);
    return __andromeda__.internal_url_get_origin(this.#serialized);
  }

  get protocol(): string {
    webidl.assertBranded(this, URL.prototype);
    return __andromeda__.internal_url_get_protocol(this.#serialized);
  }

  set protocol(value: string) {
    webidl.assertBranded(this, URL.prototype);
    const prefix = "Failed to set 'protocol' on 'URL'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    value = webidl.converters.USVString(value, prefix, "Argument 1");

    // Note: protocol setting may fail silently per spec
    try {
      const newUrl = this.#serialized.replace(/^[^:]+:/, value + ":");
      const parsed = __andromeda__.internal_url_parse_no_base(newUrl);
      if (!parsed.startsWith("Error:")) {
        this.#serialized = parsed;
      }
    } catch {
      // Silently ignore invalid protocol
    }
  }

  get username(): string {
    webidl.assertBranded(this, URL.prototype);
    return __andromeda__.internal_url_get_username(this.#serialized);
  }

  set username(value: string) {
    webidl.assertBranded(this, URL.prototype);
    const prefix = "Failed to set 'username' on 'URL'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    value = webidl.converters.USVString(value, prefix, "Argument 1");

    this.#serialized = __andromeda__.internal_url_set_username(
      this.#serialized,
      value,
    );
  }

  get password(): string {
    webidl.assertBranded(this, URL.prototype);
    return __andromeda__.internal_url_get_password(this.#serialized);
  }

  set password(value: string) {
    webidl.assertBranded(this, URL.prototype);
    const prefix = "Failed to set 'password' on 'URL'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    value = webidl.converters.USVString(value, prefix, "Argument 1");

    this.#serialized = __andromeda__.internal_url_set_password(
      this.#serialized,
      value,
    );
  }

  get host(): string {
    webidl.assertBranded(this, URL.prototype);
    const hostname = __andromeda__.internal_url_get_hostname(this.#serialized);
    const port = __andromeda__.internal_url_get_port(this.#serialized);
    if (port) {
      return `${hostname}:${port}`;
    }
    return hostname;
  }

  set host(value: string) {
    webidl.assertBranded(this, URL.prototype);
    const prefix = "Failed to set 'host' on 'URL'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    value = webidl.converters.USVString(value, prefix, "Argument 1");

    const colonIndex = value.lastIndexOf(":");
    if (colonIndex !== -1) {
      const hostname = value.substring(0, colonIndex);
      const port = value.substring(colonIndex + 1);
      this.#serialized = __andromeda__.internal_url_set_hostname(
        this.#serialized,
        hostname,
      );
      this.#serialized = __andromeda__.internal_url_set_port(
        this.#serialized,
        port,
      );
    } else {
      this.#serialized = __andromeda__.internal_url_set_hostname(
        this.#serialized,
        value,
      );
    }
  }

  get hostname(): string {
    webidl.assertBranded(this, URL.prototype);
    return __andromeda__.internal_url_get_hostname(this.#serialized);
  }

  set hostname(value: string) {
    webidl.assertBranded(this, URL.prototype);
    const prefix = "Failed to set 'hostname' on 'URL'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    value = webidl.converters.USVString(value, prefix, "Argument 1");

    this.#serialized = __andromeda__.internal_url_set_hostname(
      this.#serialized,
      value,
    );
  }

  get port(): string {
    webidl.assertBranded(this, URL.prototype);
    return __andromeda__.internal_url_get_port(this.#serialized);
  }

  set port(value: string) {
    webidl.assertBranded(this, URL.prototype);
    const prefix = "Failed to set 'port' on 'URL'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    value = webidl.converters.USVString(value, prefix, "Argument 1");

    this.#serialized = __andromeda__.internal_url_set_port(
      this.#serialized,
      value,
    );
  }

  get pathname(): string {
    webidl.assertBranded(this, URL.prototype);
    return __andromeda__.internal_url_get_pathname(this.#serialized);
  }

  set pathname(value: string) {
    webidl.assertBranded(this, URL.prototype);
    const prefix = "Failed to set 'pathname' on 'URL'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    value = webidl.converters.USVString(value, prefix, "Argument 1");

    this.#serialized = __andromeda__.internal_url_set_pathname(
      this.#serialized,
      value,
    );
  }

  get search(): string {
    webidl.assertBranded(this, URL.prototype);
    const query = __andromeda__.internal_url_get_search(this.#serialized);
    return query ? `?${query}` : "";
  }

  set search(value: string) {
    webidl.assertBranded(this, URL.prototype);
    const prefix = "Failed to set 'search' on 'URL'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    value = webidl.converters.USVString(value, prefix, "Argument 1");

    this.#serialized = __andromeda__.internal_url_set_search(
      this.#serialized,
      value,
    );
    this.#updateSearchParams();
  }

  get hash(): string {
    webidl.assertBranded(this, URL.prototype);
    const fragment = __andromeda__.internal_url_get_hash(this.#serialized);
    return fragment ? `#${fragment}` : "";
  }

  set hash(value: string) {
    webidl.assertBranded(this, URL.prototype);
    const prefix = "Failed to set 'hash' on 'URL'";
    webidl.requiredArguments(arguments.length, 1, prefix);
    value = webidl.converters.USVString(value, prefix, "Argument 1");

    this.#serialized = __andromeda__.internal_url_set_hash(
      this.#serialized,
      value,
    );
  }

  get searchParams(): URLSearchParams {
    webidl.assertBranded(this, URL.prototype);
    if (this.#queryObject === null) {
      this.#queryObject = new URLSearchParams(this.search);
      (this.#queryObject as any)[_urlObject] = this;
    }
    return this.#queryObject;
  }

  #updateSearchParams(): void {
    if (this.#queryObject !== null) {
      const newSearch = this.search.slice(1);
      const newParams = new URLSearchParams(newSearch);
      (this.#queryObject as any)[_list] = (newParams as any)[_list];
    }
  }

  toString(): string {
    webidl.assertBranded(this, URL.prototype);
    return this.#serialized;
  }

  toJSON(): string {
    webidl.assertBranded(this, URL.prototype);
    return this.#serialized;
  }
}

// Configure URL interface with WebIDL
webidl.configureInterface(URL);

// Create WebIDL converters for URL types
webidl.converters["URL"] = webidl.createInterfaceConverter(
  "URL",
  URL.prototype,
);

webidl.converters["URLSearchParams"] = webidl.createInterfaceConverter(
  "URLSearchParams",
  URLSearchParams.prototype,
);

webidl.converters[
  "sequence<sequence<USVString>> or record<USVString, USVString> or USVString"
] = (V: any, prefix?: string, context?: string, opts?: any) => {
  if (webidl.type(V) === "Object" && V !== null) {
    if (V[Symbol.iterator] !== undefined) {
      return webidl.converters["sequence<sequence<USVString>>"](
        V,
        prefix,
        context,
        opts,
      );
    }
    return webidl.converters["record<USVString, USVString>"](
      V,
      prefix,
      context,
      opts,
    );
  }
  return webidl.converters.USVString(V, prefix, context, opts);
};

// @ts-ignore globalThis is not readonly
globalThis.URL = URL;
// @ts-ignore globalThis is not readonly
globalThis.URLSearchParams = URLSearchParams;
// @ts-ignore globalThis is not readonly
globalThis.encodeURIComponent = encodeURIComponent;
// @ts-ignore globalThis is not readonly
globalThis.decodeURIComponent = decodeURIComponent;
// @ts-ignore globalThis is not readonly
globalThis.encodeURI = encodeURI;
// @ts-ignore globalThis is not readonly
globalThis.decodeURI = decodeURI;
