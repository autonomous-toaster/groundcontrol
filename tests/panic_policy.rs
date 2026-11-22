//! Guards the panic policy itself.
//!
//! The lints in Cargo.toml are what stop a panic-capable construct from being
//! written into this crate. A lint table is configuration, though, and
//! configuration is easy to delete quietly in a refactor. This test fails if the
//! policy is weakened, so removing it has to be a deliberate, visible act.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

const REQUIRED: &[(&str, &str)] = &[
    ("rust", "unsafe_code"),
    ("clippy", "panic"),
    ("clippy", "unwrap_used"),
    ("clippy", "expect_used"),
    ("clippy", "indexing_slicing"),
    ("clippy", "string_slice"),
    ("clippy", "todo"),
    ("clippy", "unimplemented"),
    ("clippy", "unreachable"),
];

fn cargo_toml() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    std::fs::read_to_string(&path).expect("read Cargo.toml")
}

/// Value declared for `key` in the `[lints.<table>]` section, if any.
fn declared(table: &str, key: &str) -> Option<String> {
    let toml = cargo_toml();
    let header = format!("[lints.{table}]");
    let mut in_section = false;
    for line in toml.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_section = line == header;
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some((k, v)) = line.split_once('=')
            && k.trim() == key
        {
            return Some(v.trim().trim_matches('"').to_string());
        }
    }
    None
}

#[test]
fn panic_lints_are_still_declared() {
    let mut missing = Vec::new();
    for (table, key) in REQUIRED {
        match declared(table, key).as_deref() {
            Some("deny") | Some("forbid") => {}
            other => missing.push(format!("[lints.{table}] {key} = {other:?}")),
        }
    }
    assert!(
        missing.is_empty(),
        "panic policy weakened; these lints must stay denied:\n  {}",
        missing.join("\n  ")
    );
}

#[test]
fn every_test_module_that_opts_out_says_why() {
    // An `#![allow(...)]` for these lints is only acceptable in test code.
    // Assert the allow appears only in files or modules that are test-only.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            if !text.contains("clippy::indexing_slicing") || !text.contains("#![allow(") {
                continue;
            }
            let is_test_file = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with("tests.rs"));
            let is_test_module = text.contains("mod tests {");
            if !is_test_file && !is_test_module {
                offenders.push(path.display().to_string());
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "the panic-lint opt-out must not appear outside test code: {offenders:?}"
    );
}
