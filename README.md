# Andromeda

<a href="https://github.com/tryandromeda/andromeda"><img align="right" src="./assets/andromeda.svg" alt="Andromeda" width="150"/></a>

[![Discord Server](https://img.shields.io/discord/1264947585882259599.svg?logo=discord&style=flat-square)](https://discord.gg/tgjAnX2Ny3)

**A modern, fast, and secure JavaScript & TypeScript runtime** built from the
ground up in [Rust 🦀](https://www.rust-lang.org/) and powered by
[Nova Engine](https://trynova.dev/) and [Oxc](https://oxc.rs).

[Andromeda](https://github.com/tryandromeda/andromeda) provides **zero-config
TypeScript support**, **rich Web APIs**, and **native performance** - making it
perfect for scripts, utilities, and applications that need to run fast without
the complexity of traditional Node.js setups.

## Key Features

- **Zero-configuration TypeScript** - Run `.ts` files directly, no
  transpilation needed
- **Import Maps** - Modern module resolution with bare specifiers and CDN
  integration
**Built-in HTTP Server** - Create web servers and APIs with minimal configuration
- **GPU-Accelerated Canvas** - Hardware-accelerated 2D Canvas API with WGPU
  backend and PNG export
- **Web Crypto API** - Industry-standard cryptographic primitives
- **SQLite Support** - Built-in support for SQLite databases
- **File System Access** - Simple APIs for reading/writing files
- **Web Storage** - localStorage and sessionStorage APIs for data persistence
- **Native Performance** - Rust-powered execution with Nova's optimized JS
  engine
- **Developer Tools** - Interactive REPL, code formatter, single-file
  compilation, and performance profiling with hotpath
- **Performance Profiling** - Integrated hotpath profiler for identifying
  bottlenecks and optimizing runtime performance for andromeda development
- **Web Standards** - TextEncoder/Decoder, Performance API, and more
- **Extensible** - Modular architecture with optional features
- **Self-Updating** - Built-in upgrade system to stay current with latest
  releases
- **Shell Integration** - Auto-completion support for bash, zsh, fish, and
  PowerShell

## Standards & Compatibility

Andromeda aims to be **[WinterTC](https://wintertc.org/)** compliant, ensuring
interoperability and compatibility with the broader JavaScript ecosystem.
WinterTC provides a test suite for JavaScript engines to ensure they conform to
ECMAScript standards and common runtime behaviors.

> **Note:** ⚠️ Andromeda is in active development. While functional, it's not
> yet recommended for production use.

## Quick Start

### Installation

Platform specific instructions:

Linux/Mac:

```sh
curl -fsSL https://tryandromeda.dev/install.sh | bash
```

Windows (PowerShell):

```sh
irm -Uri "https://tryandromeda.dev/install.ps1" | Invoke-Expression
```

Windows (CMD):

```sh
curl -L -o install.bat https://tryandromeda.dev/install.bat && install.bat
```

Install Andromeda using Cargo:

```sh
cargo install --git https://github.com/tryandromeda/andromeda andromeda
```

Install Andromeda using winget:

```sh
winget install --id Andromeda.Andromeda
```

### Running Code

Execute JavaScript or TypeScript files directly:

```sh
# Run a TypeScript file (no compilation needed!)
andromeda run hello.ts

# Run multiple files
andromeda run script1.js script2.ts

# Run with verbose output
andromeda run --verbose my-script.ts
```

### Example: Hello World with Canvas

```ts
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
canvas.saveAsPng("output.png");
console.log("Image saved to output.png");
```

### Example: Standard Library

```ts
import { Queue } from "https://tryandromeda.dev/std/collections/mod.ts";
import { flatten } from "https://tryandromeda.dev/std/data/mod.ts";

const queue = new Queue();
queue.enqueue("first");
queue.enqueue("second");
console.log(queue.dequeue());
console.log(queue.size);

console.log(flatten([[1, 2], [3, [4, 5]]], 1));
console.log(flatten([[1, 2], [3, [4, 5]]], 2));
```

## Core APIs

### File System

```ts
// Read and write files synchronously
const content = Andromeda.readTextFileSync("input.txt");
Andromeda.writeTextFileSync("output.txt", content.toUpperCase());

// and asynchronously
const contentAsync = await Andromeda.readTextFile("input.txt");
await Andromeda.writeTextFile("output.txt", contentAsync.toUpperCase());

// Access environment variables
const home = Andromeda.env.get("HOME");
Andromeda.env.set("MY_VAR", "value");
```

### Canvas & Graphics

```ts
// Create graphics programmatically
const canvas = new OffscreenCanvas(800, 600);
const ctx = canvas.getContext("2d")!;

// Draw with full Canvas 2D API
ctx.fillStyle = "linear-gradient(45deg, #f093fb, #f5576c)";
ctx.fillRect(0, 0, 800, 600);

// Export to PNG
canvas.saveAsPng("artwork.png");
```

### Web Storage

```ts
// localStorage and sessionStorage APIs
localStorage.setItem("user-preference", "dark-mode");
const preference = localStorage.getItem("user-preference");
console.log("Stored items:", localStorage.length);

// Session storage for temporary data
sessionStorage.setItem("session-id", crypto.randomUUID());
const sessionId = sessionStorage.getItem("session-id");
```

### Cryptography

```ts
// Generate secure random values
const uuid = crypto.randomUUID();
const randomBytes = crypto.getRandomValues(new Uint8Array(32));

// Hash data
const data = new TextEncoder().encode("Hello, World!");
const hash = await crypto.subtle.digest("SHA-256", data);
```

### Performance Monitoring

```ts
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

### Database Operations

```ts
const db = new Database(":memory:");

const stmt = db.prepare("INSERT INTO users (name, email) VALUES (?, ?)");
stmt.run("Alice", "alice@example.com");

const users = db.prepare("SELECT * FROM users").all();
console.log(users);

db.close();
```

## Developer Experience

### Interactive REPL

Andromeda includes a powerful REPL with enhanced developer experience:

```sh
# Start the interactive REPL
andromeda repl
```

**REPL Features:**

- **Advanced Syntax Highlighting** - TypeScript-aware coloring with keyword
  recognition
- **Smart Multiline Input** - Automatic detection of incomplete syntax
  (functions, objects, etc.)
- **Performance Metrics** - Execution timing for every evaluation
- **Command History** - Navigate through previous commands with arrow keys
- **Built-in Commands** - `help`, `history`, `clear`, `gc`, `exit`
- **Auto-completion** - Context-aware suggestions for JavaScript/TypeScript

### Code Formatting

Format TypeScript and JavaScript files with the built-in formatter:

```sh
# Format specific files
andromeda fmt script.ts utils.js

# Format entire directories
andromeda fmt src/ examples/

# Format current directory
andromeda fmt
```

### Single-File Compilation

Compile your scripts into standalone executables:

```sh
# Create a single-file executable
andromeda compile my-script.ts my-app.exe

# Run the compiled executable directly
./my-app.exe
```

### Language Server Protocol (LSP)

Andromeda includes a built-in Language Server that provides real-time
diagnostics and linting capabilities for JavaScript and TypeScript files in your
editor:

```sh
# Start the Language Server (typically called by your editor)
andromeda lsp
```

**LSP Features:**

- **Real-time Diagnostics** - Live error reporting as you type
- **Comprehensive Linting** - 5 built-in rules for code quality:
  - Empty function detection
  - Empty statement detection
  - Variable usage validation
  - Unreachable code detection
  - Invalid syntax highlighting
- **Multi-file Support** - Workspace-wide analysis
- **Rich Error Messages** - Detailed explanations with code context
- **Editor Integration** - Works with VS Code, Neovim, and other LSP-compatible
  editors

Configure your editor to use `andromeda lsp` as the language server for
JavaScript and TypeScript files to get instant feedback on code quality.

### Shell Integration

Generate completion scripts for your shell:

```sh
# Auto-detect shell and generate completions
andromeda completions

# Generate for specific shells
andromeda completions bash > /etc/bash_completion.d/andromeda
andromeda completions zsh > ~/.zsh/completions/_andromeda
andromeda completions fish > ~/.config/fish/completions/andromeda.fish
andromeda completions powershell > $PROFILE/andromeda.ps1
```

### Self-Updating

Keep Andromeda up to date with the built-in upgrade system:

```sh
# Upgrade to latest version
andromeda upgrade

# Force reinstall current version
andromeda upgrade --force

# Upgrade to specific version
andromeda upgrade --version 0.1.0-draft-49

# Preview what would be upgraded
andromeda upgrade --dry-run
```

### Task System

Andromeda includes a powerful task system inspired by Deno, allowing you to define and run custom scripts and workflows directly from your configuration file.

#### Defining Tasks

Tasks are defined in your `andromeda.json`, `andromeda.toml`, or `andromeda.yaml` configuration file:

```json
{
  "tasks": {
    "dev": "andromeda run src/main.ts",
    "build": "echo Building project...",
    "test": "andromeda run tests/main.ts",
  }
}
```

#### Running Tasks

```sh
# List all available tasks
andromeda task

# Run a specific task
andromeda task dev
andromeda task build
andromeda task test
```

## Architecture & Extensions

Andromeda is built with a modular architecture, allowing features to be enabled
or disabled as needed:

### Runtime Extensions

| Extension         | Description                   | APIs Provided                                                                  |
| ----------------- | ----------------------------- | ------------------------------------------------------------------------------ |
| **Canvas**        | GPU-accelerated 2D graphics   | `OffscreenCanvas`, `CanvasRenderingContext2D`, `ImageBitmap` with WGPU backend |
| **Crypto**        | Web Crypto API implementation | `crypto.subtle`, `crypto.randomUUID()`, `crypto.getRandomValues()`             |
| **Console**       | Enhanced console output       | `console.log()`, `console.error()`, `console.warn()`                           |
| **Fetch**         | HTTP client capabilities      | `fetch()`, `Request`, `Response`, `Headers`                                    |
| **File System**   | File I/O operations           | `Andromeda.readTextFileSync()`, `Andromeda.writeTextFileSync()`, directory ops |
| **Local Storage** | Web storage APIs              | `localStorage`, `sessionStorage` with persistence                              |
| **Process**       | System interaction            | `Andromeda.args`, `Andromeda.env`, `Andromeda.exit()`                          |
| **SQLite**        | Database operations           | `Database`, prepared statements, transactions                                  |
| **Time**          | Timing utilities              | `performance.now()`, `setTimeout()`, `setInterval()`, `Andromeda.sleep()`      |
| **URL**           | URL parsing and manipulation  | `URL`, `URLSearchParams`                                                       |
| **Web**           | Web standards                 | `TextEncoder`, `TextDecoder`, `navigator`, `queueMicrotask()`                  |

## Crates

| Crate                             | Description                                    |
| --------------------------------- | ---------------------------------------------- |
| [**andromeda**](/cli)             | Command-line interface and developer tools     |
| [**andromeda-core**](/core)       | Core runtime engine and JavaScript execution   |
| [**andromeda-runtime**](/runtime) | Runtime extensions and Web API implementations |

## Contributing

Andromeda is an open-source project and welcomes contributions! Whether you're
interested in:

- **Bug fixes** - Help improve stability
- **New features** - Add runtime capabilities
- **Documentation** - Improve guides and examples
- **Testing** - Expand test coverage

Join our [Discord community](https://discord.gg/tgjAnX2Ny3) to discuss ideas and
get involved!

## License

[Mozilla Public License Version 2.0](./LICENSE.md)
