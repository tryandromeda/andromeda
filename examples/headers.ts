const httpHeaders = {
  "Content-Type": "image/jpeg",
  "X-My-Custom-Header": "Zeke are cool",
};
const myHeaders = new Headers(httpHeaders);
console.log("myHeaders", myHeaders.get("Content-Type")); // image/jpeg
console.log("myHeaders", myHeaders.get("X-My-Custom-Header")); // Zeke are cool

const headers2 = [
  ["Set-Cookie", "greeting=hello"],
  ["Set-Cookie", "name=world"],
];
const myHeaders2 = new Headers(headers2);
console.log("myHeaders2", myHeaders2); // TODO Headers { 'Set-Cookie': 'greeting=hello, name=world' } but [object Object]
console.log("myHeaders", myHeaders2.get("Set-Cookie")); // greeting=hellogreeting=hello,name=worldname=world

const myAppendHeader = new Headers();
myAppendHeader.append("Content-Type", "image/jpeg");
console.log("myAppendHeader", myAppendHeader.get("Content-Type")); // 'image/jpeg'

console.log("myAppendHeader._headerList", myAppendHeader._headerList); // undefined
console.log("myAppendHeader._guard", myAppendHeader._guard); // undefined
