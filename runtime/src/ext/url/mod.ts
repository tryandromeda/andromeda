// deno-lint-ignore-file no-unused-vars
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

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

class URL {
  url: string;
  base?: string;
  serialized: string;
  constructor(url: string, base?: string) {
    this.url = url;
    this.base = base;
    this.serialized = base ?
      __andromeda__.internal_url_parse(url, base) :
      __andromeda__.internal_url_parse_no_base(url);
  }

  toString() {
    return this.serialized;
  }

  get searchParams(): URLSearchParams {
    return new URLSearchParams(this);
  }

  get href(): string {
    return this.serialized;
  }

  static parse(url: string, base?: string) {
    return new this(url, base);
  }

  get protocol(): string {
    return __andromeda__.internal_url_get_protocol(this.serialized);
  }

  get origin(): string {
    return __andromeda__.internal_url_get_origin(this.serialized);
  }

  get username(): string {
    return __andromeda__.internal_url_get_username(this.serialized);
  }

  set username(v: string) {
    this.serialized = __andromeda__.internal_url_set_username(
      this.serialized,
      v,
    );
  }

  get password(): string {
    return __andromeda__.internal_url_get_password(this.serialized);
  }

  set password(v: string) {
    this.serialized = __andromeda__.internal_url_set_password(
      this.serialized,
      v,
    );
  }

  get host(): string {
    return __andromeda__.internal_url_get_host(this.serialized);
  }

  get hostname(): string {
    return __andromeda__.internal_url_get_hostname(this.serialized);
  }

  set hostname(v: string) {
    this.serialized = __andromeda__.internal_url_set_hostname(
      this.serialized,
      v,
    );
  }

  get port(): string {
    return __andromeda__.internal_url_get_port(this.serialized);
  }

  set port(v: string) {
    this.serialized = __andromeda__.internal_url_set_port(this.serialized, v);
  }

  get pathname(): string {
    return __andromeda__.internal_url_get_pathname(this.serialized);
  }

  set pathname(v: string) {
    this.serialized = __andromeda__.internal_url_set_pathname(
      this.serialized,
      v,
    );
  }

  get search(): string {
    return __andromeda__.internal_url_get_search(this.serialized);
  }

  set search(v: string) {
    this.serialized = __andromeda__.internal_url_set_search(this.serialized, v);
  }

  get hash(): string {
    return __andromeda__.internal_url_get_hash(this.serialized);
  }

  set hash(v: string) {
    this.serialized = __andromeda__.internal_url_set_hash(this.serialized, v);
  }
}

class URLSearchParams {
  #pairs: Array<[string, string]> = [];
  #url?: URL;

  constructor(
    init?: string | Array<[string, string]> | Record<string, string> | URL,
  ) {
    if (init instanceof URL) {
      this.#url = init;
      this.#parse(init.search);
    } else if (typeof init === "string") {
      this.#parse(init);
    } else if (Array.isArray(init)) {
      for (const [k, v] of init) this.append(k, v);
    } else if (init && typeof init === "object") {
      for (const k of Object.keys(init)) {
        this.append(k, (init as Record<string, string>)[k]);
      }
    }
  }

  #parse(s: string) {
    this.#pairs = [];
    if (!s) return;
    let q = s;
    if (q.startsWith("?")) q = q.slice(1);
    if (q === "") return;
    for (const part of q.split("&")) {
      if (part === "") continue;
      const idx = part.indexOf("=");
      if (idx === -1) {
        this.#pairs.push([decodeURIComponent(part), ""]);
      } else {
        const k = decodeURIComponent(part.slice(0, idx));
        const v = decodeURIComponent(part.slice(idx + 1));
        this.#pairs.push([k, v]);
      }
    }
  }

  #updateURL() {
    if (this.#url) {
      const s = this.toString();
      this.#url.search = s ? `?${s}` : "";
    }
  }

  append(name: string, value: string) {
    this.#pairs.push([String(name), String(value)]);
    this.#updateURL();
  }

  set(name: string, value: string) {
    name = String(name);
    value = String(value);
    let found = false;
    this.#pairs = this.#pairs.filter(([k, _]) => {
      if (k === name) {
        if (!found) {
          found = true;
          return true; // keep one
        }
        return false; // drop subsequent
      }
      return true;
    });
    if (found) {
      for (let i = 0; i < this.#pairs.length; i++) {
        if (this.#pairs[i][0] === name) {
          this.#pairs[i][1] = value;
          break;
        }
      }
    } else {
      this.#pairs.push([name, value]);
    }
    this.#updateURL();
  }

  delete(name: string) {
    const before = this.#pairs.length;
    this.#pairs = this.#pairs.filter(([k, _]) => k !== name);
    if (this.#pairs.length !== before) this.#updateURL();
  }

  get(name: string) {
    for (const [k, v] of this.#pairs) if (k === name) return v;
    return null;
  }

  getAll(name: string) {
    const res: string[] = [];
    for (const [k, v] of this.#pairs) if (k === name) res.push(v);
    return res;
  }

  has(name: string) {
    for (const [k, _] of this.#pairs) if (k === name) return true;
    return false;
  }

  toString() {
    return this.#pairs
      .map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`)
      .join("&");
  }

  forEach(cb: (value: string, key: string) => void) {
    for (const [k, v] of this.#pairs) cb(v, k);
  }

  *entries() {
    for (const p of this.#pairs) yield p;
  }

  *keys() {
    for (const [k] of this.#pairs) yield k;
  }

  *values() {
    for (const [, v] of this.#pairs) yield v;
  }
}

// searchParams is defined per-instance in the constructor above.
