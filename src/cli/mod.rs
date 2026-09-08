//! Módulo CLI - Comandos y argumentos

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "semantic-search")]
#[command(version = "0.9.0")]
#[command(about = "🔍 Buscador semántico de código con TF-IDF, caché y filtros avanzados")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Init { force: bool },
    Alias { action: AliasAction },
    Index {
        path: String,
        ignore: String,
        force: bool,
        ext: Option<String>,
        ignore_pattern: Option<String>,
    },
    Search {
        query: Option<String>,
        path: String,
        ext: Option<String>,
        ignore: String,
        exact: bool,
        ignore_case: bool,
        verbose: bool,
        no_cache: bool,
        update: bool,
        semantic: bool,
        file: Option<String>,
        summary: bool,
        max_size: Option<String>,
        ignore_pattern: Option<String>,
        extract: bool,
        interactive: bool,
        alias: Option<String>,
    },
}

#[derive(ValueEnum, Debug, Clone)]
pub enum AliasAction {
    Save,
    List,
    Remove,
    Run,
}
