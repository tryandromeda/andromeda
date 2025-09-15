// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// deno-lint-ignore-file no-explicit-any

// Import existing Request/Response implementations
// These are loaded by the fetch extension, so we don't need to import them here

export interface ServeHandler {
  (request: Request): Response | Promise<Response>;
}

export interface ServeOptions {
  port?: number;
  hostname?: string;
  signal?: AbortSignal;
}

export interface ServeInit extends ServeOptions {
  handler: ServeHandler;
}

export interface HttpServer {
  finished: Promise<void>;
  shutdown(): Promise<void>;
  unref(): void;
  ref(): void;
}

const serverHandlers = new Map<number, ServeHandler>();

export function serve(handler: ServeHandler | ServeInit): HttpServer {
  const options = typeof handler === "function" ?
    { handler, port: 8080, hostname: "127.0.0.1" } :
    { port: 8080, hostname: "127.0.0.1", ...handler };

  const serverId = __andromeda__.internal_http_listen(
    options.hostname || "127.0.0.1",
    options.port || 8080,
  );

  serverHandlers.set(serverId, options.handler);

  return {
    finished: new Promise((resolve) => {
      // TODO: Track server lifecycle
    }),
    shutdown: async () => {
      __andromeda__.internal_http_close(serverId);
      serverHandlers.delete(serverId);
    },
    unref: () => {
      // TODO: Implement unref
    },
    ref: () => {
      // TODO: Implement ref
    },
  };
}

// Export serve function globally for namespace to pick up
// @ts-ignore - internal use
globalThis.__andromeda_http_serve = serve;
