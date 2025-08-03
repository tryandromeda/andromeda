const stream1 = new ReadableStream({
    start(controller) {
        console.log("  ReadableStream started");
        controller.enqueue("Hello");
        controller.enqueue(" ");
        controller.enqueue("World!");
        controller.close();
        console.log("  ReadableStream data enqueued and closed");
    },
});

const _reader = stream1.getReader();

const chunks: string[] = [];
const stream2 = new WritableStream({
    start(_controller) {
        console.log("  WritableStream started");
    },
    write(chunk, _controller) {
        console.log("  WritableStream received chunk:", chunk);
        if (typeof chunk === "string") {
            chunks.push(chunk);
        } else if (chunk instanceof Uint8Array) {
            const text = new TextDecoder().decode(chunk);
            chunks.push(text);
        }
    },
    close() {
        console.log("  WritableStream closed");
    },
});

const _writer = stream2.getWriter();

const strategy1 = new CountQueuingStrategy({ highWaterMark: 5 });
console.log("  Strategy highWaterMark:", strategy1.highWaterMark);
console.log("  Size of 'test':", strategy1.size("test"));
console.log("  Size of [1,2,3]:", strategy1.size([1, 2, 3]));

const strategy2 = new ByteLengthQueuingStrategy({ highWaterMark: 1024 });
console.log("  Strategy highWaterMark:", strategy2.highWaterMark);

const testBuffer = new Uint8Array([1, 2, 3, 4, 5]);
console.log("  Size of Uint8Array(5):", strategy2.size(testBuffer));

const upperCaseTransform = new TransformStream({
    transform(chunk, controller) {
        console.log("  Transform received chunk:", chunk);
        if (typeof chunk === "string") {
            controller.enqueue(chunk.toUpperCase());
        } else if (chunk instanceof Uint8Array) {
            const text = new TextDecoder().decode(chunk);
            const upperText = text.toUpperCase();
            const encoder = new TextEncoder();
            controller.enqueue(encoder.encode(upperText));
        }
    },
});

console.log("  TransformStream created successfully");
console.log("  Readable side:", upperCaseTransform.readable);
console.log("  Writable side:", upperCaseTransform.writable);
