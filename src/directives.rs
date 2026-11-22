//! Directive generation and close-match suggestions.
//!
//! A directive is the human-readable consequence of a `GroundingResult`. It is
//! derived only from the result's own diagnostics — absent identifiers, match
//! provenance, repeated slots — never from re-parsing the statement.

use crate::Statement;
use crate::types::{self, CloseMatch, Directive, GroundingStatus};

// ── Directive generation ──

pub fn generate_directives(
    result: &types::GroundingResult,
    stmt: &Statement,
    sig: &types::Signature,
) -> Vec<Directive> {
    let mut dirs = absent_identifier_directives(result, stmt);
    dirs.extend(unused_identifier_directives(result, stmt));

    match result.status {
        GroundingStatus::Grounded => {}

        GroundingStatus::Contradictory => {
            dirs.extend(contradiction_directives(result, stmt, sig));
        }

        GroundingStatus::Incomplete => {
            if let Some(best) = result.candidates.first() {
                if let Some(d) = short_argument_directive(best, stmt, sig) {
                    dirs.push(d);
                }
                dirs.extend(alias_directives(best, stmt));
            }
        }

        GroundingStatus::Ambiguous => {
            let Some(best) = result.candidates.first() else {
                return dirs;
            };
            dirs.extend(alias_directives(best, stmt));
            if dirs.is_empty() {
                dirs.push(Directive {
                    severity: "warning".to_string(),
                    element: stmt.source.clone(),
                    detail: format!(
                        "Grounded with low confidence ({:.2}). Consider using explicit task IDs.",
                        best.confidence,
                    ),
                    suggested_action: "Add explicit task ID references (e.g., 'T2.1') to the requirement statement.".to_string(),
                    close_matches: None,
                });
            }
        }

        GroundingStatus::Ungroundable => {
            let close_matches = find_close_matches(&stmt.text, sig);
            let suggestions: Vec<String> = close_matches
                .iter()
                .map(|m| format!("'{}' (alias: '{}')", m.constant, m.alias))
                .collect();

            let suggested = if suggestions.is_empty() {
                "Add a task ID reference (e.g., 'T5.1') or a known predicate keyword (BEFORE, AFTER, CONCURRENTLY, IF...THEN, ALWAYS, AT MOST ONE).".to_string()
            } else {
                format!(
                    "Close matches: {}. Use one of these task IDs explicitly in the requirement.",
                    suggestions.join(", ")
                )
            };

            dirs.push(Directive {
                severity: "blocker".to_string(),
                element: stmt.source.clone(),
                detail: format!(
                    "Could not ground: '{}'. No matching task or predicate found.",
                    stmt.text,
                ),
                suggested_action: suggested,
                close_matches: Some(close_matches),
            });
        }
    }

    dirs
}

/// One directive per identifier the statement references that this signature
/// does not declare. This is the diagnosis for most "short argument list"
/// failures: the second task exists, it just belongs to something else.
fn absent_identifier_directives(
    result: &types::GroundingResult,
    stmt: &Statement,
) -> Vec<Directive> {
    result
        .absent_identifiers
        .iter()
        .map(|id| Directive {
            severity: "blocker".to_string(),
            element: stmt.source.clone(),
            detail: format!("'{}' is not a task of this signature.", id),
            suggested_action: format!(
                "Add '{}' to tasks.md, or reference a task this change declares.",
                id,
            ),
            close_matches: None,
        })
        .collect()
}

/// The statement names a task the grounding never bound. That is our own
/// weakness — a slot was filled from something weaker while an explicit
/// reference went unused — so the suggestion never asks the author to write
/// more task IDs.
fn unused_identifier_directives(
    result: &types::GroundingResult,
    stmt: &Statement,
) -> Vec<Directive> {
    result
        .unused_identifiers
        .iter()
        .map(|id| Directive {
            severity: "warning".to_string(),
            element: stmt.source.clone(),
            detail: format!(
                "'{}' is named in this requirement but was not bound to the constraint.",
                id,
            ),
            suggested_action: format!(
                "Check that the constraint relates '{}' to a different task, or that another \
                 task does not shadow it.",
                id,
            ),
            close_matches: None,
        })
        .collect()
}

