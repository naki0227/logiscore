use super::*;

fn file(path: &str, extension: &str, source: &str) -> ProjectFilePayload {
    ProjectFilePayload::new(path, extension, source).unwrap()
}

#[test]
fn project_roundtrips_nested_unicode_files_in_canonical_order() {
    let original = ProjectPayload::new(vec![
        file("src/挨拶.rs", ".rs", "// こんにちは"),
        file("README.md", ".md", "# Logiscore"),
    ])
    .unwrap();
    let restored = ProjectPayload::from_bytes(&original.clone().into_bytes()).unwrap();
    assert_eq!(restored, original);
    assert_eq!(restored.files()[0].path(), "README.md");
}

#[test]
fn project_encoding_is_independent_of_input_order() {
    let first = ProjectPayload::new(vec![file("b.rs", ".rs", "b"), file("a.rs", ".rs", "a")])
        .unwrap()
        .into_bytes();
    let second = ProjectPayload::new(vec![file("a.rs", ".rs", "a"), file("b.rs", ".rs", "b")])
        .unwrap()
        .into_bytes();
    assert_eq!(first, second);
}

#[test]
fn project_rejects_unsafe_or_non_portable_paths() {
    for path in [
        "",
        "/etc/passwd",
        "../secret",
        "src/./main.rs",
        "src\\main.rs",
        "C:/main.rs",
        "src//main.rs",
    ] {
        assert!(
            ProjectFilePayload::new(path, ".rs", "source").is_err(),
            "accepted {path}"
        );
    }
}

#[test]
fn project_rejects_duplicate_paths_and_empty_archive() {
    assert!(ProjectPayload::new(Vec::new()).is_err());
    assert!(ProjectPayload::new(vec![file("a.rs", ".rs", "a"), file("a.rs", ".rs", "b")]).is_err());
}

#[test]
fn project_rejects_file_count_over_limit() {
    let files = (0..=MAX_FILES)
        .map(|index| file(&format!("src/{index:04}.rs"), ".rs", ""))
        .collect();
    assert!(ProjectPayload::new(files).is_err());

    let mut encoded = ProjectPayload::new(vec![file("main.rs", ".rs", "")])
        .unwrap()
        .into_bytes();
    encoded[2..4].copy_from_slice(&((MAX_FILES + 1) as u16).to_be_bytes());
    assert!(ProjectPayload::from_bytes(&encoded).is_err());
}

#[test]
fn project_rejects_source_and_total_size_over_limits() {
    assert!(ProjectFilePayload::new("main.rs", ".rs", "x".repeat(MAX_SOURCE_LENGTH + 1),).is_err());

    let mut encoded = ProjectPayload::new(vec![file("main.rs", ".rs", "")])
        .unwrap()
        .into_bytes();
    encoded[4..8].copy_from_slice(&((MAX_TOTAL_SOURCE_LENGTH + 1) as u32).to_be_bytes());
    assert!(ProjectPayload::from_bytes(&encoded).is_err());
}

#[test]
fn project_rejects_truncated_trailing_and_tampered_lengths() {
    let encoded = ProjectPayload::new(vec![file("main.rs", ".rs", "fn main() {}")])
        .unwrap()
        .into_bytes();
    assert!(ProjectPayload::from_bytes(&encoded[..encoded.len() - 1]).is_err());
    let mut trailing = encoded.clone();
    trailing.push(0);
    assert!(ProjectPayload::from_bytes(&trailing).is_err());
    let mut wrong_total = encoded;
    wrong_total[7] += 1;
    assert!(ProjectPayload::from_bytes(&wrong_total).is_err());
}

#[test]
fn project_rejects_unknown_schema_and_encoding() {
    let encoded = ProjectPayload::new(vec![file("main.rs", ".rs", "source")])
        .unwrap()
        .into_bytes();
    let mut schema = encoded.clone();
    schema[0] += 1;
    assert!(ProjectPayload::from_bytes(&schema).is_err());
    let mut encoding = encoded;
    encoding[1] += 1;
    assert!(ProjectPayload::from_bytes(&encoding).is_err());
}
