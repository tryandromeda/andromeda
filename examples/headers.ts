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
console.log("myHeaders2", myHeaders2.getSetCookie()); // ["greeting=hello", "name=world"]
console.log("myHeaders", myHeaders2.get("Set-Cookie")); // greeting=hellogreeting=hello,name=worldname=world

const myAppendHeader = new Headers();
myAppendHeader.append("Content-Type", "image/jpeg");
console.log("myAppendHeader", myAppendHeader.get("Content-Type")); // 'image/jpeg'

// Create a new test Headers object
// https://developer.mozilla.org/en-US/docs/Web/API/Headers/forEach
myHeaders.append("Content-Type", "application/json");
myHeaders.append("Cookie", "This is a demo cookie");
myHeaders.append("compression", "gzip");

myHeaders.forEach((value, key) => {
  console.log(`${key} ==> ${value}`);
});
