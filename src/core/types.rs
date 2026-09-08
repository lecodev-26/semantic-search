//! Tipos públicos de la biblioteca

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Resultado de una búsqueda
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Ruta del archivo
    pub path: PathBuf,

    /// Contenido del archivo (o fragmento)
    pub content: String,

    /// Número de coincidencias
    pub matches: usize,

    /// Tamaño del archivo en bytes
    pub size: u64,

    /// Línea de inicio de la coincidencia
    pub line_start: Option<usize>,

    /// Línea de fin de la coincidencia
    pub line_end: Option<usize>,

    /// Puntuación de similitud (para búsqueda semántica)
    pub score: Option<f32>,

    /// Lista de palabras clave relevantes (para búsqueda semántica)
    pub keywords: Vec<String>,
}

/// Configuración del motor de búsqueda
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Ruta a buscar
    pub path: String,

    /// Extensiones a incluir (ej: ["rs", "py"])
    pub extensions: Option<Vec<String>>,

    /// Carpetas a ignorar
    pub ignore_dirs: Vec<String>,

    /// Búsqueda exacta (palabra completa)
    pub exact: bool,

    /// Ignorar mayúsculas/minúsculas
    pub ignore_case: bool,

    /// Modo verboso
    pub verbose: bool,

    /// Ignorar caché
    pub no_cache: bool,

    /// Mostrar resumen
    pub summary: bool,

    /// Tamaño máximo de archivo
    pub max_size: Option<String>,

    /// Patrón glob para ignorar archivos
    pub ignore_pattern: Option<String>,

    /// Modo interactivo
    pub interactive: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            path: ".".to_string(),
            extensions: None,
            ignore_dirs: vec![
                ".git".to_string(),
                "target".to_string(),
                "node_modules".to_string(),
            ],
            exact: false,
            ignore_case: false,
            verbose: false,
            no_cache: false,
            summary: true,
            max_size: None,
            ignore_pattern: None,
            interactive: false,
        }
    }
}