/// The statement repeats one constant across slots that require different
/// constants, so the constraint it describes is meaningless.
fn contradiction_directives(
    result: &types::GroundingResult,
    stmt: &Statement,
    sig: &types::Signature,
) -> Vec<Directive> {
    result
        .candidates
        .iter()
        .filter_map(|atom| {
            let slots: Vec<&str> = sig
                .predicates
                .iter()
                .find(|p| p.name == atom.predicate)?
                .arguments
                .iter()
                .map(|a| a.name.as_str())
                .collect();
            let (first, second) = atom.arguments.iter().enumerate().find_map(|(i, a)| {
                atom.arguments
                    .iter()
                    .skip(i + 1)
                    .position(|b| b.constant == a.constant)
                    .map(|j| (i, i + 1 + j))
            })?;
            let constant = atom.arguments.get(first).map(|a| a.constant.clone())?;
            Some(Directive {
                severity: "blocker".to_string(),
                element: stmt.source.clone(),
                detail: format!(
                    "{} requires two different tasks, but '{}' fills both '{}' and '{}'.",
                    atom.predicate,
                    constant,
                    slots.get(first).copied().unwrap_or("?"),
                    slots.get(second).copied().unwrap_or("?"),
                ),
                suggested_action: format!(
                    "Name a second task: '{}' cannot be {} itself.",
                    constant, atom.predicate,
                ),
                close_matches: None,
            })
        })
        .collect()
}

/// A predicate is declared with more slots than the statement filled.
fn short_argument_directive(
    atom: &types::GroundedAtom,
    stmt: &Statement,
    sig: &types::Signature,
) -> Option<Directive> {
    let pred = sig.predicates.iter().find(|p| p.name == atom.predicate)?;
    if atom.arguments.len() >= pred.arguments.len() {
        return None;
    }
    let found: Vec<&str> = atom.names();
    Some(Directive {
        severity: "warning".to_string(),
        element: stmt.source.clone(),
        detail: format!(
            "{} takes {} tasks ({}), but only '{}' was found.",
            atom.predicate,
            pred.arguments.len(),
            pred.arguments
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            if found.is_empty() {
                "none".to_string()
            } else {
                found.join(", ")
            },
        ),
        suggested_action: "Reference every task the constraint relates. Example: 'T2.1 SHALL complete BEFORE T3.1 SHALL run'.".to_string(),
        close_matches: None,
    })
}

/// One directive per argument that matched by alias: the caller can see that
/// the binding rests on a shared word rather than an explicit task ID.
fn alias_directives(atom: &types::GroundedAtom, stmt: &Statement) -> Vec<Directive> {
    atom.arguments
        .iter()
        .filter_map(|arg| match &arg.matched {
            types::MatchKind::Name => None,
            types::MatchKind::Alias(alias) => Some(Directive {
                severity: "warning".to_string(),
                element: stmt.source.clone(),
                detail: format!(
                    "Argument matched '{}' via alias '{}' (confidence: {:.2}).",
                    arg.constant, alias, atom.confidence,
                ),
                suggested_action: format!(
                    "Replace the NL reference with '{}' or add '({})' after it.",
                    arg.constant, arg.constant,
                ),
                close_matches: None,
            }),
        })
        .collect()
}

/// Find close matches for ungroundable text by comparing against constant names and aliases.
pub fn find_close_matches(text: &str, sig: &types::Signature) -> Vec<CloseMatch> {
    let lower = text.to_lowercase();
    let mut matches: Vec<CloseMatch> = sig
        .constants
        .iter()
        .filter_map(|c| {
            // Compare against the constant name itself (e.g., "T1.3")
            let name_score = strsim(&lower, &c.name.to_lowercase());
            // Compare against all aliases
            let best_alias = c
                .aliases
                .iter()
                .map(|a| (a, strsim(&lower, a)))
                .max_by(|(_, s1), (_, s2)| s1.partial_cmp(s2).unwrap_or(std::cmp::Ordering::Equal));

            let (best_alias_str, best_alias_score) = match best_alias {
                Some((a, s)) if s > name_score => (a.clone(), s),
                _ => (c.name.clone(), name_score),
            };

            if best_alias_score > 0.4 {
                Some(CloseMatch {
                    constant: c.name.clone(),
                    alias: best_alias_str,
                    similarity: best_alias_score,
                })
            } else {
                None
            }
        })
        .collect();
    matches.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    matches.truncate(5);
    matches
}

/// Simple string similarity (Dice coefficient on bigrams).
pub fn strsim(a: &str, b: &str) -> f64 {
    let a_bigrams: Vec<Vec<char>> = a
        .as_bytes()
        .windows(2)
        .map(|w| w.iter().map(|&c| c as char).collect())
        .collect();
    let b_bigrams: Vec<Vec<char>> = b
        .as_bytes()
        .windows(2)
        .map(|w| w.iter().map(|&c| c as char).collect())
        .collect();

    if a_bigrams.is_empty() || b_bigrams.is_empty() {
        return 0.0;
    }

    let intersection = a_bigrams.iter().filter(|bg| b_bigrams.contains(bg)).count();
    (2.0 * intersection as f64) / (a_bigrams.len() + b_bigrams.len()) as f64
}

#[cfg(test)]
mod tests;
