# Andromeda 🌌

<a href="https://github.com/load1n9/andromeda"><img align="right" src="./assets/andromeda.svg" alt="Andromeda" width="150"/></a>

[![Discord Server](https://img.shields.io/discord/1264947585882259599.svg?logo=discord&style=flat-square)](https://discord.gg/tgjAnX2Ny3)

**A modern, fast, and secure JavaScript & TypeScript runtime** built from the ground up in [Rust 🦀](https://www.rust-lang.org/) and powered by [Nova Engine](https://trynova.dev/).

Andromeda provides **zero-config TypeScript support**, **rich Web APIs**, and **native performance** - making it perfect for scripts, utilities, and applications that need to run fast without the complexity of traditional Node.js setups.

## ✨ Key Features

- 🚀 **Zero-configuration TypeScript** - Run `.ts` files directly, no transpilation needed
- 🎨 **Canvas & Graphics** - Full 2D Canvas API with PNG export capabilities  
- 🔐 **Web Crypto API** - Industry-standard cryptographic primitives
- 📁 **File System Access** - Simple APIs for reading/writing files
- ⚡ **Native Performance** - Rust-powered execution with Nova's optimized JS engine
- 🛠️ **Developer Tools** - Interactive REPL, code formatter, and single-file compilation
- 🌐 **Web Standards** - TextEncoder/Decoder, Performance API, and more
- 🔧 **Extensible** - Modular architecture with optional features

## 🎯 Standards & Compatibility

Andromeda aims to be **[WinterTC](https://wintertc.org/)** compliant, ensuring interoperability and compatibility with the broader JavaScript ecosystem. WinterTC provides a test suite for JavaScript engines to ensure they conform to ECMAScript standards and common runtime behaviors.

> **Note:** ⚠️ Andromeda is in active development. While functional, it's not yet recommended for production use.

## 🚀 Quick Start

### Installation

Install Andromeda using Cargo:

```bash
cargo install --git https://github.com/tryandromeda/andromeda
```

### Running Code

Execute JavaScript or TypeScript files directly:

```bash
# Run a TypeScript file (no compilation needed!)
andromeda run hello.ts

# Run multiple files
andromeda run script1.js script2.ts

# Run with verbose output
andromeda run --verbose my-script.ts
```

### Example: Hello World with Canvas

```typescript
// Create a simple drawing
const canvas = new OffscreenCanvas(400, 300);
const ctx = canvas.getContext("2d")!;

ctx.fillStyle = "#ff6b6b";
ctx.fillRect(50, 50, 100, 100);

ctx.fillStyle = "#4ecdc4";
ctx.beginPath();
ctx.arc(200, 150, 50, 0, Math.PI * 2);
ctx.fill();

// Save as PNG
canvas.render();
canvas.saveAsPng("output.png");
console.log("Image saved to output.png");
```

## 🛠️ Core APIs

### File System

```typescript
// Read and write files synchronously
const content = Andromeda.readTextFileSync("input.txt");
Andromeda.writeTextFileSync("output.txt", content.toUpperCase());

// Access environment variables
const home = Andromeda.env.get("HOME");
Andromeda.env.set("MY_VAR", "value");
```

### Canvas & Graphics

```typescript
// Create graphics programmatically
const canvas = new OffscreenCanvas(800, 600);
const ctx = canvas.getContext("2d")!;

// Draw with full Canvas 2D API
ctx.fillStyle = "linear-gradient(45deg, #f093fb, #f5576c)";
ctx.fillRect(0, 0, 800, 600);

// Export to PNG
canvas.saveAsPng("artwork.png");
```

### Cryptography

```typescript
// Generate secure random values
const uuid = crypto.randomUUID();
const randomBytes = crypto.getRandomValues(new Uint8Array(32));

// Hash data
const data = new TextEncoder().encode("Hello, World!");
const hash = await crypto.subtle.digest("SHA-256", data);
```

### Performance Monitoring

```typescript
// High-precision timing
const start = performance.now();
await someAsyncOperation();
const duration = performance.now() - start;

// Performance marks and measures
performance.mark("operation-start");
await doWork();
performance.mark("operation-end");
performance.measure("total-time", "operation-start", "operation-end");
```

## 🎯 Developer Experience

### Interactive REPL

Andromeda includes a powerful REPL with enhanced developer experience:

```bash
# Start the interactive REPL
andromeda repl

# REPL with debugging options
andromeda repl --print-internals --expose-internals --disable-gc
```

**✨ REPL Features:**

- **Smart Multiline Input** - Automatic detection of incomplete syntax
- **Syntax Highlighting** - Type-aware output coloring  
- **Performance Metrics** - Execution timing for every evaluation
- **Command History** - Navigate through previous commands
- **Built-in Commands** - `help`, `history`, `clear`, `gc`, `exit`

### Code Formatting

Format TypeScript and JavaScript files with the built-in formatter:

```bash
# Format specific files
andromeda fmt script.ts utils.js

# Format entire directories
andromeda fmt src/ examples/

# Format current directory
andromeda fmt
```

### Single-File Compilation

Compile your scripts into standalone executables:

```bash
# Create a single-file executable
andromeda compile my-script.ts my-app.exe

# Run the compiled executable directly
./my-app.exe
```

## 🏗️ Architecture & Extensions

Andromeda is built with a modular architecture, allowing features to be enabled or disabled as needed:

### Runtime Extensions

| Extension | Description | APIs Provided |
|-----------|-------------|---------------|
| **Canvas** | 2D graphics rendering | `OffscreenCanvas`, `CanvasRenderingContext2D`, `ImageBitmap` |
| **Crypto** | Web Crypto API implementation | `crypto.subtle`, `crypto.randomUUID()`, `crypto.getRandomValues()` |
| **Console** | Enhanced console output | `console.log()`, `console.error()`, `console.warn()` |
| **Fetch** | HTTP client capabilities | `fetch()`, `Request`, `Response`, `Headers` |
| **File System** | File I/O operations | `Andromeda.readTextFileSync()`, `Andromeda.writeTextFileSync()` |
| **Process** | System interaction | `Andromeda.args`, `Andromeda.env`, `Andromeda.exit()` |
| **Time** | Timing utilities | `performance.now()`, `Andromeda.sleep()` |
| **URL** | URL parsing and manipulation | `URL`, `URLSearchParams` |
| **Web** | Web standards | `TextEncoder`, `TextDecoder`, `prompt()`, `confirm()` |

## Crates

| Crate | Description |
|-------|-------------|
| [**andromeda**](/cli) | Command-line interface and developer tools |
| [**andromeda-core**](/core) | Core runtime engine and JavaScript execution |
| [**andromeda-runtime**](/runtime) | Runtime extensions and Web API implementations |

## 🤝 Contributing

Andromeda is an open-source project and welcomes contributions! Whether you're interested in:

- 🐛 **Bug fixes** - Help improve stability
- ✨ **New features** - Add runtime capabilities  
- 📚 **Documentation** - Improve guides and examples
- 🧪 **Testing** - Expand test coverage

Join our [Discord community](https://discord.gg/tgjAnX2Ny3) to discuss ideas and get involved!

## 📜 License

[Mozilla Public License Version 2.0](./LICENSE.md)
