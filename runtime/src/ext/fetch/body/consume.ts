// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// deno-lint-ignore-file no-explicit-any

/**
 * Consumes the body and returns it as an ArrayBuffer.
 * @see https://fetch.spec.whatwg.org/#dom-body-arraybuffer
 */
async function consumeArrayBuffer(body: any): Promise<ArrayBuffer> {
  const bytes = await body.consume();
  // Create a new ArrayBuffer with a copy of the data
  const buffer = new ArrayBuffer(bytes.byteLength);
  new Uint8Array(buffer).set(bytes);
  return buffer;
}

/**
 * Consumes the body and returns it as a Blob.
 * @see https://fetch.spec.whatwg.org/#dom-body-blob
 */
async function consumeBlob(
  body: any,
  contentType?: string | null,
): Promise<Blob> {
  const bytes = await body.consume();
  return new Blob([bytes], {
    type: contentType || "",
  });
}

/**
 * Consumes the body and returns it as a Uint8Array.
 * This is a non-standard extension for easier access to raw bytes.
 */
async function consumeBytes(body: any): Promise<Uint8Array> {
  return await body.consume();
}

/**
 * Consumes the body and returns it as FormData.
 * @see https://fetch.spec.whatwg.org/#dom-body-formdata
 */
async function consumeFormData(
  body: any,
  contentType?: string | null,
): Promise<FormData> {
  const bytes = await body.consume();
  
  if (!contentType) {
    throw new TypeError("Missing Content-Type for FormData");
  }

  const contentTypeLower = contentType.toLowerCase();

  // Parse multipart/form-data
  if (contentTypeLower.includes("multipart/form-data")) {
    const boundary = extractBoundary(contentType);
    if (!boundary) {
      throw new TypeError("Missing boundary in multipart/form-data");
    }
    return parseMultipartFormData(bytes, boundary);
  }

  // Parse application/x-www-form-urlencoded
  if (contentTypeLower.includes("application/x-www-form-urlencoded")) {
    return parseUrlEncodedFormData(bytes);
  }

  throw new TypeError(
    "FormData can only be parsed from multipart/form-data or application/x-www-form-urlencoded",
  );
}

/**
 * Consumes the body and returns it as JSON.
 * @see https://fetch.spec.whatwg.org/#dom-body-json
 */
async function consumeJson<T = unknown>(body: any): Promise<T> {
  const text = await consumeText(body);
  return JSON.parse(text) as T;
}

/**
 * Consumes the body and returns it as text.
 * @see https://fetch.spec.whatwg.org/#dom-body-text
 */
async function consumeText(body: any): Promise<string> {
  const bytes = await body.consume();
  return new TextDecoder().decode(bytes);
}

/**
 * Extracts the boundary from a multipart/form-data Content-Type header.
 */
function extractBoundary(contentType: string): string | null {
  const match = /boundary=(?:"([^"]+)"|([^;]+))/.exec(contentType);
  if (!match) {
    return null;
  }
  return match[1] || match[2];
}

/**
 * Parses multipart/form-data body.
 * @see https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#multipart-form-data
 */
function parseMultipartFormData(
  bytes: Uint8Array,
  boundary: string,
): FormData {
  const formData = new FormData();
  const decoder = new TextDecoder();
  
  // Convert boundary to bytes
  const boundaryBytes = new TextEncoder().encode("--" + boundary);
  const finalBoundaryBytes = new TextEncoder().encode("--" + boundary + "--");

  let start = 0;
  
  // Find first boundary
  start = indexOfBytes(bytes, boundaryBytes, start);
  if (start === -1) {
    return formData;
  }
  start += boundaryBytes.length;

  while (true) {
    // Skip CRLF after boundary
    if (bytes[start] === 0x0D && bytes[start + 1] === 0x0A) {
      start += 2;
    }

    // Find next boundary
    let end = indexOfBytes(bytes, boundaryBytes, start);
    if (end === -1) {
      // Check for final boundary
      end = indexOfBytes(bytes, finalBoundaryBytes, start);
      if (end === -1) {
        break;
      }
    }

    // Extract this part
    const part = bytes.subarray(start, end);
    parsePart(part, formData, decoder);

    start = end + boundaryBytes.length;

    // Check if this was the final boundary
    if (
      bytes[start] === 0x2D &&
      bytes[start + 1] === 0x2D
    ) {
      break;
    }
  }

  return formData;
}

