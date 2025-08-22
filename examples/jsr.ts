import {
    decodeBase64,
    encodeBase64,
} from "https://esm.sh/jsr/@std/encoding@1.0.0/base64";

const base64 = encodeBase64(new TextEncoder().encode("Hello, world!"));
const decoded = decodeBase64(base64);
console.log(base64);
console.log(new TextDecoder().decode(decoded));
