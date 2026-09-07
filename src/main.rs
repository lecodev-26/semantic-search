use clap::{Parser, Subcommand};
use walkdir::WalkDir;
use std::fs;
use colored::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "semantic-search")]
#[command(about = "🔍 Buscador semántico de código", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Indexar archivos (mostrar estructura)
    Index {
        #[arg(short, long)]
        path: String,
    },
    /// Buscar texto en archivos
    Search {
        #[arg(short, long)]
        query: String,
        #[arg(short, long, default_value = ".")]
        path: String,
        #[arg(short, long, default_value_t = false)]
        verbose: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Index { path } => {
            println!("{} Indexando: {}", "📁".green(), path);
            let mut count = 0;
            let mut extensions = std::collections::HashSet::new();
            
            for entry in WalkDir::new(&path) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        let ext_str = ext.to_string_lossy().to_string();
                        let exts = ["rs", "py", "js", "ts", "go", "java", "c", "cpp", "h", "toml", "json", "txt", "md", "sh", "bash"];
                        if exts.contains(&ext_str.as_str()) {
                            count += 1;
                            extensions.insert(ext_str);
                        }
                    }
                }
            }
            
            println!("{} Encontrados {} archivos", "✅".green(), count);
            println!("{} Extensiones: {:?}", "📋".blue(), extensions);
            println!("\n💡 Para buscar usa: semantic-search search --query 'texto' --path /ruta");
        }
        
        Commands::Search { query, path, verbose } => {
            println!("{} Buscando: '{}' en {}", "🔍".cyan(), query, path);
            
            let encontrados = Arc::new(AtomicUsize::new(0));
            let _total = Arc::new(AtomicUsize::new(0));
            
            // Primera pasada: contar archivos
            let mut archivos = Vec::new();
            for entry in WalkDir::new(&path) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        let ext_str = ext.to_string_lossy().to_string();
                        let exts = ["rs", "py", "js", "ts", "go", "java", "c", "cpp", "h", "toml", "json", "txt", "md", "sh", "bash"];
                        if exts.contains(&ext_str.as_str()) {
                            archivos.push(entry.path().to_path_buf());
                        }
                    }
                }
            }
            
            let total_archivos = archivos.len();
            println!("{} Revisando {} archivos...", "📄".blue(), total_archivos);
            
            for (i, path) in archivos.iter().enumerate() {
                if verbose {
                    print!("\r  Progreso: {}/{}", i + 1, total_archivos);
                }
                
                if let Ok(content) = fs::read_to_string(path) {
                    if content.contains(&query) {
                        encontrados.fetch_add(1, Ordering::SeqCst);
                        
                        println!("\n{}", path.display().to_string().green());
                        
                        let lineas: Vec<String> = content.lines()
                            .enumerate()
                            .filter(|(_, line)| line.contains(&query))
                            .map(|(num, line)| {
                                let line_num = format!("{}:", num + 1).yellow();
                                let highlighted = line.replace(&query, &query.red().to_string());
                                format!("  {} {}", line_num, highlighted)
                            })
                            .collect();
                        
                        if !lineas.is_empty() {
                            println!("{}", lineas.join("\n"));
                        }
                    }
                }
            }
            
            let total_encontrados = encontrados.load(Ordering::SeqCst);
            
            println!("\n");
            if total_encontrados == 0 {
                println!("{} No se encontraron coincidencias", "⚠️".yellow());
            } else {
                println!("{} Encontrados {} archivos con coincidencias", "✅".green(), total_encontrados);
            }
        }
    }
    
    Ok(())
}
