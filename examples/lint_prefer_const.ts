// Variables that should be const
let name = "John";
let age = 25;
let isActive = true;

// Variable that should stay let (reassigned)
let counter = 0;
counter++;

// Variable that should stay let (reassigned in conditional)
let status = "pending";
if (age > 18) {
    status = "approved";
}

// Variable that should be const (never reassigned)
let config = { api: "https://example.com" };

console.log(name, age, isActive, counter, status, config);
