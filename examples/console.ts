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
// Basic color styling
console.log("%cThis text is red", "color: red");
console.log("%cThis text is blue", "color: blue");
console.log("%cThis text is green", "color: green");

// Background colors
console.log("%cWhite text on red background", "color: white; background-color: red");
console.log("%cBlack text on yellow background", "color: black; background-color: yellow");

// Font styling
console.log("%cThis text is bold", "font-weight: bold");
console.log("%cThis text is italic", "font-style: italic");
console.log("%cThis text is underlined", "text-decoration: underline");

// Combined styling
console.log("%cBold red text with blue background", "color: red; background-color: blue; font-weight: bold");
console.log("%cItalic green text", "color: green; font-style: italic");

// Multiple styled segments in one message
console.log("%cRed %cBlue %cGreen", "color: red", "color: blue", "color: green");

// Hex color support
console.log("%cHex color #ff6600", "color: #ff6600");
console.log("%cHex background #003366", "background-color: #003366; color: white");

// RGB color support
console.log("%cRGB color", "color: rgb(255, 102, 0)");
console.log("%cRGB background", "background-color: rgb(0, 51, 102); color: white");

// Mix of styled and unstyled text
console.log("%cStyled%c and %cunstyled%c text", "color: red; font-weight: bold", "", "color: blue", "");

// Complex example with multiple format specifiers
console.log("%cUser: %s, Age: %d, Score: %f", "color: cyan; font-weight: bold", "John", 25, 98.5);

// Error styling similar to browser dev tools
console.log("%c[ERROR]%c Something went wrong!", "color: white; background-color: red; font-weight: bold", "color: red");

// Success styling
console.log("%c✓ Success:%c Operation completed", "color: green; font-weight: bold", "color: green");

// Warning styling
console.log("%c⚠ Warning:%c Check your input", "color: orange; font-weight: bold", "color: orange");

// Debug styling
console.log("%c[DEBUG]%c Variable state:", "color: gray; font-style: italic", "color: gray");
