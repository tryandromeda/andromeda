const ok = "a";
console.log("ok: ", btoa(ok)); // YQ==

const notOK = "✓";
console.log("notOK: ", btoa(notOK)); // error
