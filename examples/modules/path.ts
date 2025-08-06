// Mock path module for import map testing
export function join(...paths: string[]): string {
  const result = paths.join("/");
  let cleaned = "";
  let prevWasSlash = false;
  for (let i = 0; i < result.length; i++) {
    const char = result[i];
    if (char === "/") {
      if (!prevWasSlash) {
        cleaned += char;
      }
      prevWasSlash = true;
    } else {
      cleaned += char;
      prevWasSlash = false;
    }
  }
  return cleaned;
}

export function dirname(path: string): string {
  const parts = path.split("/");
  return parts.slice(0, -1).join("/") || "/";
}

export function basename(path: string): string {
  const parts = path.split("/");
  return parts[parts.length - 1] || "";
}

console.log("📁 Path utilities loaded via import map!");
