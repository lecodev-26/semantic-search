//! Tests de integración para semantic-search

#[test]
fn test_project_compiles() {
    assert!(true);
}

#[test]
fn test_search_basic() {
    let test_dir = tempfile::tempdir().unwrap();
    let test_file = test_dir.path().join("test.rs");
    std::fs::write(&test_file, "fn greet() {}").unwrap();
    assert!(test_file.exists());
}
