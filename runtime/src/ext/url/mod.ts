// deno-lint-ignore-file no-unused-vars
class URL {
  // TODO: Need to return a URL object.
  constructor(url: string, base?: string) {
    // @ts-ignore - this is a hack to make the URL object work
    this.url = url;
    // @ts-ignore - this is a hack to make the Base URL object work
    this.base = base;
    // @ts-ignore - this is a hack to make the URL object work
    this.serialized = base
      ? internal_url_parse(url, base)
      : internal_url_parse_no_base(url);
  }

  toString() {
    // @ts-ignore - this is a hack to make the URL object work
    return this.serialized;
  }

  static parse(url: string, base?: string) {
    return new this(url, base);
  }
}
