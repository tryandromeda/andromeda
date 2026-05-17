async function main() {
  const payload = {
    favorite_axolotl: "Wooper",
    counts: [1, 2, 3],
    note: "sent from andromeda",
  };

  const res = await fetch("https://httpbin.org/post", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Andromeda-Demo": "post-json",
    },
    body: JSON.stringify(payload),
  });

  console.log("status:", res.status, res.statusText);

  if (!res.ok) throw new Error("expected 2xx, got " + res.status);

  const echoed = await res.json() as any;

  console.log("\nServer saw:");
  console.log("  url:           ", echoed.url);
  console.log("  raw data:      ", JSON.stringify(echoed.data));
  console.log("  parsed json:   ", JSON.stringify(echoed.json));
  console.log("  Content-Type:  ", echoed.headers["Content-Type"]);
  console.log("  X-Andromeda-Demo:", echoed.headers["X-Andromeda-Demo"]);

  if (echoed.json === null && echoed.data === "") {
    console.log("");
    console.log("  NOTE: server did not receive the JSON body (known gap).");
  }
}

main().catch((err) => {
  console.error("post_json failed:", err);
  throw err;
});
