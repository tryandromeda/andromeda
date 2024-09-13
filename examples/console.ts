console.log("This is a log message");
console.log("This is a log message", "with multiple arguments");
console.debug("This is a debug message");
console.warn("This is a warning message");
console.error("This is an error message");
console.info("This is an info message");

const name = prompt("What is your name?");
console.log(`Hello, ${name}`);

const thoughts = confirm("Do you agree to share your data?");

if (thoughts) {
  alert("Thank you for sharing your data.");
} else {
  alert("Thank you for not sharing your data.");
}
