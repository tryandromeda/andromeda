interface CatFact {
  fact: string;
  length: number;
}

async function main() {
  const url = "https://catfact.ninja/fact";
  console.log("GET " + url);

  const res = await fetch(url);
  console.log("  status:", res.status, res.statusText);
  console.log("  server:", res.headers.get("server"));
  console.log("  content-type:", res.headers.get("content-type"));

  if (!res.ok) throw new Error("unexpected status " + res.status);

  const data = await res.json() as CatFact;
  console.log("");
  console.log("Random fact (" + data.length + " chars):");
  console.log("  " + data.fact);
}

main().catch((err) => {
  console.error("random_fact failed:", err);
  throw err;
});
