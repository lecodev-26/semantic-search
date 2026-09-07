# 🔍 semantic-search

[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-0.4.0-brightgreen.svg)](https://github.com/lecodev-26/semantic-search/releases)
[![Termux](https://img.shields.io/badge/Termux-compatible-brightgreen.svg)](https://termux.com)

> **Buscador de código por SIGNIFICADO con TF-IDF, caché y búsqueda avanzada.**

---

## ✨ Características

- 🧠 **Búsqueda semántica con TF-IDF** (v0.4.0) - Busca por significado, no por texto exacto
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

🧠 Búsqueda semántica (TF-IDF)

```bash
# Indexar el proyecto (genera caché de palabras clave)
semantic-search index --path .

# Buscar por SIGNIFICADO
semantic-search search --query "función principal" --path . --semantic

# Buscar con más detalles
semantic-search search --query "validar email" --path . --semantic --verbose
```

🔍 Búsqueda por texto (grep)

```bash
# Búsqueda normal
semantic-search search --query "fn main" --path .

# Filtros
semantic-search search --query "Result" --ext rs --path src/

# Buscar exacto
semantic-search search --query "main" --exact --path .

# Buscar con progreso
semantic-search search --query "error" --path . --verbose
```

📊 Ejemplo de salida (búsqueda semántica)

```
🧠 Búsqueda SEMÁNTICA (TF-IDF)
  Query: 'función principal'
📄 Revisando 45 archivos...
  Progreso: 45/45

src/main.rs [Similitud: 68.42%]
  fn main() -> anyhow::Result<()> {
      let cli = Cli::parse();
      match cli.command {

✅ Encontrados 3 archivos.
```

---

🗺️ Hoja de ruta
```marckdown
Versión Novedades
v0.1.0 Buscador simple con colores
v0.2.0 Filtros, ignorar carpetas, búsqueda exacta
v0.3.0 ✅ Caché - búsquedas instantáneas
v0.4.0 ✅ Búsqueda semántica con TF-IDF
v1.0.0 🚀 Estable, publicación en crates.io (próximamente)
```
---

📁 Extensiones soportadas
```marckdown
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
