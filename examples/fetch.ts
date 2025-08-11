console.log("=== Fetch API Test Suite ===\n");

// Helper function to log test results
const logTest = (testName: string, passed: boolean, details?: string) => {
  const status = passed ? "✓ PASS" : "✗ FAIL";
  console.log(`${status}: ${testName}`);
  if (details) console.log(`  Details: ${details}`);
};

// Test 1: Check if fetch is defined
console.log("1. Basic fetch availability");
logTest("fetch is defined", typeof fetch === "function");
logTest("fetch is a function", typeof fetch === "function");

// Test 2: Test different URL schemes
console.log("\n2. URL Scheme Tests");

// Test about:blank
try {
  const aboutBlankUrl = new URL("about:blank");
  fetch(aboutBlankUrl);
  logTest("about:blank URL", true);
} catch (e: any) {
  logTest("about:blank URL", false, e.message);
}

// Test data URL
try {
  const dataUrl = "data:text/plain;base64,SGVsbG8gV29ybGQh";
  fetch(dataUrl);
  logTest("data: URL", true);
} catch (e: any) {
  logTest("data: URL", false, e.message);
}

// Test HTTP URL
try {
  fetch("http://example.com/test");
  logTest("http: URL", true);
} catch (e: any) {
  logTest("http: URL", false, e.message);
}

// Test HTTPS URL
try {
  fetch("https://example.com/test");
  logTest("https: URL", true);
} catch (e: any) {
  logTest("https: URL", false, e.message);
}

// Test 3: Request with different methods
console.log("\n3. HTTP Method Tests");

const methods = ["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH"];
for (const method of methods) {
  try {
    // @ts-ignore
    fetch("https://example.com/test", { method });
    logTest(`${method} method`, true);
  } catch (e: any) {
    logTest(`${method} method`, false, e.message);
  }
}

// Test 4: Request with headers
console.log("\n4. Request Headers Tests");

try {
  // @ts-ignore
  fetch("https://example.com/test", {
    headers: {
      "Content-Type": "application/json",
      "Accept": "application/json",
      "X-Custom-Header": "test-value"
    }
  });
  logTest("Request with headers", true);
} catch (e: any) {
  logTest("Request with headers", false, e.message);
}

// Test 5: Request with body
console.log("\n5. Request Body Tests");

try {
  // @ts-ignore
  fetch("https://example.com/test", {
    method: "POST",
    body: JSON.stringify({ test: "data" })
  });
  logTest("POST with JSON body", true);
} catch (e: any) {
  logTest("POST with JSON body", false, e.message);
}

try {
  // @ts-ignore
  fetch("https://example.com/test", {
    method: "POST",
    body: "plain text body"
  });
  logTest("POST with text body", true);
} catch (e: any) {
  logTest("POST with text body", false, e.message);
}

// Test 6: Request modes
console.log("\n6. Request Mode Tests");

const modes = ["cors", "no-cors", "same-origin", "navigate"];
for (const mode of modes) {
  try {
    // @ts-ignore
    fetch("https://example.com/test", { mode });
    logTest(`${mode} mode`, true);
  } catch (e: any) {
    logTest(`${mode} mode`, false, e.message);
  }
}

// Test 7: Redirect handling
console.log("\n7. Redirect Tests");

const redirectModes = ["follow", "error", "manual"];
for (const redirect of redirectModes) {
  try {
    // @ts-ignore
    fetch("https://example.com/test", { redirect });
    logTest(`Redirect mode: ${redirect}`, true);
  } catch (e: any) {
    logTest(`Redirect mode: ${redirect}`, false, e.message);
  }
}

// Test 8: Invalid URL handling
console.log("\n8. Error Handling Tests");

try {
  fetch("invalid://url");
  logTest("Invalid URL scheme", false, "Should have thrown an error");
} catch (e: any) {
  logTest("Invalid URL scheme error handling", true, "Correctly threw error");
}

try {
  // @ts-ignore
  fetch(null);
  logTest("Null URL", false, "Should have thrown an error");
} catch (e: any) {
  logTest("Null URL error handling", true, "Correctly threw error");
}

try {
  // @ts-ignore
  fetch(undefined);
  logTest("Undefined URL", false, "Should have thrown an error");
} catch (e: any) {
  logTest("Undefined URL error handling", true, "Correctly threw error");
}

// Test 9: Request object creation
console.log("\n9. Request Object Tests");

try {
  // @ts-ignore
  const request = new Request("https://example.com/test", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ test: "data" })
  });
  fetch(request);
  logTest("Fetch with Request object", true);
} catch (e: any) {
  logTest("Fetch with Request object", false, e.message);
}

// Test 10: CORS preflight scenarios
console.log("\n10. CORS Preflight Tests");

try {
  // @ts-ignore
  fetch("https://api.example.com/data", {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
      "X-Custom-Header": "value"
    }
  });
  logTest("CORS preflight trigger (PUT with custom header)", true);
} catch (e: any) {
  logTest("CORS preflight trigger", false, e.message);
}

// Test 11: Blob URL handling
console.log("\n11. Blob URL Tests");

try {
  fetch("blob:https://example.com/550e8400-e29b-41d4-a716-446655440000");
  logTest("Blob URL handling", true);
} catch (e: any) {
  logTest("Blob URL handling", false, e.message);
}

// Test 12: File URL handling
console.log("\n12. File URL Tests");

try {
  fetch("file:///path/to/file.txt");
  logTest("File URL handling", true);
} catch (e: any) {
  logTest("File URL handling", false, e.message);
}

// Test 13: Range requests
console.log("\n13. Range Request Tests");

try {
  // @ts-ignore
  fetch("https://example.com/large-file.bin", {
    headers: {
      "Range": "bytes=200-1023"
    }
  });
  logTest("Range request", true);
} catch (e: any) {
  logTest("Range request", false, e.message);
}

// Test 14: Integrity metadata
console.log("\n14. Integrity Tests");

try {
  // @ts-ignore
  fetch("https://example.com/script.js", {
    integrity: "sha384-oqVuAfXRKap7fdgcCY5uykM6+R9GqQ8K/uxy9rx7HNQlGYl1kPzQho1wx4JwY8wC"
  });
  logTest("Request with integrity metadata", true);
} catch (e: any) {
  logTest("Request with integrity metadata", false, e.message);
}

// Test 15: Credentials mode
console.log("\n15. Credentials Tests");

const credentialsModes = ["omit", "same-origin", "include"];
for (const credentials of credentialsModes) {
  try {
    // @ts-ignore
    fetch("https://example.com/test", { credentials });
    logTest(`Credentials: ${credentials}`, true);
  } catch (e: any) {
    logTest(`Credentials: ${credentials}`, false, e.message);
  }
}

// Test 16: Cache modes
console.log("\n16. Cache Mode Tests");

const cacheModes = ["default", "no-store", "reload", "no-cache", "force-cache", "only-if-cached"];
for (const cache of cacheModes) {
  try {
    // @ts-ignore
    fetch("https://example.com/test", { cache });
    logTest(`Cache mode: ${cache}`, true);
  } catch (e: any) {
    logTest(`Cache mode: ${cache}`, false, e.message);
  }
}

// Test 17: Real API call (commented out to avoid network dependency)
console.log("\n17. Real API Test (JSONPlaceholder)");
console.log("Attempting real fetch to jsonplaceholder.typicode.com...");
fetch("https://jsonplaceholder.typicode.com/posts/1");

console.log("\n=== Test Suite Complete ===");