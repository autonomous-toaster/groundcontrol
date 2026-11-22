//! groundcontrol — Ground Natural Language Into System Signatures for Temporal Logic.
//!
//! Standalone CLI that parses OpenSpec plans and checks whether NL requirement
//! statements can be grounded to the plan's task vocabulary.
//!
//! Usage:
//!   groundcontrol check <change-name>        # by name (openspec/changes/<name>)
//!   groundcontrol check ./path/to/change     # by path
//!   groundcontrol check                      # auto-detect
//!   groundcontrol check --stdin             # read plan from stdin
//!
//!   groundcontrol signature <change-name>    # just emit the signature JSON

mod directives;
mod grounders;
mod init;
mod parse;
mod specs;
mod types;

use std::path::Path;

use crate::specs::parse_specs;
use clap::{Parser, Subcommand};
use init::run_init;
use types::{CheckOutput, CheckSummary, GroundedInput, Grounder, GroundingStatus};

struct Statement {
    source: String,
    text: String,
}

#[derive(Parser)]
#[command(
    name = "groundcontrol",
    about = "Ground NL into system signatures for temporal logic"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check grounding quality of a plan's specs
    Check {
        /// Change name, path, or omit for auto-detect
        change: Option<String>,
        /// Output format: json or human (default: human)
        #[arg(long, default_value = "human")]
        format: String,
        /// Read plan from stdin
        #[arg(long)]
        stdin: bool,
    },
    /// Emit the signature JSON for a plan
    Signature {
        /// Change name, path, or omit for auto-detect
        change: Option<String>,
        /// Read plan from stdin
        #[arg(long)]
        stdin: bool,
    },
    /// Initialize openspec/config.yaml with grounding rules
    Init {
        /// Project root directory (default: current dir)
        #[arg(long)]
        project_root: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check {
            change,
            format,
            stdin,
        } => run_check(change, &format, stdin)?,
        Commands::Signature { change, stdin } => run_signature(change, stdin)?,
        Commands::Init { project_root } => run_init(project_root)?,
    }

    Ok(())
}

fn run_check(change: Option<String>, format: &str, stdin: bool) -> anyhow::Result<()> {
    let project_root = std::env::current_dir()?;
    let changes = resolve_changes(change.as_deref(), &project_root, stdin)?;

    let mut all_outputs = Vec::new();

    if changes.is_empty() {
        // No changes found — warn on stderr, exit 0
        eprintln!("No active changes found.");
        return Ok(());
    }

    for (change_dir, change_name) in &changes {
        let output = check_single(change_dir, change_name)?;
        all_outputs.push(output);
    }

    // Merge outputs
    let merged = merge_outputs(&all_outputs);
    let has_issues = merged.summary.ungroundable > 0
        || merged.summary.ambiguous > 0
        || merged.summary.incomplete > 0
        || merged.summary.contradictory > 0;
    // A grounded statement that left a named task unbound is a grounding
    // weakness worth showing, but not a spec defect: report it without failing.
    let has_notes = merged
        .inputs
        .iter()
        .any(|i| !i.unused_identifiers.is_empty());

    if has_issues || has_notes {
        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&merged)?),
            _ => print_human(&merged),
        }
    }
    if has_issues {
        std::process::exit(1);
    }

    Ok(())
}

fn check_single(change_dir: &std::path::Path, change_name: &str) -> anyhow::Result<CheckOutput> {
    let tasks_path = change_dir.join("tasks.md");
    let tasks_source = std::fs::read_to_string(&tasks_path)
        .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", tasks_path.display(), e))?;
    let sig = parse::signature_from_tasks(&tasks_source, &tasks_path)?;

    let specs_dir = change_dir.join("specs");
    let statements = parse_specs(&specs_dir)?;

    let grounder = grounders::RuleGrounder;
    let mut inputs = Vec::new();
    for stmt in &statements {
        let result = grounder.ground(&stmt.text, &sig);
        let directives = directives::generate_directives(&result, stmt, &sig);
        inputs.push(GroundedInput {
            source: stmt.source.clone(),
            text: stmt.text.clone(),
            groundings: result.candidates,
            status: result.status,
            absent_identifiers: result.absent_identifiers,
            unused_identifiers: result.unused_identifiers,
            directives,
        });
    }

    let summary = CheckSummary {
        total: inputs.len(),
        grounded: inputs
            .iter()
            .filter(|i| i.status == GroundingStatus::Grounded)
            .count(),
        ambiguous: inputs
            .iter()
            .filter(|i| i.status == GroundingStatus::Ambiguous)
            .count(),
        ungroundable: inputs
            .iter()
            .filter(|i| i.status == GroundingStatus::Ungroundable)
            .count(),
        incomplete: inputs
            .iter()
            .filter(|i| i.status == GroundingStatus::Incomplete)
            .count(),
        contradictory: inputs
            .iter()
            .filter(|i| i.status == GroundingStatus::Contradictory)
            .count(),
    };

    Ok(CheckOutput {
        plan_name: change_name.to_string(),
        signature: sig.summary(),
        inputs,
        summary,
    })
}

