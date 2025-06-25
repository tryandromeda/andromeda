const url = "https://example.com";
const response1 = new Response(`${url}`, {});

console.log("body", response1.body);
console.log("status", response1.status);
console.log("url", response1.url);
console.log("ok", response1.ok);
console.log("redirected", response1.redirected);
console.log("type", response1.type);
console.log("statusText", response1.statusText);
