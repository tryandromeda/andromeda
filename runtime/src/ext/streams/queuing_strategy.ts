// deno-lint-ignore-file no-unused-vars
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/**
 * CountQueuingStrategy
 */
class CountQueuingStrategy {
    highWaterMark: number;

    constructor(init: { highWaterMark: number }) {
        if (typeof init !== "object" || init === null) {
            throw new TypeError("CountQueuingStrategy constructor requires an object");
        }
        
        if (typeof init.highWaterMark !== "number") {
            throw new TypeError("highWaterMark must be a number");
        }
        
        this.highWaterMark = init.highWaterMark;
    }

    size(_chunk?: unknown): number {
        return 1;
    }
}

/**
 * ByteLengthQueuingStrategy
 * Measures the size of a chunk by its byteLength property
 */
class ByteLengthQueuingStrategy {
    highWaterMark: number;

    constructor(init: { highWaterMark: number }) {
        if (typeof init !== "object" || init === null) {
            throw new TypeError("ByteLengthQueuingStrategy constructor requires an object");
        }
        
        if (typeof init.highWaterMark !== "number") {
            throw new TypeError("highWaterMark must be a number");
        }
        
        this.highWaterMark = init.highWaterMark;
    }

    size(chunk?: unknown): number {
        if (chunk === null || chunk === undefined) {
            return 0;
        }
        
        // Try to get byteLength from various buffer-like objects
        if (typeof chunk === "object" && chunk !== null) {
            const obj = chunk as Record<string, unknown>;
            
            // Check for ArrayBuffer, TypedArray, or DataView
            if (typeof obj.byteLength === "number") {
                return obj.byteLength;
            }
            
            // Check for ArrayBufferView (TypedArray)
            if (obj instanceof Uint8Array || obj instanceof Uint16Array || 
                obj instanceof Uint32Array || obj instanceof Int8Array ||
                obj instanceof Int16Array || obj instanceof Int32Array ||
                obj instanceof Float32Array || obj instanceof Float64Array ||
                obj instanceof DataView) {
                return obj.byteLength;
            }
            
            // Check for ArrayBuffer
            if (obj instanceof ArrayBuffer) {
                return obj.byteLength;
            }
        }
        
        // For strings, use UTF-8 byte length
        if (typeof chunk === "string") {
            const encoder = new TextEncoder();
            return encoder.encode(chunk).length;
        }
        
        // Default: treat as having no measurable size
        return 0;
    }
}
