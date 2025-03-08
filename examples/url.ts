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

// https://developer.mozilla.org/en-US/docs/Web/API/URL/parse_static#examples
if ("parse" in URL) {
    // Absolute URL
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

    // TODO: error is returned, but null should be returned here
    // Invalid base URL (missing colon)
    // result = URL.parse("en-US/docs", "https//developer.mozilla.org");
    // console.log(`[5]: ${result}`);
} else {
    console.log("URL.parse() not supported");
}