fn merge_outputs(outputs: &[CheckOutput]) -> CheckOutput {
    if outputs.is_empty() {
        return CheckOutput {
            plan_name: String::new(),
            signature: types::SignatureSummary {
                tasks: 0,
                predicates: 0,
                types: 0,
            },
            inputs: vec![],
            summary: CheckSummary {
                total: 0,
                grounded: 0,
                ambiguous: 0,
                ungroundable: 0,
                incomplete: 0,
                contradictory: 0,
            },
        };
    }
    if let [only] = outputs {
        return only.clone();
    }

    let mut names: Vec<&str> = Vec::new();
    let mut all_inputs = Vec::new();
    let mut total = 0;
    let mut grounded = 0;
    let mut ambiguous = 0;
    let mut ungroundable = 0;
    let mut incomplete = 0;
    let mut contradictory = 0;

    for o in outputs {
        names.push(o.plan_name.as_str());
        for input in &o.inputs {
            let mut prefixed = input.clone();
            prefixed.source = format!("{}: {}", o.plan_name, prefixed.source);
            all_inputs.push(prefixed);
        }
        total += o.summary.total;
        grounded += o.summary.grounded;
        ambiguous += o.summary.ambiguous;
        ungroundable += o.summary.ungroundable;
        incomplete += o.summary.incomplete;
        contradictory += o.summary.contradictory;
    }

    CheckOutput {
        plan_name: names.join(", "),
        signature: types::SignatureSummary {
            tasks: 0,
            predicates: 0,
            types: 0,
        },
        inputs: all_inputs,
        summary: CheckSummary {
            total,
            grounded,
            ambiguous,
            ungroundable,
            incomplete,
            contradictory,
        },
    }
}

fn run_signature(change: Option<String>, stdin: bool) -> anyhow::Result<()> {
    let project_root = std::env::current_dir()?;
    let changes = resolve_changes(change.as_deref(), &project_root, stdin)?;
    let Some((change_dir, _)) = changes.first() else {
        anyhow::bail!("No active changes found.");
    };

    let tasks_path = change_dir.join("tasks.md");
    let tasks_source = std::fs::read_to_string(&tasks_path)
        .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", tasks_path.display(), e))?;
    let sig = parse::signature_from_tasks(&tasks_source, &tasks_path)?;

    println!("{}", serde_json::to_string_pretty(&sig)?);
    Ok(())
}

// ── Input resolution ──

fn resolve_changes(
    change: Option<&str>,
    project_root: &Path,
    stdin: bool,
) -> anyhow::Result<Vec<(std::path::PathBuf, String)>> {
    if stdin {
        anyhow::bail!("stdin mode not yet implemented");
    }

    match change {
        Some(name) => {
            // Try as a change name in openspec/changes/<name>
            let as_change = project_root.join("openspec").join("changes").join(name);
            if as_change.join("tasks.md").exists() {
                return Ok(vec![(as_change, name.to_string())]);
            }
            // Try as a direct path
            let as_path = Path::new(name);
            if as_path.join("tasks.md").exists() {
                let label = as_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(name)
                    .to_string();
                return Ok(vec![(as_path.to_path_buf(), label)]);
            }
            // Try as a project root with openspec/changes/
            let changes_dir = as_path.join("openspec").join("changes");
            if changes_dir.exists() {
                let mut changes: Vec<_> = std::fs::read_dir(&changes_dir)
                    .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", changes_dir.display(), e))?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter(|e| e.path().join("tasks.md").exists())
                    .map(|e| (e.path(), e.file_name().to_string_lossy().to_string()))
                    .collect();
                changes.sort_by(|a, b| a.1.cmp(&b.1));
                return Ok(changes); // empty vec is valid
            }
            // Try as a changes directory (contains subdirs with tasks.md)
            if as_path.is_dir() {
                let mut changes: Vec<_> = std::fs::read_dir(as_path)
                    .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", as_path.display(), e))?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter(|e| e.path().join("tasks.md").exists())
                    .map(|e| (e.path(), e.file_name().to_string_lossy().to_string()))
                    .collect();
                changes.sort_by(|a, b| a.1.cmp(&b.1));
                if !changes.is_empty() {
                    return Ok(changes);
                }
            }
            anyhow::bail!("Change '{}' not found (no tasks.md)", name);
        }
        None => {
            // Auto-detect: return ALL changes in openspec/changes/
            let changes_dir = project_root.join("openspec").join("changes");
            if changes_dir.exists() {
                let mut changes: Vec<_> = std::fs::read_dir(&changes_dir)
                    .map_err(|e| anyhow::anyhow!("Cannot read {}: {}", changes_dir.display(), e))?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter(|e| e.path().join("tasks.md").exists())
                    .map(|e| (e.path(), e.file_name().to_string_lossy().to_string()))
                    .collect();
                changes.sort_by(|a, b| a.1.cmp(&b.1));
                return Ok(changes); // empty vec is valid — caller handles it
            }
            Ok(vec![]) // no openspec/changes/ — empty is valid
        }
    }
}
// ── Human-readable output ──
// Grouped: successes first (quiet), then errors (loud).

