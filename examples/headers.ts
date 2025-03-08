const httpHeaders = {
  "Content-Type": "image/jpeg",
  "X-My-Custom-Header": "Zeke are cool",
};
const myHeaders = new Headers(httpHeaders);

console.log("myHeaders", myHeaders.get("Content-Type")); // image/jpeg
console.log("myHeaders", myHeaders.get("X-My-Custom-Header")); // Zeke are cool

// // Append a header to the headers object.
// myHeaders.append("user-agent", "Deno Deploy");

// // Print the headers of the headers object.
// for (const [key, value] of myHeaders.entries()) {
//     console.log(key, value);
// }

// // You can pass the headers instance to Response or Request constructors.
// const request = new Request("https://api.github.com/users/denoland", {
//     method: "POST",
//     headers: myHeaders,
// });
