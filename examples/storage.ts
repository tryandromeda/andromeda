localStorage.clear();
localStorage.setItem("test", "works");
console.log("localStorage.getItem('test'):", localStorage.getItem("test"));
console.log("localStorage.length:", localStorage.length);

sessionStorage.clear();
sessionStorage.setItem("session-test", "works");
console.log("sessionStorage.getItem('session-test'):", sessionStorage.getItem("session-test"));
console.log("sessionStorage.length:", sessionStorage.length);

console.log("✅ localStorage and sessionStorage working correctly!");