fn print_human(output: &CheckOutput) {
    // Pass 1: successes — quiet one-liners
    for input in &output.inputs {
        if input.status != GroundingStatus::Grounded {
            continue;
        }
        if let Some(g) = input.groundings.first() {
            println!(
                "\u{2713} {} \u{2192} {}({})",
                input.source,
                g.predicate,
                g.names().join(", ")
            );
            if !input.unused_identifiers.is_empty() {
                println!(
                    "   unbound although named: {}",
                    input.unused_identifiers.join(", ")
                );
            }
        }
    }

    // Pass 2: errors — loud with full details
    for input in &output.inputs {
        if input.status == GroundingStatus::Grounded {
            continue;
        }
        let marker = match input.status {
            GroundingStatus::Ungroundable | GroundingStatus::Contradictory => "\u{2717}",
            _ => "\u{26A0}",
        };
        println!("{} {} [{}]", marker, input.source, input.status.as_str());
        println!("   \"{}\"", input.text);
        if let Some(g) = input.groundings.first() {
            let args: Vec<String> = g
                .arguments
                .iter()
                .map(|a| match &a.matched {
                    types::MatchKind::Name => a.constant.clone(),
                    types::MatchKind::Alias(alias) => format!("{} (alias '{}')", a.constant, alias),
                })
                .collect();
            println!(
                "   \u{2192} {}({}) @ {:.2}{}",
                g.predicate,
                args.join(", "),
                g.confidence,
                if g.exact { "" } else { " (not exact)" }
            );
        }
        for d in &input.directives {
            println!("   {}", d.detail);
            if let Some(matches) = &d.close_matches
                && !matches.is_empty()
            {
                println!(
                    "   Close matches: {}",
                    matches
                        .iter()
                        .map(|m| format!("'{}' (alias: '{}')", m.constant, m.alias))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            println!("   Suggestion: {}", d.suggested_action);
        }
    }

    // Summary line only if there are issues
    let has_issues = output.summary.ungroundable > 0 || output.summary.ambiguous > 0;
    if has_issues {
        println!(
            "Summary: {} total | {} grounded | {} incomplete | {} ambiguous | {} contradictory | {} ungroundable",
            output.summary.total,
            output.summary.grounded,
            output.summary.incomplete,
            output.summary.ambiguous,
            output.summary.contradictory,
            output.summary.ungroundable
        );
    }
}
#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::string_slice,
        clippy::panic
    )]
    use crate::specs::extract_shall;

    #[test]
    fn test_extract_shall_simple() {
        let body = "T1.1 SHALL complete BEFORE T1.2 SHALL run.";
        let result = extract_shall(body);
        // Trailing period is preserved (split on ". " not just ".")
        assert_eq!(result, "T1.1 SHALL complete BEFORE T1.2 SHALL run.");
    }

    #[test]
    fn test_extract_shall_with_task_ids() {
        let body = "T3.1 SHALL parse tasks.md BEFORE T4.1 SHALL ground NL statements. T3.1 SHALL produce a Signature.";
        let result = extract_shall(body);
        // Should not split on periods inside task IDs like "T3.1"
        assert!(result.contains("T3.1"));
        assert!(result.contains("BEFORE"));
        assert!(result.contains("T4.1"));
    }

    #[test]
    fn test_extract_shall_multiple_sentences() {
        let body = "First sentence. T5.1 SHALL generate LTL. Third sentence.";
        let result = extract_shall(body);
        assert_eq!(result, "T5.1 SHALL generate LTL");
    }

    #[test]
    fn test_extract_shall_no_shall() {
        let body = "Just some text without any SHALL or MUST keywords.";
        let result = extract_shall(body);
        assert_eq!(result, "Just some text without any SHALL or MUST keywords.");
    }

    #[test]
    fn test_extract_shall_must() {
        let body = "The system MUST handle errors. Other text.";
        let result = extract_shall(body);
        assert_eq!(result, "The system MUST handle errors");
    }
}
