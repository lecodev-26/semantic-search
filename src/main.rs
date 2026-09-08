use clap::{Parser, Subcommand};
use colored::*;
use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use byte_unit::Byte;
use glob::Pattern;
use dirs;


#[derive(Parser)]
#[command(name = "semantic-search")]
#[command(version = "0.8.0")]
#[command(about = "🔍 Buscador semántico de código con configuración, alias y modo interactivo")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(short, long)]
        force: bool,
    },
    Alias {
        #[command(subcommand)]
        action: AliasAction,
    },
    Index {
        #[arg(short, long)]
        path: String,
        #[arg(short, long, default_value = ".git,target,node_modules,dist,build")]
        ignore: String,
        #[arg(short, long)]
        force: bool,
        #[arg(short = 'e', long, value_name = "EXT")]
        ext: Option<String>,
        #[arg(long, value_name = "PATTERN")]
        ignore_pattern: Option<String>,
    },
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
        #[arg(long, value_name = "PATTERN")]
        ignore_pattern: Option<String>,
        #[arg(long)]
        extract: bool,
        #[arg(long)]
        interactive: bool,
        #[arg(long, value_name = "NAME")]
        alias: Option<String>,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum AliasAction {
    Save {
        name: String,
        query: String,
        params: Vec<String>,
    },
    List,
    Remove {
        name: String,
    },
    Run {
        name: String,
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

#[derive(Debug, Serialize, Deserialize, Default)]
struct Config {
    default_ext: Option<Vec<String>>,
    default_ignore: Option<Vec<String>>,
    default_ignore_pattern: Option<String>,
    max_size: Option<String>,
    verbose: Option<bool>,
    interactive: Option<bool>,
    aliases: HashMap<String, AliasEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AliasEntry {
    query: String,
    params: Vec<String>,
}

fn get_config_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("semantic-search").join("config.toml")
    } else {
        PathBuf::from(".semantic-search-config.toml")
    }
}

fn load_config() -> anyhow::Result<Config> {
    let config_path = get_config_path();
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

fn save_config(config: &Config) -> anyhow::Result<()> {
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    Ok(())
}

fn apply_config_to_search(
    mut ext: Option<String>,
    mut ignore: String,
    mut verbose: bool,
    mut interactive: bool,
    mut max_size: Option<String>,
    mut ignore_pattern: Option<String>,
    config: &Config,
) -> (Option<String>, String, bool, bool, Option<String>, Option<String>) {
    if let Some(ref default_ext) = config.default_ext {
        if ext.is_none() {
            ext = Some(default_ext.join(","));
        }
    }
    if let Some(ref default_ignore) = config.default_ignore {
        ignore = default_ignore.join(",");
    }
    if config.verbose.unwrap_or(false) && !verbose {
        verbose = true;
    }
    if config.interactive.unwrap_or(false) && !interactive {
        interactive = true;
    }
    if max_size.is_none() {
        max_size = config.max_size.clone();
    }
    if ignore_pattern.is_none() {
        ignore_pattern = config.default_ignore_pattern.clone();
    }
    (ext, ignore, verbose, interactive, max_size, ignore_pattern)
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

struct SearchResult {
    path: PathBuf,
    content: String,
    matches: usize,
    size: u64,
    line_start: Option<usize>,
    line_end: Option<usize>,
    highlight_regex: Option<Regex>,
}

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

fn matches_pattern(path: &Path, pattern: &str) -> bool {
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        if let Ok(pattern) = Pattern::new(pattern) {
            return pattern.matches(file_name);
        }
    }
    false
}

fn extract_archive_content(path: &Path) -> anyhow::Result<Option<String>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext == "zip" {
        if let Ok(content) = fs::read_to_string(path) {
            return Ok(Some(content));
        }
    }
    Ok(None)
}

fn interactive_mode(results: &[SearchResult]) -> anyhow::Result<()> {
    if results.is_empty() {
        println!("{} No hay resultados para mostrar.", "⚠️".yellow());
        return Ok(());
    }

    let total = results.len();
    let mut idx = 0;

    loop {
        clear_screen();
        println!("{} {} de {} (Presiona Enter para avanzar, q para salir)", 
            "📖".cyan(), idx + 1, total);

        let result = &results[idx];
        println!("\n{}", result.path.display().to_string().green().bold());
        println!("  Coincidencias: {}", result.matches);
        println!("  Tamaño: {}", format_size(result.size));

        let lines: Vec<&str> = result.content.lines().collect();
        let start = result.line_start.unwrap_or(0).saturating_sub(3);
        let end = result.line_end.unwrap_or(lines.len()).saturating_add(3);

        for (i, line) in lines.iter().enumerate().take(end).skip(start) {
            let line_num = format!("{}:", i + 1).yellow();
            let display_line = if let Some(ref re) = result.highlight_regex {
                re.replace_all(line, |caps: &regex::Captures| {
                    caps[0].to_string().red().to_string()
                }).to_string()
            } else {
                line.to_string()
            };
            println!("  {} {}", line_num, display_line);
        }

        println!("\n---");
        print!("[Enter] siguiente  [q] salir ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "q" || input == "Q" {
            break;
        }

        idx = (idx + 1) % total;
        if idx == 0 {
            println!("\n{} Has llegado al final. Volviendo al principio...", "🔄".yellow());
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let start_time = Instant::now();

    let config = load_config().unwrap_or_default();

    match cli.command {
        Commands::Init { force } => {
            let config_path = get_config_path();
            if config_path.exists() && !force {
                println!("{} Configuración ya existe. Usa --force para sobreescribir.", "⚠️".yellow());
                return Ok(());
            }
            let default_config = Config {
                default_ext: Some(vec!["rs".to_string(), "md".to_string(), "toml".to_string()]),
                default_ignore: Some(vec![".git".to_string(), "target".to_string(), "node_modules".to_string()]),
                default_ignore_pattern: Some("*.log".to_string()),
                max_size: Some("1MB".to_string()),
                verbose: Some(true),
                interactive: Some(false),
                aliases: HashMap::new(),
            };
            save_config(&default_config)?;
            println!("{} Configuración creada en: {}", "✅".green(), config_path.display());
            println!("{} Puedes editarla manualmente o usar 'alias' para gestionar búsquedas.", "💡".cyan());
        }

        Commands::Alias { action } => {
            let mut config = load_config()?;
            match action {
                AliasAction::Save { name, query, params } => {
                    config.aliases.insert(name.clone(), AliasEntry { query, params });
                    save_config(&config)?;
                    println!("{} Alias '{}' guardado.", "✅".green(), name);
                }
                AliasAction::List => {
                    if config.aliases.is_empty() {
                        println!("{} No hay alias guardados.", "📭".yellow());
                    } else {
                        println!("{} Alias guardados:", "📋".blue());
                        for (name, entry) in &config.aliases {
                            println!("  {}: {} {}", name.green(), entry.query, entry.params.join(" "));
                        }
                    }
                }
                AliasAction::Remove { name } => {
                    if config.aliases.remove(&name).is_some() {
                        save_config(&config)?;
                        println!("{} Alias '{}' eliminado.", "🗑️".green(), name);
                    } else {
                        println!("{} Alias '{}' no encontrado.", "⚠️".yellow(), name);
                    }
                }
                AliasAction::Run { name } => {
                    if let Some(entry) = config.aliases.get(&name) {
                        println!("{} Ejecutando alias '{}':", "🚀".cyan(), name);
                        println!("  query: {}", entry.query);
                        println!("  params: {}", entry.params.join(" "));
                        println!("💡 Para ejecutar, usa: semantic-search search --query \"{}\" {}", entry.query, entry.params.join(" "));
                    } else {
                        println!("{} Alias '{}' no encontrado.", "⚠️".yellow(), name);
                    }
                }
            }
        }

        Commands::Index { path, ignore, force, ext, ignore_pattern } => {
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
                    if let Some(ref pattern) = ignore_pattern {
                        if matches_pattern(p, pattern) {
                            continue;
                        }
                    }

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
                            "zip",
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
            if let Some(ref pattern) = ignore_pattern {
                println!("{} Ignorando patrón: {}", "🚫".blue(), pattern);
            }
            println!("{} Tamaño total: {}", "💾".blue(), format_size(total_size));
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
            update: _,
            semantic,
            file,
            summary,
            max_size,
            ignore_pattern,
            extract,
            interactive,
            alias,
        } => {
            let (query, ext, ignore, verbose, interactive, max_size, ignore_pattern) = if let Some(alias_name) = alias {
                let config = load_config()?;
                if let Some(entry) = config.aliases.get(&alias_name) {
                    let q = entry.query.clone();
                    let params = entry.params.clone();
                    let mut new_ext = ext;
                    let mut new_ignore = ignore;
                    let mut new_verbose = verbose;
                    let mut new_interactive = interactive;
                    let mut new_max_size = max_size;
                    let mut new_ignore_pattern = ignore_pattern;
                    for param in &params {
                        if param.starts_with("--ext") || param.starts_with("-e") {
                            if let Some(ext_value) = params.iter().find(|p| p.starts_with("--ext") || p.starts_with("-e")).and_then(|p| p.split('=').nth(1)) {
                                new_ext = Some(ext_value.to_string());
                            }
                        }
                        if param == "--verbose" || param == "-v" {
                            new_verbose = true;
                        }
                        if param == "--interactive" {
                            new_interactive = true;
                        }
                        if param.starts_with("--max-size") {
                            if let Some(size_value) = params.iter().find(|p| p.starts_with("--max-size")).and_then(|p| p.split('=').nth(1)) {
                                new_max_size = Some(size_value.to_string());
                            }
                        }
                        if param.starts_with("--ignore-pattern") {
                            if let Some(pattern_value) = params.iter().find(|p| p.starts_with("--ignore-pattern")).and_then(|p| p.split('=').nth(1)) {
                                new_ignore_pattern = Some(pattern_value.to_string());
                            }
                        }
                        if param.starts_with("--ignore") || param.starts_with("-i") {
                            if let Some(ignore_value) = params.iter().find(|p| p.starts_with("--ignore") || p.starts_with("-i")).and_then(|p| p.split('=').nth(1)) {
                                new_ignore = ignore_value.to_string();
                            }
                        }
                    }
                    (Some(q), new_ext, new_ignore, new_verbose, new_interactive, new_max_size, new_ignore_pattern)
                } else {
                    println!("{} Alias '{}' no encontrado.", "⚠️".yellow(), alias_name);
                    return Ok(());
                }
            } else if let Some(q) = query {
                let (new_ext, new_ignore, new_verbose, new_interactive, new_max_size, new_ignore_pattern) = 
                    apply_config_to_search(ext, ignore, verbose, interactive, max_size, ignore_pattern, &config);
                (Some(q), new_ext, new_ignore, new_verbose, new_interactive, new_max_size, new_ignore_pattern)
            } else {
                println!("{} Debes proporcionar una query con --query o un alias con --alias", "⚠️".yellow());
                return Ok(());
            };

            let query_str = match query {
                Some(q) => q,
                None => {
                    println!("{} Error: no se pudo obtener la query.", "⚠️".yellow());
                    return Ok(());
                }
            };

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

            if semantic {
                println!("{} Búsqueda SEMÁNTICA (TF-IDF)", "🧠".cyan());
            } else {
                println!("{} Búsqueda por TEXTO", "🔍".cyan());
            }
            println!("  Query: '{}'", query_str);

            if let Some(max_size) = max_size_bytes {
                println!("  {} Máximo tamaño: {}", "📏".blue(), format_size(max_size));
            }
            if let Some(ref pattern) = ignore_pattern {
                println!("  {} Ignorando patrón: {}", "🚫".blue(), pattern);
            }
            if extract {
                println!("  {} Buscando en archivos comprimidos (experimental)", "📦".blue());
            }
            if interactive {
                println!("  {} Modo interactivo activado", "🎮".blue());
            }

            let encontrados = Arc::new(AtomicUsize::new(0));
            let total_ocurrencias = Arc::new(AtomicUsize::new(0));
            let total_archivos = cache.entries.len();
            let mut search_results = Vec::new();

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

                if let Some(ref pattern) = ignore_pattern {
                    if matches_pattern(p, pattern) {
                        continue;
                    }
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

                let content_to_search = if extract {
                    if let Ok(Some(extracted)) = extract_archive_content(p) {
                        extracted
                    } else {
                        entry.content.clone()
                    }
                } else {
                    entry.content.clone()
                };

                if semantic {
                    if let Some(q_vec) = &query_words {
                        let entry_vec = get_word_vector(&content_to_search);
                        let similarity = cosine_similarity(q_vec, &entry_vec);

                        if similarity > 0.15 {
                            encontrados.fetch_add(1, Ordering::SeqCst);
                            total_ocurrencias.fetch_add(1, Ordering::SeqCst);

                            let ocurrencias = entry_vec.values().sum::<u32>();
                            let result = SearchResult {
                                path: p.to_path_buf(),
                                content: content_to_search.clone(),
                                matches: ocurrencias as usize,
                                size: entry.size,
                                line_start: None,
                                line_end: None,
                                highlight_regex: None,
                            };
                            search_results.push(result);

                            if !interactive {
                                println!("\n{} [Similitud: {:.2}%] ({} palabras clave) [{}]",
                                    p.display().to_string().green(),
                                    similarity * 100.0,
                                    ocurrencias,
                                    format_size(entry.size).dimmed()
                                );
                                let preview: String = content_to_search.lines().take(3).collect::<Vec<_>>().join("\n");
                                println!("  {}", preview);
                            }
                        }
                    }
                } else {
                    if let Some(re) = &query_regex {
                        let matches: Vec<_> = re.find_iter(&content_to_search).collect();
                        if !matches.is_empty() {
                            encontrados.fetch_add(1, Ordering::SeqCst);
                            total_ocurrencias.fetch_add(matches.len(), Ordering::SeqCst);

                            let first_match = matches.first().map(|m| m.start());
                            let last_match = matches.last().map(|m| m.end());
                            let (line_start, line_end) = if let (Some(start), Some(end)) = (first_match, last_match) {
                                let content_before = &content_to_search[..start];
                                let line_start = content_before.lines().count();
                                let content_until = &content_to_search[..end];
                                let line_end = content_until.lines().count();
                                (Some(line_start), Some(line_end))
                            } else {
                                (None, None)
                            };

                            let result = SearchResult {
                                path: p.to_path_buf(),
                                content: content_to_search.clone(),
                                matches: matches.len(),
                                size: entry.size,
                                line_start,
                                line_end,
                                highlight_regex: query_regex.clone(),
                            };
                            search_results.push(result);

                            if !interactive {
                                println!("\n{} ({} coincidencias) [{}]",
                                    p.display().to_string().green(),
                                    matches.len(),
                                    format_size(entry.size).dimmed()
                                );

                                let lineas: Vec<String> = content_to_search
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
            }

            if verbose {
                println!();
            }

            let total_encontrados = encontrados.load(Ordering::SeqCst);
            let total_matches = total_ocurrencias.load(Ordering::SeqCst);
            let elapsed = start_time.elapsed();

            if interactive && !search_results.is_empty() {
                interactive_mode(&search_results)?;
            } else if !interactive && total_encontrados == 0 {
                println!("{} No se encontraron coincidencias.", "⚠️".yellow());
            } else if !interactive {
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
