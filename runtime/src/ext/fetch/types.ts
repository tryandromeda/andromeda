// deno-lint-ignore-file no-explicit-any
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Type definitions for fetch API
type Header = [string, string];
type HeaderList = Header[];

type HeadersGuard =
  | "immutable"
  | "request"
  | "request-no-cors"
  | "response"
  | "none";

type RequestInit = {
  method?: string;
  headers?: HeadersInit;
  body?: any;
  mode?: string;
  credentials?: string;
  cache?: string;
  redirect?: string;
  referrer?: string;
  integrity?: string;
};

type ResponseInit = {
  status?: number;
  statusText?: string;
  headers?: HeadersInit;
};

type HeadersInit =
  | Headers
  | [string, string][]
  | Record<string, string>;

type RequestInfo = Request | string | URL;
