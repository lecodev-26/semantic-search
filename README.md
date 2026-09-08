# 🔍 semantic-search

[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-0.8.0-brightgreen.svg)](https://github.com/lecodev-26/semantic-search/releases)
[![Termux](https://img.shields.io/badge/Termux-compatible-brightgreen.svg)](https://termux.com)

> **Buscador de código por SIGNIFICADO con TF-IDF, caché y filtros avanzados.**

---

## ✨ Características

- 🧠 **Búsqueda semántica con TF-IDF** (por significado, no por texto exacto)
- 📄 **Búsqueda por nombre de archivo** (`--file "main.rs"`)
- 🔢 **Contador de ocurrencias** (muestra cuántas coincidencias hay por archivo)
- 📊 **Resumen de resultados** (archivos, coincidencias totales y tiempo)
- 💾 **Caché inteligente** (búsquedas instantáneas después del primer indexado)
- 📁 **Indexado por extensión** (`--ext rs,md,toml` para indexar solo esos)
- 📏 **Filtro por tamaño** (`--max-size 100KB` para ignorar archivos grandes)
- 🔍 **Búsqueda por texto** con resaltado en color y números de línea
- 📱 **Compatible con Termux** (Android) y 100% local (sin internet)

---

## 🚀 Instalación

```bash
git clone https://github.com/lecodev-26/semantic-search
cd semantic-search
cargo build --release
sudo cp target/release/semantic-search /usr/local/bin/
```

---

📖 Uso

1. Indexar el proyecto (necesario para búsquedas rápidas)

```bash
# Indexar todo
semantic-search index --path .

# Indexar solo ciertas extensiones (ej: Rust y Markdown)
semantic-search index --path . --ext rs,md
```

2. Buscar por significado (semántica)

```bash
semantic-search search --query "función principal" --path . --semantic
```

3. Buscar por texto (grep mejorado)

```bash
# Búsqueda normal
semantic-search search --query "fn main" --path .

# Con filtro de tamaño (ignora archivos > 100KB)
semantic-search search --query "fn" --path . --max-size 100KB

# Búsqueda exacta
semantic-search search --query "main" --path . --exact
```

4. Buscar por nombre de archivo

```bash
semantic-search search --file "main.rs" --path .
```

5. Modo verbose (con progreso)

```bash
semantic-search search --query "fn" --path . --verbose
```

---

🗺️ Hoja de ruta
```maeckdown
Versión Novedades Estado
v0.1.0 Base: buscador simple con colores ✅
v0.2.0 Filtros, ignorar carpetas, búsqueda exacta ✅
v0.3.0 Caché - búsquedas instantáneas ✅
v0.4.0 Búsqueda semántica con TF-IDF ✅
v0.5.0 Búsqueda por nombre, contador de ocurrencias, resumen ✅
v0.6.0 Indexado por extensión, filtro por tamaño, tamaño visible ✅
v0.7.0 Ignorar por patrón, búsqueda en comprimidos  ✅
v0.8.0 Configuración global, alias y modo interactivo ✅
v1.0.0 Publicación en crates.io ⬜
```
---

📄 Licencia

MIT

---

👤 Autor

Manuel (@lecodev-26)
