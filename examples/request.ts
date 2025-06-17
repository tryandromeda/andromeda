const oldRequest = new Request("https://github.com/mdn/content/issues/12959", {
  headers: { From: "webmaster@example.org" },
});
console.log("oldRequest", oldRequest);
