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

// Test invalid URL scheme with async/await
async function testInvalidScheme() {
  try {
    const response = await fetch("invalid://url");
    logTest("Invalid URL scheme", false, "Should have thrown an error");
  } catch (e: any) {
    logTest("Invalid URL scheme error handling", true, "Correctly threw error");
  }
}

testInvalidScheme();

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

// Test 17: Response object methods and properties
console.log("\n17. Response Object Tests");

async function testResponse() {
  try {
    console.log("Testing response from successful request...");
    const response = await fetch("https://example.com/test");
    
    // Test response properties
    logTest("Response has ok property", typeof response.ok === "boolean");
    logTest("Response has status property", typeof response.status === "number");
    logTest("Response has statusText property", typeof response.statusText === "string");
    logTest("Response has headers property", Array.isArray(response.headers) || typeof response.headers === "object");
    logTest("Response has url property", typeof response.url === "string");
    logTest("Response has type property", typeof response.type === "string");
    
    console.log(`  Status: ${response.status} ${response.statusText}`);
    console.log(`  OK: ${response.ok}`);
    console.log(`  Type: ${response.type}`);
    console.log(`  URL: ${response.url}`);
    
    // Test response methods
    logTest("Response has text method", typeof response.text === "function");
    logTest("Response has json method", typeof response.json === "function");
    logTest("Response has arrayBuffer method", typeof response.arrayBuffer === "function");
    logTest("Response has blob method", typeof response.blob === "function");
    
    // Test text() method
    try {
      const text = await response.text();
      logTest("Response.text() returns string", typeof text === "string");
      console.log(`  Response text: ${text.substring(0, 100)}${text.length > 100 ? '...' : ''}`);
    } catch (e: any) {
      logTest("Response.text() method", false, e.message);
    }
    
  } catch (e: any) {
    logTest("Response object creation", false, e.message);
  }
}

testResponse();

// Test 18: Error status codes
console.log("\n18. HTTP Status Code Tests");

async function testErrorStatuses() {
  // Test 404 Not Found
  try {
    console.log("Testing 404 response...");
    const response = await fetch("https://example.com/notfound");
    logTest("404 response received", response.status === 404);
    logTest("404 response not ok", !response.ok);
    console.log(`  Status: ${response.status} ${response.statusText}`);
    
    const errorData = await response.text();
    console.log(`  Error body: ${errorData}`);
  } catch (e: any) {
    logTest("404 status test", false, e.message);
  }
  
  // Test 500 Internal Server Error
  try {
    console.log("Testing 500 response...");
    const response = await fetch("https://example.com/error");
    logTest("500 response received", response.status === 500);
    logTest("500 response not ok", !response.ok);
    console.log(`  Status: ${response.status} ${response.statusText}`);
    
    const errorData = await response.text();
    console.log(`  Error body: ${errorData}`);
  } catch (e: any) {
    logTest("500 status test", false, e.message);
  }
}

testErrorStatuses();

// Test 19: JSON response parsing
console.log("\n19. JSON Response Tests");

async function testJsonResponse() {
  try {
    console.log("Testing JSON response parsing...");
    const response = await fetch("https://example.com/api/data");
    
    if (response.ok) {
      try {
        const jsonData = await response.json();
        logTest("JSON parsing successful", typeof jsonData === "object");
        console.log("  Parsed JSON:", JSON.stringify(jsonData));
      } catch (e: any) {
        logTest("JSON parsing", false, e.message);
      }
    } else {
      console.log(`  Response not OK: ${response.status}`);
    }
  } catch (e: any) {
    logTest("JSON response test", false, e.message);
  }
}

testJsonResponse();

// Test 20: Request with different content types
console.log("\n20. Content Type Tests");

