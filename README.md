# 🔍 semantic-search

[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-0.2.0-brightgreen.svg)](https://github.com/lecodev-26/semantic-search/releases)
[![Termux](https://img.shields.io/badge/Termux-compatible-brightgreen.svg)](https://termux.com)

> **Buscador de código en terminal con búsqueda avanzada, filtros y resaltado.**

---

## ✨ Características

- 🔍 Búsqueda rápida en archivos de código
- 🎨 Resaltado en color de la palabra buscada
- 📊 Muestra número de línea exacto
- 📁 Filtro por extensiones (`--ext rs,py,js`)
- 🚫 Ignora carpetas automáticamente (`.git`, `target`, `node_modules`)
- 🎯 Búsqueda exacta (`--exact`)
- 🔤 Búsqueda ignorando mayúsculas (`--ignore-case`)
- ⏱️ Modo verbose con barra de progreso
- 📱 Compatible con Termux

---

## 🚀 Instalación

### Desde GitHub

```bash
git clone https://github.com/lecodev-26/semantic-search
cd semantic-search
cargo build --release
sudo cp target/release/semantic-search /usr/local/bin/
```

En Termux

```bash
pkg install rust
git clone https://github.com/lecodev-26/semantic-search
cd semantic-search
cargo build --release
cp target/release/semantic-search $PREFIX/bin/
```

---

📖 Uso

Comandos básicos

```bash
# Buscar texto en todo el proyecto
semantic-search search --query "fn main" --path .

# Buscar solo en archivos Rust
semantic-search search --query "Result" --ext rs --path src/

# Buscar ignorando mayúsculas
semantic-search search --query "hello" --ignore-case --path .

# Búsqueda exacta (palabra completa)
semantic-search search --query "main" --exact --path .

# Buscar con progreso
semantic-search search --query "error" --path . --verbose

# Ignorar carpetas personalizadas
semantic-search search --query "test" --ignore "target,node_modules,dist" --path .
```

Indexar (estadísticas)

```bash
# Mostrar estadísticas del proyecto
semantic-search index --path .

# Ignorar carpetas personalizadas
semantic-search index --path . --ignore "target,build"
```

Ejemplo de salida

```
🔍 Buscando: 'main' en src/

src/main.rs
  34: fn main() -> anyhow::Result<()> {

✅ Encontrados 1 archivos con coincidencias
```

---

📁 Extensiones soportadas

· Rust (.rs)
· Python (.py)
· JavaScript/TypeScript (.js, .ts)
· Go (.go)
· Java (.java)
· C/C++ (.c, .cpp, .h)
· Y más: .toml, .json, .yaml, .md, .sh, .bash, .css, .html, .xml, .sql, .rb, .php, .swift, .kt

---

🛠️ Desarrollo

```bash
# Clonar
git clone https://github.com/lecodev-26/semantic-search

# Compilar
cargo build

# Compilar optimizado
cargo build --release

# Ejecutar tests
cargo test
```

---

📄 Licencia

MIT

---

👤 Autor

Manuel (@lecodev-26)

---

⬆ Volver arriba

```

---
