// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

interface ServeHandler {
  (request: Request): Response | Promise<Response>;
}

interface ServeOptions {
  port?: number;
  hostname?: string;
  signal?: AbortSignal;
  reusePort?: boolean;
  key?: string;
  cert?: string;
  onError?: (error: unknown) => Response | Promise<Response>;
  onListen?: (params: { hostname: string; port: number; }) => void;
  handler?: ServeHandler;
}

function parseHttpRequest(data: string): {
  method: string;
  path: string;
  headers: Headers;
  body: string;
} {
  const lines = data.split("\r\n");
  if (lines.length === 0) {
    return { method: "GET", path: "/", headers: new Headers(), body: "" };
  }

  const requestLine = lines[0].split(" ");
  const method = requestLine[0] || "GET";
  const path = requestLine[1] || "/";

  const headers = new Headers();
  let i = 1;
  for (; i < lines.length; i++) {
    if (lines[i] === "") break;
    const colonIndex = lines[i].indexOf(":");
    if (colonIndex > 0) {
      const key = lines[i].substring(0, colonIndex).trim();
      const value = lines[i].substring(colonIndex + 1).trim();
      headers.append(key, value);
    }
  }

  const body = lines.slice(i + 1).join("\r\n");
  // TODO: Handle chunked transfer encoding
  // TODO: Handle multipart form data
  // TODO: Validate Content-Length header

  return { method, path, headers, body };
}

function createHttpResponse(
  statusCode: number,
  statusText: string,
  headers: Headers,
  body: string,
): string {
  const lines = [`HTTP/1.1 ${statusCode} ${statusText}`];

  // Iterate all headers using forEach
  headers.forEach((value, name) => {
    lines.push(`${name}: ${value}`);
  });

  if (!headers.has("Content-Length")) {
    lines.push(`Content-Length: ${body.length}`);
  }

  if (!headers.has("Connection")) {
    lines.push("Connection: close");
  }

  lines.push("", body);
  return lines.join("\r\n");
}

async function handleConnection(
  connectionId: number,
  handler: ServeHandler,
  options: ServeOptions,
): Promise<void> {
  try {
    const READ_BUFFER_SIZE = "4096";

    const readResult = await __andromeda__.tcp_read_async(
      connectionId,
      READ_BUFFER_SIZE,
    );
    if (!readResult || readResult === "0") {
      __andromeda__.tcp_close(connectionId);
      return;
    }

    const {
      method,
      path,
      headers,
      body: requestBody,
    } = parseHttpRequest(readResult);

    const url = `http://${options.hostname}:${options.port}${path}`;

    const request = new Request(url, {
      method: method,
      headers: headers,
      body: requestBody || null,
    });

    const response = await handler(request);

    let responseBody = "";
    if (response.body) {
      responseBody = await response.text();
    }

    const statusCode = response.status || 200;
    const statusText = response.statusText || "OK";
    const responseHeaders = response.headers;

    const httpResponse = createHttpResponse(
      statusCode,
      statusText,
      responseHeaders,
      responseBody,
    );
    await __andromeda__.tcp_write_async(connectionId, httpResponse);
    __andromeda__.tcp_close(connectionId);
  } catch (error) {
    try {
      __andromeda__.tcp_close(connectionId);
    } catch (e) {}
  }
}

async function serve(
  handlerOrOptions: ServeHandler | ServeOptions,
  maybeOptions?: ServeOptions,
): Promise<void> {
  const DEFAULT_HOSTNAME = "127.0.0.1";
  const DEFAULT_PORT = 8080;

  let handler: ServeHandler;
  let options: ServeOptions;

  if (typeof handlerOrOptions === "function") {
    handler = handlerOrOptions;
    options = maybeOptions ?? {};
  } else {
    options = handlerOrOptions;
    if (!options.handler) {
      throw new Error(
        "Handler function is required. Provide either serve(handler) or serve({ handler, ... })",
      );
    }
    handler = options.handler;
  }

  const hostname = options.hostname ?? DEFAULT_HOSTNAME;
  const port = options.port ?? DEFAULT_PORT;
  console.info(`HTTP server running on http://${hostname}:${port}/`);

  const listenResult = __andromeda__.tcp_listen(hostname, port);
  const listenData = JSON.parse(listenResult);
  if (!listenData.success) {
    console.error("Failed to create listener:", listenData.error);
    return;
  }
  const listenerId = listenData.resourceId;

  while (true) {
    try {
      const acceptPromise = __andromeda__.tcp_accept_async(String(listenerId));
      const acceptResult = await acceptPromise;
      const result = JSON.parse(acceptResult);
      if (!result.success) {
        continue;
      }

      // TODO: Handle connections concurrently instead of sequentially
      handleConnection(result.resourceId, handler, {
        ...options,
        hostname,
        port,
      });
    } catch (error) {
      break;
    }
  }
}

// TODO: Add support for server configuration options:
// - TLS/HTTPS support
// - Connection timeout
// - Request size limits
// - Keep-Alive support
// - HTTP/2 support
// - WebSocket upgrade
// - Signal/AbortController for graceful shutdown
// - onListen callback

globalThis.__andromeda_http_serve = serve;
