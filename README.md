# 🔍 semcode-search

[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-1.0.0-brightgreen.svg)](https://github.com/lecodev-26/semcode-search/releases)
[![Termux](https://img.shields.io/badge/Termux-compatible-brightgreen.svg)](https://termux.com)

> **Fast semantic code search CLI with TF-IDF ranking, caching, and advanced filtering.**

---

## ✨ Features

- 🔍 **Text search** with color highlighting
- 🧠 **Semantic search** using TF-IDF ranking
- 💾 **Intelligent cache** for instant searches
- 📁 **Advanced filtering** by extension, size, and glob patterns
- 📄 **Filename search**
- ⚙️ **Global configuration**
- 🏷️ **Search aliases** (save, list, remove, run)
- 🎮 **Interactive mode** to navigate results
- ⚡ **Parallel indexing** with rayon
- 📱 **Termux compatible** (Android)
- 📚 **Public API** for use as a library

---

## 🚀 Installation

```bash
# From crates.io (soon)
cargo install semcode-search

# From GitHub
git clone https://github.com/lecodev-26/semcode-search
cd semcode-search
cargo build --release
sudo cp target/release/semcode-search /usr/local/bin/
```

---

📖 Usage

Basic commands

```bash
# Index a project
semcode-search index --path .

# Search by text
semcode-search search --query "fn main" --path .

# Search semantically (TF-IDF)
semcode-search search --query "authentication middleware" --path . --semantic

# Search by filename
semcode-search search --file "main.rs" --path .

# Search with verbose output
semcode-search search --query "error" --path . --verbose
```

Alias management

```bash
# Save a search alias
semcode-search alias save find-main "main" -- --ext rs --path .

# List all aliases
semcode-search alias list

# Run an alias
semcode-search alias run find-main

# Remove an alias
semcode-search alias remove find-main
```

Configuration

```bash
# Initialize configuration
semcode-search init

# Configuration file is saved at:
# ~/.config/semcode-search/config.toml (Linux)
# ~/Library/Application Support/semcode-search/config.toml (macOS)
```

Interactive mode

```bash
semcode-search search --query "fn" --path . --interactive
```

---

🏗️ Architecture

```
┌─────────────────────────────────────────────────────┐
│                    CLI (clap)                       │
├─────────────────────────────────────────────────────┤
│  • Commands: init, alias, index, search            │
│  • Subcommands: save, list, remove, run            │
└─────────────────────┬───────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────┐
│                 Core Library                        │
├─────────────────────────────────────────────────────┤
│  • SearchEngine (public API)                       │
│  • TF-IDF ranking                                  │
│  • Parallel indexing (rayon)                       │
│  • Cache management                                │
└─────────────────────────────────────────────────────┘
```

---

🗺️ Roadmap

Version Features Status
v0.1.0 Basic search with colors ✅
v0.2.0 Filters, ignore directories, exact search ✅
v0.3.0 Cache - instant searches ✅
v0.4.0 Semantic search with TF-IDF ✅
v0.5.0 Filename search, occurrence counter, summary ✅
v0.6.0 Extension indexing, size filtering ✅
v0.7.0 Glob pattern ignore, compressed file search ✅
v0.8.0 Global config, aliases, interactive mode ✅
v0.9.0 Refactoring, parallel indexing ✅
v1.0.0 ✅ Stable release with public API ✅

---

📁 Supported extensions

· Rust (.rs)
· Python (.py)
· JavaScript/TypeScript (.js, .ts)
· Go (.go)
· Java (.java)
· C/C++ (.c, .cpp, .h)
· And more: .toml, .json, .yaml, .md, .sh, .bash, .css, .html, .xml, .sql, .rb, .php, .swift, .kt

---

🛠️ Development

```bash
# Clone
git clone https://github.com/lecodev-26/semcode-search
cd semcode-search

# Build
cargo build

# Build optimized
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

---

📄 License

MIT

---

👤 Author

Manuel (@lecodev-26)
