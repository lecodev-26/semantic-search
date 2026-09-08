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
use std::time::Instant;
use byte_unit::Byte;

#[derive(Parser)]
#[command(name = "semantic-search")]
#[command(version = "0.6.0")]
#[command(about = "🔍 Buscador semántico de código con TF-IDF, caché y filtros avanzados")]
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
        #[arg(short, long)]
        force: bool,
        #[arg(short = 'e', long, value_name = "EXT")]
        ext: Option<String>,
    },
    /// Buscar en archivos
    Search {
        #[arg(short, long)]
        query: Option<String>,

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

        #[arg(long)]
        no_cache: bool,

        #[arg(long)]
        update: bool,

        #[arg(long, default_value_t = false)]
        semantic: bool,

        #[arg(short = 'f', long, value_name = "FILE")]
        file: Option<String>,

        #[arg(long, default_value_t = true)]
        summary: bool,

        #[arg(long, value_name = "SIZE")]
        max_size: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CacheEntry {
    path: PathBuf,
    content: String,
    modified: u64,
    words: Vec<String>,
    size: u64,
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
        if let Ok(updated) = chrono::DateTime::parse_from_rfc3339(&self.updated) {
            let now = chrono::Local::now();
            let diff = now.signed_duration_since(updated.with_timezone(&chrono::Local));
            return diff.num_minutes() < 60;
        }
        false
    }
}

// 👇 TF-IDF SIMPLE
fn get_word_vector(text: &str) -> HashMap<String, u32> {
    let mut word_count = HashMap::new();
    for word in text.split_whitespace() {
        let word = word.to_lowercase();
        let word = word.trim_matches(|c: char| !c.is_alphanumeric());
        if word.len() > 2 {
            *word_count.entry(word.to_string()).or_insert(0) += 1;
        }
    }
    word_count
}

fn cosine_similarity(vec1: &HashMap<String, u32>, vec2: &HashMap<String, u32>) -> f32 {
    let mut dot_product = 0.0;
    let mut norm1 = 0.0;
    let mut norm2 = 0.0;

    for (word, count1) in vec1 {
        if let Some(count2) = vec2.get(word) {
            dot_product += (*count1 as f32) * (*count2 as f32);
        }
        norm1 += (*count1 as f32) * (*count1 as f32);
    }

    for count2 in vec2.values() {
        norm2 += (*count2 as f32) * (*count2 as f32);
    }

    if norm1 == 0.0 || norm2 == 0.0 {
        return 0.0;
    }

    dot_product / (norm1.sqrt() * norm2.sqrt())
}

