//! Módulo Cache - Gestión de la caché

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheEntry {
    pub path: PathBuf,
    pub content: String,
    pub modified: u64,
    pub words: Vec<String>,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
    pub entries: HashMap<PathBuf, CacheEntry>,
    pub created: String,
    pub updated: String,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            created: chrono::Local::now().to_string(),
            updated: chrono::Local::now().to_string(),
        }
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let json = fs::read_to_string(path)?;
        let cache: Cache = serde_json::from_str(&json)?;
        Ok(cache)
    }

    pub fn is_valid(&self) -> bool {
        if let Ok(updated) = chrono::DateTime::parse_from_rfc3339(&self.updated) {
            let now = chrono::Local::now();
            let diff = now.signed_duration_since(updated.with_timezone(&chrono::Local));
            return diff.num_minutes() < 60;
        }
        false
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
