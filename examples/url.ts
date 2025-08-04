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
