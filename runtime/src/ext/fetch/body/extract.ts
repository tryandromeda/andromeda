// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Get InnerBody from globalThis (loaded from inner_body.ts)
// deno-lint-ignore no-explicit-any
const InnerBody = (globalThis as any).InnerBody;

/**
 * Valid body init types according to the Fetch specification.
 * @see https://fetch.spec.whatwg.org/#typedefdef-bodyinit
 */
type BodyInit =
  | ReadableStream<Uint8Array>
  | Blob
  | ArrayBuffer
  | ArrayBufferView
  | FormData
  | URLSearchParams
  | string;

/**
 * Result of extracting a body from a BodyInit value.
 */
interface ExtractedBody {
  body: typeof InnerBody;
  contentType: string | null;
}

/**
 * Extracts a body from a BodyInit value.
 *
 * This implements the "extract a body" algorithm from the Fetch specification.
 * @see https://fetch.spec.whatwg.org/#concept-bodyinit-extract
 *
 * @param object - The BodyInit value to extract from
 * @returns An ExtractedBody with the body and content type
 */
function extractBody(object: BodyInit): ExtractedBody {
  let contentType: string | null = null;
  // deno-lint-ignore no-explicit-any
  let body: any;

  // ReadableStream
  if (object instanceof ReadableStream) {
    body = new InnerBody(object);
    return { body, contentType };
  }

  // Blob
  if (object instanceof Blob) {
    contentType = object.type || null;

    // Convert Blob to ReadableStream
    const stream = object.stream();
    body = new InnerBody(stream);
    return { body, contentType };
  }

  // FormData
  if (object instanceof FormData) {
    // FormData is serialized as multipart/form-data
    const boundary = generateBoundary();
    contentType = `multipart/form-data; boundary=${boundary}`;

    const encoded = encodeFormData(object, boundary);
    body = new InnerBody({ body: encoded, consumed: false });
    return { body, contentType };
  }

  // URLSearchParams
  if (object instanceof URLSearchParams) {
    contentType = "application/x-www-form-urlencoded;charset=UTF-8";

    const encoded = new TextEncoder().encode(object.toString());
    body = new InnerBody({ body: encoded, consumed: false });
    return { body, contentType };
  }

  // String
  if (typeof object === "string") {
    contentType = "text/plain;charset=UTF-8";

    const encoded = new TextEncoder().encode(object);
    body = new InnerBody({ body: encoded, consumed: false });
    return { body, contentType };
  }

  // ArrayBuffer
  if (object instanceof ArrayBuffer) {
    body = new InnerBody({
      body: new Uint8Array(object),
      consumed: false,
    });
    return { body, contentType };
  }

  // ArrayBufferView (TypedArray, DataView)
  if (ArrayBuffer.isView(object)) {
    body = new InnerBody({
      body: new Uint8Array(
        object.buffer,
        object.byteOffset,
        object.byteLength,
      ),
      consumed: false,
    });
    return { body, contentType };
  }

  // This should never happen if TypeScript types are correct
  throw new TypeError("Invalid BodyInit type");
}

/**
 * Generates a random boundary string for multipart/form-data.
 */
function generateBoundary(): string {
  // Generate a random boundary using crypto
  const boundary = "----formdata-andromeda-" +
    Array.from(crypto.getRandomValues(new Uint8Array(16)))
      .map(b => b.toString(16).padStart(2, "0"))
      .join("");
  return boundary;
}

/**
 * Encodes FormData as multipart/form-data.
 * @see https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#multipart-form-data
 */
function encodeFormData(formData: FormData, boundary: string): Uint8Array {
  const encoder = new TextEncoder();
  const parts: Uint8Array[] = [];

  for (const [name, value] of formData.entries()) {
    // Add boundary
    parts.push(encoder.encode(`--${boundary}\r\n`));

    if (typeof value === "string") {
      // Text field
      parts.push(
        encoder.encode(
          `Content-Disposition: form-data; name="${
            escapeQuotes(name)
          }"\r\n\r\n`,
        ),
      );
      parts.push(encoder.encode(value));
      parts.push(encoder.encode("\r\n"));
    } else {
      // File field
      const filename = value.name || "blob";
      const contentType = value.type || "application/octet-stream";

      parts.push(
        encoder.encode(
          `Content-Disposition: form-data; name="${
            escapeQuotes(name)
          }"; filename="${escapeQuotes(filename)}"\r\n`,
        ),
      );
      parts.push(encoder.encode(`Content-Type: ${contentType}\r\n\r\n`));

      // Note: This is synchronous for simplicity
      // In a real implementation, we might want to handle this asynchronously
      // For now, we'll need to add a sync way to read the blob
      // This is a limitation that will need to be addressed
      throw new Error("File upload in FormData not yet supported");
    }
  }

  // Final boundary
  parts.push(encoder.encode(`--${boundary}--\r\n`));

  // Combine all parts
  const totalLength = parts.reduce((acc, part) => acc + part.byteLength, 0);
  const result = new Uint8Array(totalLength);
  let offset = 0;
  for (const part of parts) {
    result.set(part, offset);
    offset += part.byteLength;
  }

  return result;
}

/**
 * Escapes quotes in a string for use in HTTP headers.
 */
function escapeQuotes(str: string): string {
  return str.replace(/"/g, '\\"');
}

/**
 * Clones a body init value if possible.
 * This is used when creating a new Request/Response from an existing one.
 */
function cloneBodyInit(body: BodyInit): BodyInit {
  // ReadableStream cannot be cloned directly
  if (body instanceof ReadableStream) {
    throw new TypeError("Cannot clone a ReadableStream");
  }

  // Blob can be cloned by slicing
  if (body instanceof Blob) {
    return body.slice();
  }

  // FormData can be cloned by creating a new FormData
  if (body instanceof FormData) {
    const cloned = new FormData();
    for (const [name, value] of body.entries()) {
      cloned.append(name, value);
    }
    return cloned;
  }

  // URLSearchParams can be cloned by creating a new one
  if (body instanceof URLSearchParams) {
    return new URLSearchParams(body);
  }

  // String is immutable
  if (typeof body === "string") {
    return body;
  }

  // ArrayBuffer can be copied
  if (body instanceof ArrayBuffer) {
    return body.slice(0);
  }

  // ArrayBufferView can be copied
  if (ArrayBuffer.isView(body)) {
    return new Uint8Array(body.buffer.slice(0));
  }

  throw new TypeError("Invalid BodyInit type");
}

// Export to globalThis for use in other files
// deno-lint-ignore no-explicit-any
(globalThis as any).extractBody = extractBody;
// deno-lint-ignore no-explicit-any
(globalThis as any).cloneBodyInit = cloneBodyInit;
