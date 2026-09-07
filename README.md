# 🔍 semantic-search

[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-0.5.0-brightgreen.svg)](https://github.com/lecodev-26/semantic-search/releases)
[![Termux](https://img.shields.io/badge/Termux-compatible-brightgreen.svg)](https://termux.com)

> **Buscador de código por SIGNIFICADO con TF-IDF, caché y búsqueda por nombre de archivo.**

---

## ✨ Características

- 🧠 **Búsqueda semántica con TF-IDF** (v0.4.0) - Busca por significado
- 📄 **Búsqueda por nombre de archivo** (v0.5.0) - Encuentra archivos por su nombre
- 🔢 **Contador de ocurrencias** (v0.5.0) - Muestra cuántas coincidencias por archivo
- 📊 **Resumen de resultados** (v0.5.0) - Archivos, coincidencias y tiempo
- 💾 **Caché inteligente** - Guarda palabras clave para búsquedas instantáneas
- 🔍 Búsqueda por texto (grep) con resaltado en color
- 📁 Filtro por extensiones y carpetas
- 📱 **Compatible con Termux** (Android)
- ⚡ **100% local** - No necesita internet

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

📄 Buscar por nombre de archivo (v0.5.0)

```bash
# Buscar archivo exacto
semantic-search search --file "main.rs" --path .

# Buscar ignorando mayúsculas
semantic-search search --file "Readme" --ignore-case --path .
```

🧠 Búsqueda semántica (TF-IDF)

```bash
# Indexar primero
semantic-search index --path .

# Buscar por significado
semantic-search search --query "función principal" --path . --semantic

# Con progreso
semantic-search search --query "validar email" --path . --semantic --verbose
```

🔍 Búsqueda por texto

```bash
# Búsqueda normal (con contador de ocurrencias)
semantic-search search --query "fn main" --path .

# Con resumen al final (activado por defecto)
semantic-search search --query "error" --path . --verbose

# Desactivar resumen
semantic-search search --query "error" --path . --no-summary

# Filtros
semantic-search search --query "Result" --ext rs --path src/
```

📊 Ejemplo de salida

```
🔍 Búsqueda por TEXTO
  Query: 'fn'
📄 Revisando 10 archivos...

./src/main.rs (7 coincidencias)
  34: fn main() -> anyhow::Result<()> {

📊 Resumen:
  • Archivos encontrados: 3
  • Coincidencias totales: 12
  • Tiempo: 0.00s
```

---

🗺️ Hoja de ruta
```text
Versión Novedades
v0.1.0 Buscador simple con colores
v0.2.0 Filtros, ignorar carpetas, búsqueda exacta
v0.3.0 ✅ Caché - búsquedas instantáneas
v0.4.0 ✅ Búsqueda semántica con TF-IDF
v0.5.0 ✅ Búsqueda por nombre, contador, resumen
v0.6.0 🚀 Próximamente
v1.0.0 Estable, publicación en crates.io
```
---

📁 Extensiones soportadas
```text
· Rust (.rs)
· Python (.py)
· JavaScript/TypeScript (.js, .ts)
· Go (.go)
· Java (.java)
· C/C++ (.c, .cpp, .h)
· Y más: .toml, .json, .yaml, .md, .sh, .bash, .css, .html, .xml, .sql, .rb, .php, .swift, .kt
```

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
