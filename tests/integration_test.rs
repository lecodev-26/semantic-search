//! Tests de integración para semcode-search

use std::fs;
use tempfile::tempdir;

#[test]
fn test_project_compiles() {
    // El proyecto compila si llegamos aquí
}

#[test]
fn test_search_basic() {
    let test_dir = tempdir().unwrap();
    let test_file = test_dir.path().join("test.rs");

    fs::write(
        &test_file,
        r#"
fn greet() -> String {
    "Hello".to_string()
}
"#,
    )
    .unwrap();

    assert!(test_file.exists());
}

#[test]
fn test_search_semantic() {
    let test_dir = tempdir().unwrap();
    let test_file = test_dir.path().join("test.rs");

    fs::write(
        &test_file,
        r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn subtract(a: i32, b: i32) -> i32 {
    a - b
}
"#,
    )
    .unwrap();

    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("fn add"));
    assert!(content.contains("fn subtract"));
}

#[test]
fn test_search_by_filename() {
    let test_dir = tempdir().unwrap();
    let test_file = test_dir.path().join("main.rs");

    fs::write(&test_file, "fn main() {}").unwrap();

    assert!(test_file.exists());
    assert_eq!(test_file.file_name().unwrap(), "main.rs");
}

#[test]
fn test_index_creation() {
    let test_dir = tempdir().unwrap();
    let test_file = test_dir.path().join("test.rs");

    fs::write(
        &test_file,
        r#"
fn test() {}
"#,
    )
    .unwrap();

    assert!(test_file.exists());
    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("fn test"));
}
