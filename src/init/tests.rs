use super::*;

#[test]
fn test_merge_config_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("openspec").join("config.yaml");
    assert!(!path.exists());
    merge_config(&path).unwrap();
    assert!(path.exists());
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("# groundcontrol init"));
    assert!(content.contains("Reference tasks by explicit ID"));
    assert!(content.contains("Write descriptive task descriptions"));
    // Context must not cite the tool
    assert!(!content.contains("groundcontrol checks"));
}

#[test]
fn test_merge_config_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("openspec").join("config.yaml");
    merge_config(&path).unwrap();
    let after_first = std::fs::read_to_string(&path).unwrap();
    // Run again — should not add duplicates
    merge_config(&path).unwrap();
    let after_second = std::fs::read_to_string(&path).unwrap();
    assert_eq!(after_first, after_second);
}

#[test]
fn test_merge_config_preserves_existing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("openspec").join("config.yaml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let existing = "schema: spec-driven\ncontext: |-\n  Use conventional commits\n";
    std::fs::write(&path, existing).unwrap();
    merge_config(&path).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    // Original content preserved
    assert!(content.contains("Use conventional commits"));
    // New content added
    assert!(content.contains("Reference tasks by explicit ID"));
    assert!(content.contains("Write descriptive task descriptions"));
    // Context must not cite the tool
    assert!(!content.contains("groundcontrol checks"));
}

#[test]
fn test_merge_config_rules_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("openspec").join("config.yaml");
    merge_config(&path).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();
    // Context block — tool-agnostic
    assert!(content.contains("Every task reference in a spec MUST map to a real task ID"));
    assert!(content.contains("Use explicit N.M IDs"));
    assert!(content.contains("descriptive descriptions that capture the task's purpose"));
    // Specs rules
    assert!(content.contains("Reference tasks by explicit ID"));
    assert!(content.contains("Use parenthetical syntax"));
    assert!(content.contains("BEFORE requires two task IDs"));
    assert!(content.contains("ALWAYS requires one task ID"));
    // Tasks rules
    assert!(content.contains("Write descriptive task descriptions"));
    // Must not cite the tool
    assert!(!content.contains("groundcontrol checks"));
}

#[test]
fn test_yaml_merge_context_appends() {
    let existing: serde_yaml::Value =
        serde_yaml::from_str("context: |-\n  Original context\n").unwrap();
    let mut rules = BTreeMap::new();
    rules.insert("specs".to_string(), vec![]);
    let merged = yaml_merge(&existing, "New context", &rules);
    let ctx = merged["context"].as_str().unwrap();
    assert!(ctx.contains("Original context"));
    assert!(ctx.contains("New context"));
}

#[test]
fn test_yaml_merge_rules_dedup() {
    let existing: serde_yaml::Value =
        serde_yaml::from_str("rules:\n  specs:\n    - \"Existing rule\"\n").unwrap();
    let mut rules = BTreeMap::new();
    rules.insert(
        "specs".to_string(),
        vec!["Existing rule".to_string(), "New rule".to_string()],
    );
    let merged = yaml_merge(&existing, "", &rules);

    let specs = merged["rules"]["specs"].as_sequence().unwrap();
    assert_eq!(specs.len(), 2);
    assert!(specs.iter().any(|v| v.as_str() == Some("Existing rule")));
    assert!(specs.iter().any(|v| v.as_str() == Some("New rule")));
}
