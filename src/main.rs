use clap::{Parser, Subcommand};
use colored::*;
use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "semantic-search")]
#[command(version = "0.3.0")]
#[command(about = "🔍 Buscador de código con índice en caché", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Indexar archivos y guardar caché
    Index {
        #[arg(short, long)]
        path: String,

        #[arg(short, long, default_value = ".git,target,node_modules,dist,build")]
        ignore: String,

        /// Forzar re-indexado completo
        #[arg(short, long)]
        force: bool,
    },

    /// Buscar texto en archivos (usa caché si existe)
    Search {
        #[arg(short, long)]
        query: String,

        #[arg(short, long, default_value = ".")]
        path: String,

        #[arg(short = 'e', long, value_name = "EXT")]
        ext: Option<String>,

        #[arg(short = 'i', long, default_value = ".git,target,node_modules,dist,build")]
        ignore: String,

        #[arg(long)]
        exact: bool,

        #[arg(long)]
        ignore_case: bool,

        #[arg(short, long)]
        verbose: bool,

        /// Ignorar caché y escanear de nuevo
        #[arg(long)]
        no_cache: bool,

        /// Actualizar caché antes de buscar
        #[arg(long)]
        update: bool,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CacheEntry {
    path: PathBuf,
    content: String,
    modified: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cache {
    entries: HashMap<PathBuf, CacheEntry>,
    created: String,
    updated: String,
}

impl Cache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            created: chrono::Local::now().to_string(),
            updated: chrono::Local::now().to_string(),
        }
    }

    fn save(&self, path: &Path) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    fn load(path: &Path) -> anyhow::Result<Self> {
        let json = fs::read_to_string(path)?;
        let cache: Cache = serde_json::from_str(&json)?;
        Ok(cache)
    }

    fn is_valid(&self) -> bool {
        // Si el caché tiene menos de 1 hora, es válido
        if let Ok(updated) = chrono::DateTime::parse_from_rfc3339(&self.updated) {
            let now = chrono::Local::now();
            let diff = now.signed_duration_since(updated.with_timezone(&chrono::Local));
            return diff.num_minutes() < 60;
        }
        false
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Index { path, ignore, force } => {
            let cache_path = Path::new(".semantic-index.json");
            
            if force {
                println!("{} Forzando re-indexado...", "🔄".yellow());
                if cache_path.exists() {
                    fs::remove_file(cache_path)?;
                }
            }

            println!("{} Indexando: {}", "📁".green(), path);
            let mut count = 0;
            let mut extensions = std::collections::HashSet::new();
            let mut cache = Cache::new();

            let ignore_dirs: Vec<&str> = ignore.split(',').collect();

            let walker = WalkBuilder::new(&path)
                .git_ignore(true)
                .follow_links(false)
                .build();

            for result in walker {
                let entry = result?;
                let path = entry.path();

                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if ignore_dirs.contains(&name) {
                            continue;
                        }
                    }
                }

                if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_string();
                        let exts = [
                            "rs", "py", "js", "ts", "go", "java", "c", "cpp", "h",
                            "toml", "json", "txt", "md", "sh", "bash", "yaml", "yml",
                            "css", "html", "xml", "sql", "rb", "php", "swift", "kt",
                        ];
                        if exts.contains(&ext_str.as_str()) {
                            count += 1;
                            extensions.insert(ext_str);

                            // Guardar en caché
                            if let Ok(content) = fs::read_to_string(path) {
                                let metadata = fs::metadata(path)?;
                                let modified = metadata
                                    .modified()
                                    .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                                    .unwrap_or(0);

                                cache.entries.insert(
                                    path.to_path_buf(),
                                    CacheEntry {
                                        path: path.to_path_buf(),
                                        content,
                                        modified,
                                    },
                                );
                            }
                        }
                    }
                }
            }

            cache.updated = chrono::Local::now().to_string();
            cache.save(cache_path)?;

            println!("{} Encontrados {} archivos", "✅".green(), count);
            println!("{} Extensiones: {:?}", "📋".blue(), extensions);
            println!("{} Caché guardada en .semantic-index.json", "💾".green());
        }

        Commands::Search {
            query,
            path,
            ext,
            ignore,
            exact,
            ignore_case,
            verbose,
            no_cache,
            update,
        } => {
            let cache_path = Path::new(".semantic-index.json");

            // Si hay que actualizar o no hay caché o se fuerza no-cache
            let use_cache = !no_cache && cache_path.exists();

            let mut cache = if use_cache {
                match Cache::load(cache_path) {
                    Ok(c) => {
                        if verbose {
                            println!("{} Caché cargada ({} archivos)", "💾".green(), c.entries.len());
                        }
                        c
                    }
                    Err(_) => {
                        if verbose {
                            println!("{} Caché corrupta, escaneando de nuevo...", "⚠️".yellow());
                        }
                        Cache::new()
                    }
                }
            } else {
                if verbose && !no_cache {
                    println!("{} No se encontró caché, escaneando...", "📄".blue());
                }
                Cache::new()
            };

            // Si update o no hay caché o la caché es inválida, re-escaneamos
            let should_scan = update || !use_cache || !cache.is_valid();

            if should_scan {
                if verbose {
                    println!("{} Escaneando archivos...", "📄".blue());
                }

                let ignore_dirs: Vec<&str> = ignore.split(',').collect();
                let mut new_cache = Cache::new();

                let walker = WalkBuilder::new(&path)
                    .git_ignore(true)
                    .follow_links(false)
                    .build();

                for result in walker {
                    let entry = result?;
                    let p = entry.path();

                    if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                            if ignore_dirs.contains(&name) {
                                continue;
                            }
                        }
                    }

                    if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        if let Some(ext_str) = p.extension().and_then(|e| e.to_str()) {
                            let exts = [
                                "rs", "py", "js", "ts", "go", "java", "c", "cpp", "h",
                                "toml", "json", "txt", "md", "sh", "bash", "yaml", "yml",
                                "css", "html", "xml", "sql", "rb", "php", "swift", "kt",
                            ];
                            if exts.contains(&ext_str) {
                                if let Ok(content) = fs::read_to_string(p) {
                                    let metadata = fs::metadata(p)?;
                                    let modified = metadata
                                        .modified()
                                        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                                        .unwrap_or(0);

                                    new_cache.entries.insert(
                                        p.to_path_buf(),
                                        CacheEntry {
                                            path: p.to_path_buf(),
                                            content,
                                            modified,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }

                new_cache.updated = chrono::Local::now().to_string();
                new_cache.save(cache_path)?;
                cache = new_cache;

                if verbose {
                    println!("{} Caché actualizada ({} archivos)", "💾".green(), cache.entries.len());
                }
            }

            let query_regex = if ignore_case {
                Regex::new(&format!(r"(?i){}", regex::escape(&query)))?
            } else if exact {
                Regex::new(&format!(r"\b{}\b", regex::escape(&query)))?
            } else {
                Regex::new(&regex::escape(&query))?
            };

            let ext_filter: Option<Vec<&str>> = ext.as_ref().map(|e| e.split(',').collect());

            println!(
                "{} Buscando: '{}' en {}",
                "🔍".cyan(),
                query,
                if ext_filter.is_some() {
                    format!("(filtro: {})", ext.as_ref().unwrap())
                } else {
                    path.clone()
                }
            );

            let encontrados = Arc::new(AtomicUsize::new(0));
            let total_archivos = cache.entries.len();

            if verbose {
                println!("{} Revisando {} archivos...", "📄".blue(), total_archivos);
            }

            let entries: Vec<(&PathBuf, &CacheEntry)> = cache.entries.iter().collect();

            for (i, (p, entry)) in entries.iter().enumerate() {
                if verbose {
                    print!("\r  Progreso: {}/{}", i + 1, total_archivos);
                }

                let should_include = if let Some(ref exts) = ext_filter {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| exts.contains(&e))
                        .unwrap_or(false)
                } else {
                    true
                };

                if should_include {
                    if query_regex.is_match(&entry.content) {
                        encontrados.fetch_add(1, Ordering::SeqCst);

                        let lineas: Vec<String> = entry
                            .content
                            .lines()
                            .enumerate()
                            .filter_map(|(num, line)| {
                                if query_regex.is_match(line) {
                                    let line_num = format!("{}:", num + 1).yellow();
                                    let highlighted = if ignore_case {
                                        let re = Regex::new(&format!(r"(?i){}", regex::escape(&query))).unwrap();
                                        re.replace_all(line, |caps: &regex::Captures| {
                                            caps[0].to_string().red().to_string()
                                        }).to_string()
                                    } else if exact {
                                        let re = Regex::new(&format!(r"\b{}\b", regex::escape(&query))).unwrap();
                                        re.replace_all(line, |caps: &regex::Captures| {
                                            caps[0].to_string().red().to_string()
                                        }).to_string()
                                    } else {
                                        line.replace(&query, &query.red().to_string())
                                    };
                                    Some(format!("  {} {}", line_num, highlighted))
                                } else {
                                    None
                                }
                            })
                            .collect();

                        if !lineas.is_empty() {
                            println!("\n{}", p.display().to_string().green());
                            println!("{}", lineas.join("\n"));
                        }
                    }
                }
            }

            if verbose {
                println!();
            }

            let total_encontrados = encontrados.load(Ordering::SeqCst);

            if total_encontrados == 0 {
                println!("{} No se encontraron coincidencias", "⚠️".yellow());
            } else {
                println!(
                    "\n{} Encontrados {} archivos con coincidencias",
                    "✅".green(),
                    total_encontrados
                );
            }
        }
    }

    Ok(())
}
