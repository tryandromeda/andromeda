function hexToUtf8(hex: string) {
  if (!hex) return "";
  const bytes = new Uint8Array(
    hex.match(/.{1,2}/g)!.map((b) => parseInt(b, 16)),
  );
  return new TextDecoder().decode(bytes);
}

const host = "example.org";
const port = 443;

console.log(`Connecting to ${host}:${port}...`);
const rid = await internal_tls_connect(host, port);
console.log(`Connected, rid=${rid}`);

const httpReq = `GET / HTTP/1.1\r\nHost: ${host}\r\nConnection: close\r\n\r\n`;
console.log(`Writing request (${httpReq.length} bytes)`);
await internal_tls_write(rid, httpReq);

let out = "";
while (true) {
  try {
    const chunk = await internal_tls_read(rid, 4096);
    if (!chunk || chunk.length === 0) break;
    out += chunk;
  } catch (e) {
    try {
      console.error("Read error (type):", typeof e);
      console.error(
        "Read error (toString):",
        e && typeof e.toString === "function" ? e.toString() : String(e),
      );
      console.error("Read error (json):", JSON.stringify(e));
    } catch (_) {
      console.error("Read error (raw):", e);
    }
    break;
  }
}

console.log("--- Response start (hex) ---");
console.log(out.substring(0, 2000));
console.log("--- Response end (hex) ---");
console.log("--- Response (decoded) ---");
try {
  console.log(hexToUtf8(out).substring(0, 2000));
} catch (e) {
  console.warn("Failed to decode response to UTF-8:", e);
}

try {
  const certHex = await internal_tls_get_peer_certificate(rid);
  console.log("Peer certificate (hex):", certHex);
  try {
    console.log(
      "Peer certificate (decoded snippet):",
      hexToUtf8(certHex).substring(0, 200),
    );
  } catch { /* ignore decode failures */ }
} catch (e) {
  console.warn(
    "No peer certificate or error:",
    e && (e.toString ? e.toString() : JSON.stringify(e)),
  );
}

await internal_tls_close(rid);
console.log("Closed.");
