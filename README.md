# Andromeda 🌌

<a href="https://github.com/load1n9/andromeda"><img align="right" src="./assets/andromeda.svg" alt="Andromeda" width="150"/></a>

[![Discord Server](https://img.shields.io/discord/1264947585882259599.svg?logo=discord&style=flat-square)](https://discord.gg/tgjAnX2Ny3)

The simplest JavaScript and TypeScript runtime, fully written in
[Rust 🦀](https://www.rust-lang.org/) and powered [Nova](https://trynova.dev/).

> Note: ⚠️ This project is still in early stages and is not suitable for serious
> use.

## Installation

The easiest way to install Andromeda is to have
[Cargo](https://doc.rust-lang.org/cargo/) installed and run the following
command:

```bash
cargo install --git https://github.com/tryandromeda/andromeda
```

## Getting Started

To get started with Andromeda, follow these steps:

1. **Clone the Repository:**

   ```bash
   git clone https://github.com/tryandromeda/andromeda
   cd andromeda
   ```

2. **Install**

   ```bash
   cargo install --path ./cli
   ```

---

## Usage

To run a JavaScript or TypeScript file, use the following command:

```bash
andromeda run <file>
```

### Interactive REPL

Andromeda includes an interactive REPL (Read-Eval-Print Loop) for testing JavaScript or TypeScript code quickly:

```bash
# Start the REPL
andromeda repl

# REPL with debugging options
andromeda repl --print-internals --expose-internals
```

**REPL Commands:**

- Type JavaScript code and press Enter to evaluate
- Type `exit` to quit
- Type `gc` to trigger garbage collection
- Press Ctrl+C to exit

**REPL Options:**

- `--expose-internals`: Expose Nova internal APIs for debugging
- `--print-internals`: Print internal debugging information  
- `--disable-gc`: Disable garbage collection

## 🎨 Enhanced REPL Features

The Andromeda REPL provides a beautiful and powerful development experience:

### ✨ Smart Multiline Input
- **Automatic Detection**: Incomplete JavaScript syntax automatically triggers multiline mode
- **Visual Feedback**: Clear continuation prompts with line numbers
- **Manual Control**: Force completion with `;;;` or cancel with Ctrl+C
- **Syntax Awareness**: Handles functions, objects, arrays, and control structures

### 🎯 Interactive Commands
- `help` - Show available commands and multiline tips
- `history` - View command history (last 20 entries)
- `clear` - Clear the screen
- `gc` - Manual garbage collection with progress feedback
- `exit`/`quit` - Graceful exit

### 🌈 Visual Enhancements  
- **Type-aware Output**: Different colors for strings, numbers, booleans, etc.
- **Execution Timing**: Performance metrics for every evaluation
- **Beautiful Themes**: Consistent color scheme throughout
- **Smart Prompts**: Dynamic prompts showing evaluation count
- **Startup Tips**: Random JavaScript examples to get you started

### Example Multiline Usage:
```javascript
js [1] function fibonacci(n) {
...[2]   if (n <= 1) return n;
...[3]   return fibonacci(n-1) + fibonacci(n-2);
...[4] }
← function fibonacci(n) { ... } (function)
  ⏱️ 3ms

js [2] fibonacci(10)
← 55 (number)
  ⏱️ 1ms
```

## Crates

| Crate                         | Description                                               |
| ----------------------------- | --------------------------------------------------------- |
| [andromeda](/cli)             | Contains the Executable Command Line Interface (CLI) code |
| [andromeda-core](/core)       | Contains the core runtime code                            |
| [andromeda-runtime](/runtime) | Contains the runtime code                                 |

[Mozilla Public License Version 2.0](./LICENSE.md)
