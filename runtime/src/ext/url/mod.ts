// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// deno-lint-ignore-file no-unused-vars
class URL {
  url: string;
  base?: string;
  serialized: string;
  constructor(url: string, base?: string) {
    this.url = url;
    this.base = base;
    this.serialized = base ?
      internal_url_parse(url, base) :
      internal_url_parse_no_base(url);
  }

  toString() {
    return this.serialized;
  }

  static parse(url: string, base?: string) {
    return new this(url, base);
  }

  get protocol(): string {
    return internal_url_get_protocol(this.serialized);
  }

  get username(): string {
    return internal_url_get_username(this.serialized);
  }

  get password(): string {
    return internal_url_get_password(this.serialized);
  }

  get host(): string {
    return internal_url_get_host(this.serialized);
  }

  get hostname(): string {
    return internal_url_get_hostname(this.serialized);
  }

  get port(): string {
    return internal_url_get_port(this.serialized);
  }

  get pathname(): string {
    return internal_url_get_pathname(this.serialized);
  }

  get search(): string {
    return internal_url_get_search(this.serialized);
  }

  get hash(): string {
    return internal_url_get_hash(this.serialized);
  }
}
