// https://developer.mozilla.org/en-US/docs/Web/API/Request#examples
const test1 = () => {
  const request = new Request("https://www.mozilla.org/favicon.ico");

  const url = request.url;
  console.log("url", url);
  const method = request.method;
  console.log("method", method);
  const credentials = request.credentials;
  console.log("credentials", credentials);
};
test1();

// https://developer.mozilla.org/en-US/docs/Web/API/Request#examples
const test2 = () => {
  const request = new Request("https://example.com", {
    method: "POST",
    body: "{\"foo\": \"bar\"}",
  });

  const url = request.url;
  console.log("url", url);
  const method = request.method;
  console.log("method", method);
  const credentials = request.credentials;
  console.log("credentials", credentials);
  const bodyUsed = request.bodyUsed;
  console.log("bodyUsed", bodyUsed);
};
test2();

// https://developer.mozilla.org/en-US/docs/Web/API/Request/Request#options
// TODO: comment in nova support module
// function test3() {
//   const oldRequest = new Request(
//     "https://github.com/mdn/content/issues/12959",
//     { headers: { From: "webmaster@example.org" } },
//   );
//   console.log(oldRequest.headers.get("From")); // "webmaster@example.org"
//   const newRequest = new Request(oldRequest, {
//     headers: { From: "developer@example.org" },
//   });
//   console.log(newRequest.headers.get("From")); // "developer@example.org"
// }
// test3();
