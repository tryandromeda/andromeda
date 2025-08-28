// taken from https://developer.mozilla.org/en-US/docs/Web/API/URL/URL

const baseUrl = "https://developer.mozilla.org";

const A = new URL("/", baseUrl); // 'https://developer.mozilla.org/'
console.log(A);

const B = new URL(baseUrl); // 'https://developer.mozilla.org/'
console.log(B);

console.log(new URL("en-US/docs", B)); // 'https://developer.mozilla.org/en-US/docs'

const D = new URL("/en-US/docs", B); // 'https://developer.mozilla.org/en-US/docs'
console.log(D);

console.log(new URL("/en-US/docs", D)); // 'https://developer.mozilla.org/en-US/docs'

console.log(new URL("/en-US/docs", A)); // 'https://developer.mozilla.org/en-US/docs'

new URL("/en-US/docs", "https://developer.mozilla.org/fr-FR/toto"); // 'https://developer.mozilla.org/en-US/docs'

let result = URL.parse("https://developer.mozilla.org/en-US/docs");
console.log(`[1]: ${result}`);

// Relative reference to a valid base URL
result = URL.parse("en-US/docs", "https://developer.mozilla.org");
console.log(`[2]: ${result}`);

// Relative reference to a "complicated" valid base URL
// (only the scheme and domain are used to resolve url)
result = URL.parse(
  "/different/place",
  "https://developer.mozilla.org:443/some/path?id=4",
);
console.log(`[3]: ${result}`);

// Absolute url argument (base URL ignored)
result = URL.parse(
  "https://example.org/some/docs",
  "https://developer.mozilla.org",
);
console.log(`[4]: ${result}`);

console.log(
  `Username: ${
    new URL("https://user:pass@developer.mozilla.org/en-US/docs").username
  }`,
);
console.log(
  `Password: ${
    new URL("https://user:pass@developer.mozilla.org/en-US/docs").password
  }`,
);
console.log(
  `Host: ${new URL("https://user:pass@developer.mozilla.org/en-US/docs").host}`,
);
console.log(
  `Hostname: ${
    new URL("https://user:pass@developer.mozilla.org/en-US/docs").hostname
  }`,
);
console.log(
  `Port: ${
    new URL("https://user:pass@developer.mozilla.org:8080/en-US/docs").port
  }`,
);
console.log(
  `Pathname: ${
    new URL("https://user:pass@developer.mozilla.org/en-US/docs").pathname
  }`,
);
console.log(
  `Search: ${
    new URL(
      "https://user:pass@developer.mozilla.org/en-US/docs?foo=bar&baz=qux",
    ).search
  }`,
);
console.log(
  `Hash: ${
    new URL("https://user:pass@developer.mozilla.org/en-US/docs#section1").hash
  }`,
);

// Additional checks
const u = new URL(
  "https://user:pass@developer.mozilla.org:8080/en-US/docs?foo=bar#sec",
);
console.log(`href: ${u.href}`);
console.log(`origin: ${u.origin}`);
console.log(`protocol: ${u.protocol}`);
console.log(`toJSON: ${u.toString()}`);

// Mutate some properties (TS shim will call into host ops we must implement)
u.hostname = "example.org";
u.port = "3000";
u.pathname = "/new/path";
u.search = "?a=1&b=2";
u.hash = "#changed";
u.username = "alice";
u.password = "s3cr3t";

console.log(`mutated href: ${u.href}`);
console.log(`mutated origin: ${u.origin}`);
console.log("--- URLSearchParams tests ---");
const p = new URLSearchParams("?a=1&b=2");
console.log("p.get(a):", p.get("a"));
p.append("c", "3");
console.log("p.toString after append c=3:", p.toString());
p.set("a", "9");
console.log("p.getAll(a):", p.getAll("a"));
p.delete("b");
console.log("p.toString after delete b:", p.toString());

// test integration with URL.search
const u2 = new URL("https://example.com/path?x=1");
console.log("u2.search initially:", u2.search);
u2.searchParams.append("y", "2");
console.log("u2.search after append y=2:", u2.search);
