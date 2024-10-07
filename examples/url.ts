// taken from https://developer.mozilla.org/en-US/docs/Web/API/URL/URL

const baseUrl = "https://developer.mozilla.org";

const A = new URL("/", baseUrl); // 'https://developer.mozilla.org/'
console.log(A)

const B = new URL(baseUrl); // 'https://developer.mozilla.org/'
console.log(B)

console.log(new URL("en-US/docs", B)); // 'https://developer.mozilla.org/en-US/docs'


const D = new URL("/en-US/docs", B); // 'https://developer.mozilla.org/en-US/docs'
console.log(D)

console.log(new URL("/en-US/docs", D)); // 'https://developer.mozilla.org/en-US/docs'

console.log(new URL("/en-US/docs", A)); // 'https://developer.mozilla.org/en-US/docs'

new URL("/en-US/docs", "https://developer.mozilla.org/fr-FR/toto"); // 'https://developer.mozilla.org/en-US/docs'