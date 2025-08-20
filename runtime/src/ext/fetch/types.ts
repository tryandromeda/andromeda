// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Type definitions for fetch API
export type Header = [string, string];
export type HeaderList = Header[];

export type HeadersGuard = 
  | "immutable" 
  | "request" 
  | "request-no-cors" 
  | "response" 
  | "none";

export type RequestInit = {
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

export type ResponseInit = {
  status?: number;
  statusText?: string;
  headers?: HeadersInit;
};

export type HeadersInit = 
  | Headers 
  | [string, string][] 
  | Record<string, string>;