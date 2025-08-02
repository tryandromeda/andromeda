const blob = new Blob(["Hello, ", "World!"], { type: "text/plain" });
console.log("   Blob size:", blob.size);
console.log("   Blob type:", blob.type);
const sliced = blob.slice(0, 5);
console.log("   Sliced size:", sliced.size);

const file = new File(["File content here"], "document.txt", {
    type: "text/plain",
    lastModified: Date.now() - 10000,
});
console.log("   File name:", file.name);
console.log("   File size:", file.size);
console.log("   File type:", file.type);
console.log("   File lastModified:", file.lastModified);

const fileSliced = file.slice(0, 4);
console.log("   File sliced size:", fileSliced.size);

const formData = new FormData();
formData.append("text", "hello world");
formData.append("file", file);
formData.append("blob", blob, "data.txt");

console.log("   FormData has text:", formData.has("text"));
console.log("   FormData get text:", formData.get("text"));
console.log("   FormData has file:", formData.has("file"));
console.log("   FormData has blob:", formData.has("blob"));

console.log("   FormData keys:");
for (const key of formData.keys()) {
    console.log("     -", key);
}
