ls -la | grep Cargo.lock
# 🔍 semantic-search

[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Termux](https://img.shields.io/badge/Termux-compatible-brightgreen.svg)](https://termux.com)

> **Buscador de código en terminal con resaltado de sintaxis.**
> Corre en Termux, Linux, macOS y Windows.

---

## ✨ Características

- 🔍 Búsqueda rápida en archivos de código
- 🎨 **Resaltado en color** de la palabra buscada
- 📊 Muestra **número de línea** exacto
- 📁 Soporta múltiples extensiones (Rust, Python, JS, Go, Java, C/C++, etc.)
- ⏱️ Modo **verbose** con barra de progreso
- 📱 **Compatible con Termux** (móvil)
- ⚡ Compilado en Rust: rápido y seguro

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

```bash
# Buscar texto en archivos
semantic-search search --query "fn main" --path src/

# Buscar con progreso
semantic-search search --query "Result" --path src/ --verbose

# Mostrar estadísticas
semantic-search index --path .
```

Ejemplo de salida

```
🔍 Buscando: 'main' en src/

src/main.rs
  34: fn main() -> anyhow::Result<()> {

✅ Encontrados 1 archivos con coincidencias
```

---

🛠️ Desarrollo

```bash
# Clonar
git clone https://github.com/lecodev-26/semantic-search

# Compilar en desarrollo
cargo build

# Compilar optimizado
cargo build --release

# Ejecutar
cargo run -- search --query "texto" --path .
```

---

📁 Estructura

```
semantic-search/
├── Cargo.toml          # Dependencias
├── Cargo.lock          # Versiones bloqueadas
├── README.md           # Documentación
├── .gitignore          # Archivos ignorados
└── src/
    └── main.rs         # Código principal (único archivo)
```

---

🧪 Tests (próximamente)

El proyecto está en desarrollo activo. Próximamente se añadirán tests unitarios y de integración.

---

📄 Licencia

MIT

---

⭐ Contribuciones

¡Las contribuciones son bienvenidas! Abre un issue o PR.

---

👤 Autor

Manuel (@lecodev-26)