async function testContentTypes() {
  // Test JSON request
  try {
    console.log("Testing JSON request...");
    const response = await fetch("https://example.com/api/post", {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        name: "test",
        value: 123,
        active: true
      })
    });
    
    logTest("JSON POST request", response.ok);
    console.log(`  Status: ${response.status}`);
  } catch (e: any) {
    logTest("JSON POST request", false, e.message);
  }
  
  // Test form data request
  try {
    console.log("Testing form data request...");
    const response = await fetch("https://example.com/api/form", {
      method: "POST",
      headers: {
        "Content-Type": "application/x-www-form-urlencoded"
      },
      body: "name=test&value=123&active=true"
    });
    
    logTest("Form data POST request", response.ok);
    console.log(`  Status: ${response.status}`);
  } catch (e: any) {
    logTest("Form data POST request", false, e.message);
  }
  
  // Test plain text request
  try {
    console.log("Testing plain text request...");
    const response = await fetch("https://example.com/api/text", {
      method: "POST",
      headers: {
        "Content-Type": "text/plain"
      },
      body: "This is plain text data for testing"
    });
    
    logTest("Plain text POST request", response.ok);
    console.log(`  Status: ${response.status}`);
  } catch (e: any) {
    logTest("Plain text POST request", false, e.message);
  }
}

testContentTypes();

// Test 21: ArrayBuffer and Blob responses
console.log("\n21. Binary Response Tests");

async function testBinaryResponses() {
  try {
    console.log("Testing arrayBuffer response...");
    const response = await fetch("https://example.com/binary-data");
    
    if (response.ok) {
      try {
        const buffer = await response.arrayBuffer();
        logTest("ArrayBuffer response", buffer instanceof ArrayBuffer);
        console.log(`  Buffer size: ${buffer.byteLength} bytes`);
      } catch (e: any) {
        logTest("ArrayBuffer response", false, e.message);
      }
    }
  } catch (e: any) {
    logTest("ArrayBuffer test", false, e.message);
  }
  
  try {
    console.log("Testing blob response...");
    const response = await fetch("https://example.com/blob-data");
    
    if (response.ok) {
      try {
        const blob = await response.blob();
        logTest("Blob response", blob instanceof Blob);
        console.log(`  Blob size: ${blob.size} bytes, type: ${blob.type}`);
      } catch (e: any) {
        logTest("Blob response", false, e.message);
      }
    }
  } catch (e: any) {
    logTest("Blob test", false, e.message);
  }
}

testBinaryResponses();

// Test 22: Network error simulation
console.log("\n22. Network Error Tests");

async function testNetworkErrors() {
  // Test invalid protocol
  try {
    await fetch("ftp://example.com/file.txt");
    logTest("Invalid protocol error", false, "Should have thrown an error");
  } catch (e: any) {
    logTest("Invalid protocol error handling", true, "Correctly threw error");
    console.log(`  Error: ${e.message}`);
  }
  
  // Test malformed URL
  try {
    await fetch("not-a-url");
    logTest("Malformed URL error", false, "Should have thrown an error");
  } catch (e: any) {
    logTest("Malformed URL error handling", true, "Correctly threw error");
    console.log(`  Error: ${e.message}`);
  }
}

testNetworkErrors();

// Test 23: Headers processing
console.log("\n23. Headers Tests");

async function testHeaders() {
  try {
    console.log("Testing request and response headers...");
    const response = await fetch("https://example.com/headers-test", {
      headers: {
        "User-Agent": "Andromeda-Test/1.0",
        "Accept": "application/json, text/plain, */*",
        "Accept-Language": "en-US,en;q=0.9",
        "Cache-Control": "no-cache"
      }
    });
    
    logTest("Headers request sent", response.ok);
    
    // Check response headers
    if (response.headers && Array.isArray(response.headers)) {
      logTest("Response has headers array", true);
      console.log("  Response headers:");
      response.headers.forEach(([name, value]: [string, string]) => {
        console.log(`    ${name}: ${value}`);
      });
    }
    
  } catch (e: any) {
    logTest("Headers test", false, e.message);
  }
}

testHeaders();

console.log("\n=== Advanced HttpFetch Implementation Tests Complete ===");
console.log("Note: These tests use mock responses from the httpNetworkFetch implementation.");
console.log("In a production environment, these would make real network requests.");