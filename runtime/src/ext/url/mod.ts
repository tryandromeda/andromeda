// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// deno-lint-ignore-file no-unused-vars
class URL {
  url: string;
  base?: string;
  serialized: string;
  // TODO: Need to return a URL object.
  constructor(url: string, base?: string) {
    this.url = url;
    this.base = base;
    this.serialized = base ?
      internal_url_parse(url, base) :
      internal_url_parse_no_base(url);
  }

  toString() {
    // @ts-ignore - this is a hack to make the URL object work
    return this.serialized;
  }

  static parse(url: string, base?: string) {
    return new this(url, base);
  }
}
