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

type RequestMode = "navigate" | "same-origin" | "no-cors" | "cors";

type RequestCredentials = "omit" | "same-origin" | "include";

type RequestCache =
  | "default"
  | "no-store"
  | "reload"
  | "no-cache"
  | "force-cache"
  | "only-if-cached";

type RequestRedirect = "follow" | "error" | "manual";

type RequestDuplex = "half";

type RequestPriority = "high" | "low" | "auto";

type ResponseType =
  | "basic"
  | "cors"
  | "default"
  | "error"
  | "opaque"
  | "opaqueredirect";

type RequestInit = {
  method?: string;
  headers?: HeadersInit;
  body?: any;
  mode?: RequestMode;
  credentials?: RequestCredentials;
  cache?: RequestCache;
  redirect?: RequestRedirect;
  referrer?: string;
  referrerPolicy?: string;
  integrity?: string;
  keepalive?: boolean;
  signal?: AbortSignal | null;
  priority?: RequestPriority;
  duplex?: RequestDuplex;
  window?: any;
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
