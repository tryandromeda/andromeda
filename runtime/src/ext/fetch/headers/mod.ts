// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

class Headers {
  // TODO: Private properties
  // #guard
  // #headerList

  // TODO: this is HeaderList type
  // https://fetch.spec.whatwg.org/#headers-class
  constructor(init = undefined) {
    // @ts-ignore
    this.guard = "none";
    // @ts-ignore
    this.headerList = [];
    fillHeaders(this, init);
  }

  // https://fetch.spec.whatwg.org/#dom-headers-get
  get(name: string) {
    return getHeader(this.headerList, name);
  }

  // https://fetch.spec.whatwg.org/#dom-headers-append
  append(name, value) {
    return appendHeader(this, name, value);
  }
}

function fillHeaders(headers: Headers, object) {
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

function byteLowerCase(s) {
  // NOTE: correct since all callers convert to ByteString first
  // TODO(@AaronO): maybe prefer a ByteString_Lower webidl converter
  return s;
}

//  https://fetch.spec.whatwg.org/#concept-headers-append
function appendHeader(headers, name, value) {
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

function normalizeHeaderValue(potentialValue) {
  return httpTrim(potentialValue);
}

// TODO: move to web
function isHttpWhitespace(char) {
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
function httpTrim(s) {
  if (!isHttpWhitespace(s[0]) && !isHttpWhitespace(s[s.length - 1])) {
    return s;
  }
  // return String.prototype.match(s, HTTP_BETWEEN_WHITESPACE)?.[1] ?? "";
  // TODO: implement to nova RegExp
  return s;
}

//  https://fetch.spec.whatwg.org/#concept-header-list-get
function getHeader(list, name) {
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
    return entries.join(entries, "\x2C\x20");
  }
}
