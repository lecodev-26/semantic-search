//! # semcode-search - Punto de entrada de la aplicación

use clap::Parser;
use colored::*;
use semcode_search::cli::{Cli, Commands, AliasAction};
use semcode_search::core::{index_files, search_files, SearchConfigInternal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ===== Configuración =====

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub default_ext: Option<Vec<String>>,
    pub default_ignore: Option<Vec<String>>,
    pub default_ignore_pattern: Option<String>,
    pub max_size: Option<String>,
    pub verbose: Option<bool>,
    pub interactive: Option<bool>,
    pub aliases: HashMap<String, AliasEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AliasEntry {
    pub query: String,
    pub params: Vec<String>,
}

pub fn get_config_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("semcode-search").join("config.toml")
    } else {
        PathBuf::from(".semcode-search-config.toml")
    }
}

pub fn load_config() -> anyhow::Result<Config> {
    let config_path = get_config_path();
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

pub fn save_config(config: &Config) -> anyhow::Result<()> {
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    Ok(())
}

type AppliedConfig = (Option<Vec<String>>, Vec<String>, bool, bool, Option<String>, Option<String>);

fn apply_config_to_search(
    ext: Option<String>,
    ignore: String,
    verbose: bool,
    interactive: bool,
    max_size: Option<String>,
    ignore_pattern: Option<String>,
    config: &Config,
) -> AppliedConfig {
    let mut ext_vec = ext.as_ref().map(|e| e.split(',').map(|s| s.to_string()).collect());
    let mut ignore_vec: Vec<String> = ignore.split(',').map(|s| s.to_string()).collect();
    let mut new_verbose = verbose;
    let mut new_interactive = interactive;
    let mut new_max_size = max_size;
    let mut new_ignore_pattern = ignore_pattern;

    if let Some(ref default_ext) = config.default_ext {
        if ext_vec.is_none() {
            ext_vec = Some(default_ext.clone());
        }
    }
    if let Some(ref default_ignore) = config.default_ignore {
        ignore_vec = default_ignore.clone();
    }
    if config.verbose.unwrap_or(false) && !verbose {
        new_verbose = true;
    }
    if config.interactive.unwrap_or(false) && !interactive {
        new_interactive = true;
    }
    if new_max_size.is_none() {
        new_max_size = config.max_size.clone();
    }
    if new_ignore_pattern.is_none() {
        new_ignore_pattern = config.default_ignore_pattern.clone();
    }
    (
        ext_vec,
        ignore_vec,
        new_verbose,
        new_interactive,
        new_max_size,
        new_ignore_pattern,
    )
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = load_config().unwrap_or_default();

    match cli.command {
        Commands::Init { force } => {
            let config_path = get_config_path();
            if config_path.exists() && !force {
                println!(
                    "{} Configuración ya existe. Usa --force para sobreescribir.",
                    "⚠️".yellow()
                );
                return Ok(());
            }
            let default_config = Config {
                default_ext: Some(vec!["rs".to_string(), "md".to_string(), "toml".to_string()]),
                default_ignore: Some(vec![
                    ".git".to_string(),
                    "target".to_string(),
                    "node_modules".to_string(),
                ]),
                default_ignore_pattern: Some("*.log".to_string()),
                max_size: Some("1MB".to_string()),
                verbose: Some(true),
                interactive: Some(false),
                aliases: HashMap::new(),
            };
            save_config(&default_config)?;
            println!(
                "{} Configuración creada en: {}",
                "✅".green(),
                config_path.display()
            );
            println!(
                "{} Puedes editarla manualmente o usar 'alias' para gestionar búsquedas.",
                "💡".cyan()
            );
        }

        Commands::Alias(action) => {
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
                            println!(
                                "  {}: {} {}",
                                name.green(),
                                entry.query,
                                entry.params.join(" ")
                            );
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
                        let params_str = entry.params.join(" ");
                        println!("{} Ejecutando alias '{}':", "🚀".cyan(), name);
                        println!("  query: {}", entry.query);
                        println!("  params: {}", params_str);
                        println!(
                            "💡 Para ejecutar manualmente: semcode-search search --query \"{}\" {}",
                            entry.query, params_str
                        );
                    } else {
                        println!("{} Alias '{}' no encontrado.", "⚠️".yellow(), name);
                    }
                }
            }
        }

        Commands::Index {
            path,
            ignore,
            force,
            ext,
            ignore_pattern,
        } => {
            let ignore_dirs: Vec<&str> = ignore.split(',').collect();
            let ext_filter = ext.as_ref().map(|e| e.split(',').collect());
            index_files(
                &path,
                ignore_dirs,
                ext_filter,
                ignore_pattern.as_deref(),
                force,
            )?;
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
            let (query, ext_vec, ignore_vec, verbose, interactive, max_size, ignore_pattern) =
                if let Some(alias_name) = alias {
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
                                if let Some(ext_value) = params
                                    .iter()
                                    .find(|p| p.starts_with("--ext") || p.starts_with("-e"))
                                    .and_then(|p| p.split('=').nth(1))
                                {
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
                                if let Some(size_value) = params
                                    .iter()
                                    .find(|p| p.starts_with("--max-size"))
                                    .and_then(|p| p.split('=').nth(1))
                                {
                                    new_max_size = Some(size_value.to_string());
                                }
                            }
                            if param.starts_with("--ignore-pattern") {
                                if let Some(pattern_value) = params
                                    .iter()
                                    .find(|p| p.starts_with("--ignore-pattern"))
                                    .and_then(|p| p.split('=').nth(1))
                                {
                                    new_ignore_pattern = Some(pattern_value.to_string());
                                }
                            }
                            if param.starts_with("--ignore") || param.starts_with("-i") {
                                if let Some(ignore_value) = params
                                    .iter()
                                    .find(|p| p.starts_with("--ignore") || p.starts_with("-i"))
                                    .and_then(|p| p.split('=').nth(1))
                                {
                                    new_ignore = ignore_value.to_string();
                                }
                            }
                        }
                        let (ext_vec, ignore_vec, v, i, max, pat) = apply_config_to_search(
                            new_ext,
                            new_ignore,
                            new_verbose,
                            new_interactive,
                            new_max_size,
                            new_ignore_pattern,
                            &config,
                        );
                        (Some(q), ext_vec, ignore_vec, v, i, max, pat)
                    } else {
                        println!("{} Alias '{}' no encontrado.", "⚠️".yellow(), alias_name);
                        return Ok(());
                    }
                } else if let Some(q) = query {
                    let (ext_vec, ignore_vec, v, i, max, pat) = apply_config_to_search(
                        ext,
                        ignore,
                        verbose,
                        interactive,
                        max_size,
                        ignore_pattern,
                        &config,
                    );
                    (Some(q), ext_vec, ignore_vec, v, i, max, pat)
                } else {
                    println!(
                        "{} Debes proporcionar una query con --query o un alias con --alias",
                        "⚠️".yellow()
                    );
                    return Ok(());
                };

            let query_str = match query {
                Some(q) => q,
                None => {
                    println!("{} Error: no se pudo obtener la query.", "⚠️".yellow());
                    return Ok(());
                }
            };

            let search_config = SearchConfigInternal {
                query: query_str,
                path,
                ext: ext_vec,
                ignore: ignore_vec,
                exact,
                ignore_case,
                verbose,
                no_cache,
                semantic,
                file,
                summary,
                max_size,
                ignore_pattern,
                extract,
                interactive,
            };

            search_files(search_config)?;
        }
    }

    Ok(())
}
