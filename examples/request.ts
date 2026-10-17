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
    body: '{"foo": "bar"}',
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
