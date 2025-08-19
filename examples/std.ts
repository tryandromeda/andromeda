import { Queue } from "https://tryandromeda.dev/std/collections/mod.ts";
import { flatten } from "https://tryandromeda.dev/std/data/mod.ts";

const queue = new Queue();
queue.enqueue("first");
queue.enqueue("second");
console.log(queue.dequeue());
console.log(queue.size);

console.log(flatten([[1, 2], [3, [4, 5]]], 1));
console.log(flatten([[1, 2], [3, [4, 5]]], 2));
