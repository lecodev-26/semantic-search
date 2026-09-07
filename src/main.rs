use clap::{Parser, Subcommand};
use colored::*;
use ignore::WalkBuilder;
use regex::Regex;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "semantic-search")]
#[command(version = "0.2.0")]
#[command(about = "🔍 Buscador semántico de código con búsqueda avanzada", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Indexar archivos (mostrar estadísticas)
    Index {
        #[arg(short, long)]
        path: String,

        /// Ignorar carpetas (comma-separated)
        #[arg(short, long, default_value = ".git,target,node_modules,dist,build")]
        ignore: String,
    },

    /// Buscar texto en archivos
    Search {
        #[arg(short, long)]
        query: String,

        #[arg(short, long, default_value = ".")]
        path: String,

        /// Filtrar por extensiones (comma-separated, ej: rs,py,js)
        #[arg(short = 'e', long, value_name = "EXT")]
        ext: Option<String>,

        /// Ignorar carpetas (comma-separated)
        #[arg(short = 'i', long, default_value = ".git,target,node_modules,dist,build")]
        ignore: String,

        /// Búsqueda exacta (distingue mayúsculas)
        #[arg(long)]
        exact: bool,

        /// Ignorar mayúsculas/minúsculas
        #[arg(long)]
        ignore_case: bool,

        /// Mostrar progreso
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Index { path, ignore } => {
            println!("{} Indexando: {}", "📁".green(), path);
            let mut count = 0;
            let mut extensions = std::collections::HashSet::new();

            let ignore_dirs: Vec<&str> = ignore.split(',').collect();

            let walker = WalkBuilder::new(&path)
                .git_ignore(true)
                .follow_links(false)
                .build();

            for result in walker {
                let entry = result?;
                let path = entry.path();

                // Saltar carpetas ignoradas
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
                        }
                    }
                }
            }

            println!("{} Encontrados {} archivos", "✅".green(), count);
            println!("{} Extensiones: {:?}", "📋".blue(), extensions);
        }

        Commands::Search {
            query,
            path,
            ext,
            ignore,
            exact,
            ignore_case,
            verbose,
        } => {
            let ignore_dirs: Vec<&str> = ignore.split(',').collect();

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
            let mut archivos = Vec::new();

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
                    let should_include = if let Some(ref exts) = ext_filter {
                        path.extension()
                            .and_then(|e| e.to_str())
                            .map(|e| exts.contains(&e))
                            .unwrap_or(false)
                    } else {
                        true
                    };

                    if should_include {
                        archivos.push(path.to_path_buf());
                    }
                }
            }

            let total_archivos = archivos.len();

            if verbose {
                println!("{} Revisando {} archivos...", "📄".blue(), total_archivos);
            }

            for (i, path) in archivos.iter().enumerate() {
                if verbose {
                    print!("\r  Progreso: {}/{}", i + 1, total_archivos);
                }

                if let Ok(content) = fs::read_to_string(path) {
                    let mut found = false;
                    let lineas: Vec<String> = content
                        .lines()
                        .enumerate()
                        .filter_map(|(num, line)| {
                            if query_regex.is_match(line) {
                                found = true;
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

                    if found {
                        encontrados.fetch_add(1, Ordering::SeqCst);
                        println!("\n{}", path.display().to_string().green());
                        if !lineas.is_empty() {
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