/**
 * Parses a single part of multipart/form-data.
 */
function parsePart(
  part: Uint8Array,
  formData: FormData,
  decoder: TextDecoder,
): void {
  // Find the blank line separating headers from body
  let headerEnd = -1;
  for (let i = 0; i < part.length - 3; i++) {
    if (
      part[i] === 0x0D &&
      part[i + 1] === 0x0A &&
      part[i + 2] === 0x0D &&
      part[i + 3] === 0x0A
    ) {
      headerEnd = i;
      break;
    }
  }

  if (headerEnd === -1) {
    return;
  }

  const headerBytes = part.subarray(0, headerEnd);
  const bodyBytes = part.subarray(headerEnd + 4);

  // Trim trailing CRLF from body
  let bodyEnd = bodyBytes.length;
  if (
    bodyEnd >= 2 &&
    bodyBytes[bodyEnd - 2] === 0x0D &&
    bodyBytes[bodyEnd - 1] === 0x0A
  ) {
    bodyEnd -= 2;
  }
  const trimmedBody = bodyBytes.subarray(0, bodyEnd);

  // Parse headers
  const headerText = decoder.decode(headerBytes);
  const headers = parseHeaders(headerText);

  // Get Content-Disposition
  const disposition = headers.get("content-disposition");
  if (!disposition) {
    return;
  }

  const name = extractDispositionParam(disposition, "name");
  if (!name) {
    return;
  }

  const filename = extractDispositionParam(disposition, "filename");
  
  if (filename) {
    // File field
    const contentType = headers.get("content-type") || "application/octet-stream";
    const blob = new Blob([trimmedBody], { type: contentType });
    const file = new File([blob], filename, { type: contentType });
    formData.append(name, file);
  } else {
    // Text field
    const value = decoder.decode(trimmedBody);
    formData.append(name, value);
  }
}

/**
 * Parses URL-encoded form data.
 */
function parseUrlEncodedFormData(bytes: Uint8Array): FormData {
  const formData = new FormData();
  const text = new TextDecoder().decode(bytes);
  const params = new URLSearchParams(text);
  
  for (const [name, value] of params.entries()) {
    formData.append(name, value);
  }
  
  return formData;
}

/**
 * Finds the index of a byte sequence in a Uint8Array.
 */
function indexOfBytes(
  haystack: Uint8Array,
  needle: Uint8Array,
  start: number = 0,
): number {
  outer: for (let i = start; i <= haystack.length - needle.length; i++) {
    for (let j = 0; j < needle.length; j++) {
      if (haystack[i + j] !== needle[j]) {
        continue outer;
      }
    }
    return i;
  }
  return -1;
}

/**
 * Parses HTTP headers from text.
 */
function parseHeaders(text: string): Map<string, string> {
  const headers = new Map<string, string>();
  const lines = text.split(/\r?\n/);
  
  for (const line of lines) {
    const colonIndex = line.indexOf(":");
    if (colonIndex === -1) {
      continue;
    }
    
    const name = line.slice(0, colonIndex).trim().toLowerCase();
    const value = line.slice(colonIndex + 1).trim();
    headers.set(name, value);
  }
  
  return headers;
}

/**
 * Extracts a parameter value from a Content-Disposition header.
 */
function extractDispositionParam(
  disposition: string,
  param: string,
): string | null {
  const regex = new RegExp(`${param}=(?:"([^"]+)"|([^;\\s]+))`);
  const match = regex.exec(disposition);
  if (!match) {
    return null;
  }
  return match[1] || match[2];
}

// Export to globalThis for use in other files
(globalThis as any).consumeArrayBuffer = consumeArrayBuffer;
(globalThis as any).consumeBlob = consumeBlob;
(globalThis as any).consumeBytes = consumeBytes;
(globalThis as any).consumeFormData = consumeFormData;
(globalThis as any).consumeJson = consumeJson;
(globalThis as any).consumeText = consumeText;
