# Andromeda 🌌

<a href="https://github.com/load1n9/andromeda"><img align="right" src="./assets/andromeda.svg" alt="Andromeda" width="150"/></a>

[![Discord Server](https://img.shields.io/discord/1264947585882259599.svg?logo=discord&style=flat-square)](https://discord.gg/tgjAnX2Ny3)

The simplest JavaScript and TypeScript runtime, fully written in [Rust 🦀](https://www.rust-lang.org/) and powered [Nova](https://trynova.dev/).

> Note: ⚠️ This project is still in early stages and is not suitable for serious use.

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

## Crates

| Crate | Description |
|-----------|-------------|
| [andromeda](/cli)| Contains the Executable Command Line Interface (CLI) code |
| [andromeda-core](/core)| Contains the core runtime code |
| [andromeda-runtime](/runtime)| Contains the runtime code |

[Mozilla Public License Version 2.0](./LICENSE.md)
