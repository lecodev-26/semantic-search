//! Tipos de error personalizados para la biblioteca

use thiserror::Error;

/// Errores específicos de semcode-search
#[derive(Error, Debug)]
pub enum SearchError {
    /// Error de E/S al leer/escribir archivos
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Error al parsear JSON de la caché
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error al parsear TOML de la configuración
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Error al parsear un tamaño (ej: "1MB")
    #[error("Invalid size format: {0}")]
    InvalidSize(String),

    /// Error al parsear una expresión regular
    #[error("Invalid regex: {0}")]
    InvalidRegex(#[from] regex::Error),

    /// Error de glob pattern
    #[error("Invalid glob pattern: {0}")]
    InvalidGlob(String),

    /// La caché no existe o está corrupta
    #[error("Cache not found or corrupted. Run `index` first.")]
    CacheNotFound,

    /// El alias no existe
    #[error("Alias '{0}' not found")]
    AliasNotFound(String),

    /// Error general
    #[error("{0}")]
    Other(String),
}

/// Resultado con tipos de error personalizados
pub type Result<T> = std::result::Result<T, SearchError>;
