//! Spec parsing: extract requirement statements from spec.md files.

use std::path::Path;

use crate::Statement;

// ── Spec parsing ──

pub fn parse_specs(specs_dir: &Path) -> anyhow::Result<Vec<Statement>> {
    let mut statements = Vec::new();

    if !specs_dir.exists() {
        return Ok(statements);
    }

    collect_spec_files(specs_dir, specs_dir, &mut statements)?;
    Ok(statements)
}

pub fn collect_spec_files(base: &Path, dir: &Path, out: &mut Vec<Statement>) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_spec_files(base, &path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md")
            && path.file_stem().and_then(|s| s.to_str()) == Some("spec")
        {
            let source = std::fs::read_to_string(&path)?;
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            extract_requirements(&source, &rel, out);
        }
    }
    Ok(())
}

/// Simple line-based extraction of requirement SHALL statements from spec.md.
/// This is a lightweight alternative to full tree-sitter parsing of specs.
pub fn extract_requirements(source: &str, rel_path: &str, out: &mut Vec<Statement>) {
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    let mut in_requirement = false;
    let mut req_line = 0;

    while let Some(raw) = lines.get(i) {
        let line = raw.trim();

        if line.starts_with("### Requirement:") || line.starts_with("### requirement:") {
            in_requirement = true;
            req_line = i + 1; // 1-indexed
            i += 1;
            continue;
        }

        if in_requirement {
            // Collect body paragraphs until next heading
            let mut body = String::new();
            while let Some(raw) = lines.get(i) {
                let bl = raw.trim();
                if bl.starts_with("###") || bl.starts_with("####") {
                    break;
                }
                if !bl.is_empty() {
                    if !body.is_empty() {
                        body.push(' ');
                    }
                    body.push_str(bl);
                }
                i += 1;
            }

            // Extract SHALL statement from body
            let statement = extract_shall(&body);
            if !statement.is_empty() {
                out.push(Statement {
                    source: format!("{}#L{}", rel_path, req_line),
                    text: statement,
                });
            }
            in_requirement = false;
            continue;
        }

        i += 1;
    }
}

/// Extract the first sentence containing SHALL/MUST from body text.
/// Avoids splitting on periods within task IDs (e.g., "T3.1").
pub fn extract_shall(body: &str) -> String {
    // Split on sentence boundaries: ". " (period+space), "! ", "? ", or end of string.
    // Skip periods that are part of task IDs like "T2.1 " (digit.space → not a sentence end).
    let sentences: Vec<&str> = body
        .split(['!', '?'])
        .flat_map(|part| {
            let mut result = Vec::new();
            let mut start = 0;
            let bytes = part.as_bytes();
            let len = part.len();
            let mut i = 0;
            while i < len.saturating_sub(1) {
                let (Some(dot), Some(space)) = (bytes.get(i), bytes.get(i + 1)) else {
                    break;
                };
                if *dot == b'.' && *space == b' ' {
                    // Check if this period is part of a task ID (digit before, digit after space)
                    let is_task_id = i > 0
                        && bytes.get(i - 1).is_some_and(|b| b.is_ascii_digit())
                        && bytes.get(i + 2).is_some_and(|b| b.is_ascii_digit());
                    if !is_task_id && let Some(head) = part.get(start..i) {
                        result.push(head);
                        start = i + 2;
                    }
                    i += 1;
                } else {
                    i += 1;
                }
            }
            if let Some(tail) = part.get(start..) {
                result.push(tail);
            }
            result
        })
        .collect();
    for s in &sentences {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            continue;
        }
        let upper = trimmed.to_uppercase();
        if upper.contains("SHALL") || upper.contains("MUST") {
            return trimmed.to_string();
        }
    }
    // Fallback: return the first non-empty line
    body.lines().next().unwrap_or("").trim().to_string()
}