// 👇 CORREGIDO: parse_str + as_u64
fn parse_size(size_str: &str) -> anyhow::Result<u64> {
    let size = Byte::parse_str(size_str, true)
        .map_err(|e| anyhow::anyhow!("Error parsing size: {}", e))?;
    Ok(size.as_u64())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let start_time = Instant::now();

    match cli.command {
        Commands::Index { path, ignore, force, ext } => {
            let cache_path = Path::new(".semantic-index.json");
            if force && cache_path.exists() {
                fs::remove_file(cache_path)?;
                println!("{} Caché eliminada.", "🗑️".yellow());
            }

            println!("{} Indexando: {}", "📁".green(), path);

            let ext_filter: Option<Vec<&str>> = ext.as_ref().map(|e| e.split(',').collect());

            let mut count = 0;
            let mut total_size = 0u64;
            let mut cache = Cache::new();
            let ignore_dirs: Vec<&str> = ignore.split(',').collect();

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
                        let should_include = if let Some(ref exts) = ext_filter {
                            exts.contains(&ext_str)
                        } else {
                            true
                        };

                        if !should_include {
                            continue;
                        }

                        let exts = [
                            "rs", "py", "js", "ts", "go", "java", "c", "cpp", "h",
                            "toml", "json", "txt", "md", "sh", "bash", "yaml", "yml",
                            "css", "html", "xml", "sql", "rb", "php", "swift", "kt",
                        ];
                        if exts.contains(&ext_str) {
                            if let Ok(content) = fs::read_to_string(p) {
                                let metadata = fs::metadata(p)?;
                                let file_size = metadata.len();
                                total_size += file_size;
                                count += 1;
                                let modified = metadata
                                    .modified()
                                    .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                                    .unwrap_or(0);

                                let words: Vec<String> = content
                                    .split_whitespace()
                                    .map(|w| w.to_lowercase())
                                    .filter(|w| w.len() > 2)
                                    .collect();

                                cache.entries.insert(
                                    p.to_path_buf(),
                                    CacheEntry {
                                        path: p.to_path_buf(),
                                        content,
                                        modified,
                                        words,
                                        size: file_size,
                                    },
                                );
                            }
                        }
                    }
                }
            }

            cache.updated = chrono::Local::now().to_string();
            cache.save(cache_path)?;

            println!("{} Indexados {} archivos.", "✅".green(), count);
            if let Some(ref exts) = ext_filter {
                println!("{} Filtro por extensiones: {:?}", "📋".blue(), exts);
            }
            println!("{} Tamaño total: {}", "💾".blue(), format_size(total_size));
            println!("{} Caché guardada en .semantic-index.json", "💾".green());
        }

        Commands::Search {
            query,
            path: _,
            ext,
            ignore: _,
            exact,
            ignore_case,
            verbose,
            no_cache,
            update: _,
            semantic,
            file,
            summary,
            max_size,
        } => {
            let cache_path = Path::new(".semantic-index.json");

            let use_cache = !no_cache && cache_path.exists();
            let cache = if use_cache {
                match Cache::load(cache_path) {
                    Ok(c) => c,
                    Err(_) => {
                        println!("{} Caché corrupta. Ejecute 'index' primero.", "⚠️".yellow());
                        return Ok(());
                    }
                }
            } else {
                println!("{} No se encontró caché. Ejecute 'index' primero.", "⚠️".yellow());
                return Ok(());
            };

            let max_size_bytes = if let Some(size_str) = max_size {
                Some(parse_size(&size_str)?)
            } else {
                None
            };

            let ext_filter: Option<Vec<&str>> = ext.as_ref().map(|e| e.split(',').collect());

            if let Some(file_pattern) = file {
                println!("{} Buscando archivos por nombre: '{}'", "📄".cyan(), file_pattern);

                let found: Vec<PathBuf> = cache
                    .entries
                    .keys()
                    .filter(|p| {
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| {
                                if ignore_case {
                                    n.to_lowercase().contains(&file_pattern.to_lowercase())
                                } else {
                                    n.contains(&file_pattern)
                                }
                            })
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect();

                if found.is_empty() {
                    println!("{} No se encontraron archivos con ese nombre.", "⚠️".yellow());
                } else {
                    let total = found.len();
                    for p in &found {
                        println!("{}", p.display().to_string().green());
                    }
                    println!("\n{} Encontrados {} archivos.", "✅".green(), total);
                }
                return Ok(());
            }

            let query_str = match query {
                Some(q) => q,
                None => {
                    println!("{} Debes proporcionar una query con --query", "⚠️".yellow());
                    println!("  Buscar por texto: --query 'texto'");
                    println!("  Buscar por nombre: --file 'nombre.rs'");
                    return Ok(());
                }
            };

            if semantic {
                println!("{} Búsqueda SEMÁNTICA (TF-IDF)", "🧠".cyan());
            } else {
                println!("{} Búsqueda por TEXTO", "🔍".cyan());
            }
            println!("  Query: '{}'", query_str);

            if let Some(max_size) = max_size_bytes {
                println!("  {} Máximo tamaño: {}", "📏".blue(), format_size(max_size));
            }

            let encontrados = Arc::new(AtomicUsize::new(0));
            let total_ocurrencias = Arc::new(AtomicUsize::new(0));
            let total_archivos = cache.entries.len();

            if verbose {
                println!("{} Revisando {} archivos...", "📄".blue(), total_archivos);
            }

            let query_words = if semantic {
                Some(get_word_vector(&query_str))
            } else {
                None
            };

            let query_regex = if !semantic {
                Some(if ignore_case {
                    Regex::new(&format!(r"(?i){}", regex::escape(&query_str)))?
                } else if exact {
                    Regex::new(&format!(r"\b{}\b", regex::escape(&query_str)))?
                } else {
                    Regex::new(&regex::escape(&query_str))?
                })
            } else {
                None
            };

            let entries: Vec<(&PathBuf, &CacheEntry)> = cache.entries.iter().collect();

            for (i, (p, entry)) in entries.iter().enumerate() {
                if verbose {
                    print!("\r  Progreso: {}/{}", i + 1, total_archivos);
                }

                if let Some(max_size) = max_size_bytes {
                    if entry.size > max_size {
                        continue;
                    }
                }

                let should_include = if let Some(ref exts) = ext_filter {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| exts.contains(&e))
                        .unwrap_or(false)
                } else {
                    true
                };

                if !should_include {
                    continue;
                }

                if semantic {
                    if let Some(q_vec) = &query_words {
                        let entry_vec = get_word_vector(&entry.content);
                        let similarity = cosine_similarity(q_vec, &entry_vec);

                        if similarity > 0.15 {
                            encontrados.fetch_add(1, Ordering::SeqCst);
                            total_ocurrencias.fetch_add(1, Ordering::SeqCst);

                            let ocurrencias = entry_vec.values().sum::<u32>();
                            println!("\n{} [Similitud: {:.2}%] ({} palabras clave) [{}]",
                                p.display().to_string().green(),
                                similarity * 100.0,
                                ocurrencias,
                                format_size(entry.size).dimmed()
                            );

                            let preview: String = entry.content.lines().take(3).collect::<Vec<_>>().join("\n");
                            println!("  {}", preview);
                        }
                    }
                } else {
                    if let Some(re) = &query_regex {
                        let matches: Vec<_> = re.find_iter(&entry.content).collect();
                        if !matches.is_empty() {
                            encontrados.fetch_add(1, Ordering::SeqCst);
                            total_ocurrencias.fetch_add(matches.len(), Ordering::SeqCst);

                            println!("\n{} ({} coincidencias) [{}]",
                                p.display().to_string().green(),
                                matches.len(),
                                format_size(entry.size).dimmed()
                            );

                            let lineas: Vec<String> = entry
                                .content
                                .lines()
                                .enumerate()
                                .filter_map(|(num, line)| {
                                    if re.is_match(line) {
                                        let line_num = format!("{}:", num + 1).yellow();
                                        let highlighted = if ignore_case {
                                            let re_ignore = Regex::new(&format!(r"(?i){}", regex::escape(&query_str))).unwrap();
                                            re_ignore.replace_all(line, |caps: &regex::Captures| {
                                                caps[0].to_string().red().to_string()
                                            }).to_string()
                                        } else if exact {
                                            let re_exact = Regex::new(&format!(r"\b{}\b", regex::escape(&query_str))).unwrap();
                                            re_exact.replace_all(line, |caps: &regex::Captures| {
                                                caps[0].to_string().red().to_string()
                                            }).to_string()
                                        } else {
                                            line.replace(&query_str, &query_str.red().to_string())
                                        };
                                        Some(format!("  {} {}", line_num, highlighted))
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            if !lineas.is_empty() {
                                println!("{}", lineas.join("\n"));
                            }
                        }
                    }
                }
            }

            if verbose {
                println!();
            }

            let total_encontrados = encontrados.load(Ordering::SeqCst);
            let total_matches = total_ocurrencias.load(Ordering::SeqCst);
            let elapsed = start_time.elapsed();

            if total_encontrados == 0 {
                println!("{} No se encontraron coincidencias.", "⚠️".yellow());
            } else {
                if summary {
                    println!("\n{} {}", "📊".blue(), "Resumen:".bold());
                    println!("  {} Archivos encontrados: {}", "•".cyan(), total_encontrados);
                    println!("  {} Coincidencias totales: {}", "•".cyan(), total_matches);
                    println!("  {} Tiempo: {:.2}s", "•".cyan(), elapsed.as_secs_f32());
                } else {
                    println!(
                        "\n{} Encontrados {} archivos.",
                        "✅".green(),
                        total_encontrados
                    );
                }
            }
        }
    }

    Ok(())
}
