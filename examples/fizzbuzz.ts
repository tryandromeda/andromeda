// FizzBuzz example

for (let i = 1; i <= 100; i++) {
    let output = "";
    if (i % 3 === 0) {
        output += "Fizz";
    }
    if (i % 5 === 0) {
        output += "Buzz";
    }
    if (output === "") {
        output = i.toString();
    }
    console.log(output);
}
