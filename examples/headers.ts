const myHeaders = new Headers({
    accept: "application/json",
});

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

