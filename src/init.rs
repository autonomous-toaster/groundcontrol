//! init subcommand: merge grounding rules into openspec/config.yaml.

use std::collections::BTreeMap;

// ── Init command ──

pub const GROUNDCONTROL_CONTEXT: &str = "\
Every task reference in a spec MUST map to a real task ID.\n\
Use explicit N.M IDs (e.g. 'T2.1') — not descriptions like 'the migration step'.\n\
Task descriptions in tasks.md become aliases for fuzzy matching, so write\n\
descriptive descriptions that capture the task's purpose.";

pub fn groundcontrol_rules() -> BTreeMap<String, Vec<String>> {
    let mut rules = BTreeMap::new();
    rules.insert(
        "specs".to_string(),
        vec![
            "Reference tasks by explicit ID: 'T2.1' not 'the migration step'".to_string(),
            "Use parenthetical syntax for task IDs: 'the migration step (T2.1)'".to_string(),
            "BEFORE requires two task IDs: 'T2.1 SHALL complete BEFORE T3.1 SHALL run'".to_string(),
            "ALWAYS requires one task ID: 'ALWAYS T2.1 SHALL validate input'".to_string(),
        ],
    );
    rules.insert(
        "tasks".to_string(),
        vec!["Write descriptive task descriptions — they become aliases for grounding".to_string()],
    );
    rules
}

pub fn run_init(project_root: Option<String>) -> anyhow::Result<()> {
    let root = match project_root {
        Some(p) => std::path::PathBuf::from(p),
        None => std::env::current_dir()?,
    };
    let config_path = root.join("openspec").join("config.yaml");
    merge_config(&config_path)?;
    println!("✓ groundcontrol init: {}", config_path.display());
    Ok(())
}

/// Merge groundcontrol's context and rules into openspec/config.yaml using YAML-aware merge.
/// If the file does not exist, creates it with schema + groundcontrol content.
pub fn merge_config(path: &std::path::Path) -> anyhow::Result<()> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let config = create_fresh_config();
        std::fs::write(path, config)?;
        return Ok(());
    }

    let content = std::fs::read_to_string(path)?;
    let existing: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", path.display(), e))?;

    let merged = yaml_merge(&existing, GROUNDCONTROL_CONTEXT, &groundcontrol_rules());
    let output = serde_yaml::to_string(&merged)
        .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;

    let output = format!("# groundcontrol init\n{}", output);
    std::fs::write(path, output)?;
    Ok(())
}

/// Create a fresh config with schema + groundcontrol context and rules.
pub fn create_fresh_config() -> String {
    let mut config = serde_yaml::Mapping::new();
    config.insert(
        serde_yaml::Value::String("schema".to_string()),
        serde_yaml::Value::String("spec-driven".to_string()),
    );
    let merged = yaml_merge(
        &serde_yaml::Value::Mapping(config),
        GROUNDCONTROL_CONTEXT,
        &groundcontrol_rules(),
    );
    let output = serde_yaml::to_string(&merged).unwrap();
    format!("# groundcontrol init\n{}", output)
}

/// Merge new context and rules into an existing YAML value.
/// Context is appended with a blank line separator.
/// Rules are merged per artifact type, deduplicating by exact string match.
pub fn yaml_merge(
    existing: &serde_yaml::Value,
    new_context: &str,
    new_rules: &BTreeMap<String, Vec<String>>,
) -> serde_yaml::Value {
    let mut merged = existing.clone();

    // Merge context: append with blank line separator, skip if already present
    match merged.get_mut("context") {
        Some(serde_yaml::Value::String(ctx)) => {
            let trimmed_new = new_context.trim();
            if !ctx.contains(trimmed_new) {
                let combined = format!("{}\n\n{}", ctx.trim(), trimmed_new);
                *ctx = combined;
            }
        }
        _ => {
            merged["context"] = serde_yaml::Value::String(new_context.to_string());
        }
    }

    // Merge rules: add new items per artifact type, deduplicating
    let mut rules = match merged.get("rules") {
        Some(serde_yaml::Value::Mapping(m)) => m.clone(),
        _ => serde_yaml::Mapping::new(),
    };

    for (artifact_type, new_items) in new_rules {
        let key = serde_yaml::Value::String(artifact_type.clone());
        let existing_items = match rules.get(&key) {
            Some(serde_yaml::Value::Sequence(s)) => s.clone(),
            _ => serde_yaml::Sequence::new(),
        };

        let mut merged_items = existing_items.clone();
        for item in new_items {
            let item_val = serde_yaml::Value::String(item.clone());
            if !merged_items.contains(&item_val) {
                merged_items.push(item_val);
            }
        }

        rules.insert(key, serde_yaml::Value::Sequence(merged_items));
    }

    merged["rules"] = serde_yaml::Value::Mapping(rules);
    merged
}

#[cfg(test)]
mod tests;
