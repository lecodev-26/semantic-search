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
#[command(version = "0.4.0")]
#[command(about = "🔍 Buscador semántico de código con TF-IDF")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Index {
        #[arg(short, long)]
        path: String,
        #[arg(short, long, default_value = ".git,target,node_modules,dist,build")]
        ignore: String,
        #[arg(short, long)]
        force: bool,
    },
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
        #[arg(long)]
        no_cache: bool,
        #[arg(long)]
        update: bool,
        #[arg(long, default_value_t = false)]
        semantic: bool,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CacheEntry {
    path: PathBuf,
    content: String,
    modified: u64,
    words: Vec<String>,
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

// 👇 TF-IDF SIMPLE: Vector de palabras
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

// 👇 Similitud coseno entre dos vectores
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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Index { path, ignore, force } => {
            let cache_path = Path::new(".semantic-index.json");
            if force && cache_path.exists() {
                fs::remove_file(cache_path)?;
                println!("{} Caché eliminada.", "🗑️".yellow());
            }

            println!("{} Indexando: {}", "📁".green(), path);
            let mut count = 0;
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
                    if let Some(ext) = p.extension() {
                        let ext_str = ext.to_string_lossy().to_string();
                        let exts = [
                            "rs", "py", "js", "ts", "go", "java", "c", "cpp", "h",
                            "toml", "json", "txt", "md", "sh", "bash", "yaml", "yml",
                            "css", "html", "xml", "sql", "rb", "php", "swift", "kt",
                        ];
                        if exts.contains(&ext_str.as_str()) {
                            if let Ok(content) = fs::read_to_string(p) {
                                count += 1;
                                let metadata = fs::metadata(p)?;
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

            let ext_filter: Option<Vec<&str>> = ext.as_ref().map(|e| e.split(',').collect());

            if semantic {
                println!("{} Búsqueda SEMÁNTICA (TF-IDF)", "🧠".cyan());
            } else {
                println!("{} Búsqueda por TEXTO", "🔍".cyan());
            }

            println!("  Query: '{}'", query);

            let encontrados = Arc::new(AtomicUsize::new(0));
            let total_archivos = cache.entries.len();

            if verbose {
                println!("{} Revisando {} archivos...", "📄".blue(), total_archivos);
            }

            let query_words = if semantic {
                Some(get_word_vector(&query))
            } else {
                None
            };

            let query_regex = if !semantic {
                Some(if ignore_case {
                    Regex::new(&format!(r"(?i){}", regex::escape(&query)))?
                } else if exact {
                    Regex::new(&format!(r"\b{}\b", regex::escape(&query)))?
                } else {
                    Regex::new(&regex::escape(&query))?
                })
            } else {
                None
            };

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

                if !should_include {
                    continue;
                }

                if semantic {
                    if let Some(q_vec) = &query_words {
                        let entry_vec = get_word_vector(&entry.content);
                        let similarity = cosine_similarity(q_vec, &entry_vec);
                        
                        if similarity > 0.15 {
                            encontrados.fetch_add(1, Ordering::SeqCst);
                            println!("\n{} [Similitud: {:.2}%]", p.display().to_string().green(), similarity * 100.0);
                            
                            let preview: String = entry.content.lines().take(3).collect::<Vec<_>>().join("\n");
                            println!("  {}", preview);
                        }
                    }
                } else {
                    if let Some(re) = &query_regex {
                        if re.is_match(&entry.content) {
                            encontrados.fetch_add(1, Ordering::SeqCst);

                            let lineas: Vec<String> = entry
                                .content
                                .lines()
                                .enumerate()
                                .filter_map(|(num, line)| {
                                    if re.is_match(line) {
                                        let line_num = format!("{}:", num + 1).yellow();
                                        let highlighted = if ignore_case {
                                            let re_ignore = Regex::new(&format!(r"(?i){}", regex::escape(&query))).unwrap();
                                            re_ignore.replace_all(line, |caps: &regex::Captures| {
                                                caps[0].to_string().red().to_string()
                                            }).to_string()
                                        } else if exact {
                                            let re_exact = Regex::new(&format!(r"\b{}\b", regex::escape(&query))).unwrap();
                                            re_exact.replace_all(line, |caps: &regex::Captures| {
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
            }

            if verbose {
                println!();
            }

            let total_encontrados = encontrados.load(Ordering::SeqCst);
            if total_encontrados == 0 {
                println!("{} No se encontraron coincidencias.", "⚠️".yellow());
            } else {
                println!(
                    "\n{} Encontrados {} archivos.",
                    "✅".green(),
                    total_encontrados
                );
            }
        }
    }

    Ok(())
}
