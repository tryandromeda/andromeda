const validBase64 = "YQ==";
console.log("validBase64: ", atob(validBase64)); // "a"

const validBase64Multiple = "SGVsbG8sIEFuZHJvbWVkYSE=";
console.log("validBase64Multiple: ", atob(validBase64Multiple)); // "Hello, Andromeda!"
