const URLS = [
  "https://catfact.ninja/fact",
  "https://dog.ceo/api/breeds/image/random",
  "https://uselessfacts.jsph.pl/api/v2/facts/random",
  "https://api.adviceslip.com/advice",
];

async function fetchOne(url: string) {
  const started = Date.now();
  try {
    const res = await fetch(url);
    const text = await res.text();
    return {
      url,
      ok: res.ok,
      status: res.status,
      bytes: text.length,
      ms: Date.now() - started,
      preview: text.slice(0, 80),
    };
  } catch (err: any) {
    return {
      url,
      ok: false,
      error: (err && err.message) || String(err),
      ms: Date.now() - started,
    };
  }
}

async function main() {
  console.log("Fetching " + URLS.length + " URLs in parallel...");
  const t0 = Date.now();
  const results = await Promise.all(URLS.map(fetchOne));
  const t1 = Date.now();

  for (const r of results) {
    if ("error" in r) {
      console.log("  FAIL  [" + r.ms + "ms] " + r.url + " — " + r.error);
    } else {
      console.log(
        "  " + (r.ok ? "OK  " : "FAIL") +
          " [" + r.ms + "ms]  " + r.status +
          "  " + r.bytes + " bytes  " + r.url,
      );
      console.log("        preview: " + JSON.stringify(r.preview));
    }
  }
  console.log("Wall clock: " + (t1 - t0) + "ms");
}

main().catch((err) => {
  console.error("parallel example failed:", err);
  throw err;
});
