//! Rule-based grounder: keyword matching + positional heuristics.
//!
//! No ML dependency. Works well for structured OpenSpec specs
//! that use explicit task IDs (T1.3) and temporal keywords.

//!
//! Grounding is structural: the result reports *why* a statement failed to
//! ground (absent identifier, short argument list, repeated slot, ambiguous
//! alias match) rather than collapsing every failure into one status.

use crate::types::{
    BoundArgument, ConstantDef, GroundedAtom, Grounder, GroundingResult, GroundingStatus,
    PredicateDef, Signature,
};

/// Predicate keyword groups and their argument ordering.
const PREDICATE_KEYWORDS: &[(&str, &[&str])] = &[
    (
        "BEFORE",
        &["before", "complete before", "must finish before"],
    ),
    ("AFTER", &["after", "only after"]),
    (
        "CONCURRENTLY",
        &[
            "concurrently",
            "in parallel",
            "simultaneously",
            "at the same time",
        ],
    ),
    ("IF_THEN", &["if", "unless", "in case of", "when ... then"]),
    ("ALWAYS", &["always", "throughout", "at all times"]),
    (
        "AT_MOST_ONE",
        &[
            "at most one",
            "mutually exclusive",
            "not concurrently",
            "not together",
        ],
    ),
];

pub struct RuleGrounder;

impl Grounder for RuleGrounder {
    fn ground(&self, nl: &str, sig: &Signature) -> GroundingResult {
        let lower = nl.to_lowercase();
        let absent_identifiers = extract_absent_identifiers(nl, sig);
        let mut candidates = Vec::new();

        for (pred_name, keywords) in PREDICATE_KEYWORDS {
            // 1. Predicate matching: does the text contain any keyword for this predicate?
            let matched = keywords.iter().any(|kw| lower.contains(kw));
            if !matched {
                continue;
            }

            // 2. Argument extraction: find all matching constants in the text
            let matched_constants: Vec<&ConstantDef> = sig
                .constants
                .iter()
                .filter(|c| appears_in(c, nl, &lower))
                .collect();

            // 3. Find the predicate definition
            let pred_def = sig.predicates.iter().find(|p| p.name == *pred_name);

            // 4. Assign arguments based on predicate-specific positional heuristics.
            // An alias match is a weak signal: when the text also names constants
            // this signature does not declare, the alias may well describe one of
            // those instead. Prefer "unknown constant" over a guessed binding.
            let allow_alias = absent_identifiers.is_empty();
            let args = assign_arguments(pred_name, &matched_constants, nl, &lower, allow_alias);

            // 5. Distinctness: a predicate that distinguishes its slots must not
            // receive the same constant twice.
            let repeated =
                pred_def.is_some_and(distinguishes_slots) && has_repeated_constant(&args);

            // 6. Structural confidence scoring
            let exact = !args.is_empty() && args.iter().all(BoundArgument::is_exact);
            let confidence = if args.is_empty() {
                0.0
            } else if repeated {
                0.5
            } else {
                match pred_def.map(|p| p.arguments.len()) {
                    Some(expected) if args.len() == expected => {
                        if exact {
                            0.95
                        } else {
                            0.9
                        }
                    }
                    None if !exact => 0.5,
                    None => 0.8,
                    Some(_) => 0.5,
                }
            };

            if confidence > 0.0 {
                candidates.push(GroundedAtom {
                    predicate: pred_name.to_string(),
                    arguments: args,
                    exact,
                    confidence,
                });
            }
        }

        // Sort by confidence descending
        candidates.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let status = classify(sig, &candidates, &absent_identifiers);

        GroundingResult {
            candidates,
            status,
            absent_identifiers,
        }
    }
}

/// Decide the overall status. Order matters: a contradiction is reported even
/// when another candidate is grounded, and an explainable shortfall is
/// `Incomplete` rather than the misleading `Ambiguous`.
fn classify(
    sig: &Signature,
    candidates: &[GroundedAtom],
    absent_identifiers: &[String],
) -> GroundingStatus {
    let is_contradictory = |c: &GroundedAtom| {
        sig.predicates
            .iter()
            .find(|p| p.name == c.predicate)
            .is_some_and(distinguishes_slots)
            && has_repeated_constant(&c.arguments)
    };

    if candidates.is_empty() {
        return if absent_identifiers.is_empty() {
            GroundingStatus::Ungroundable
        } else {
            GroundingStatus::Incomplete
        };
    }
    if candidates.iter().any(is_contradictory) {
        return GroundingStatus::Contradictory;
    }
    if candidates.iter().any(|c| c.confidence >= 0.8) {
        return GroundingStatus::Grounded;
    }
    // Short argument list against a declared predicate that needs more: the
    // cause is a partially stated requirement, not an ambiguous one.
    let short_of_declared = candidates.iter().any(|c| {
        sig.predicates
            .iter()
            .find(|p| p.name == c.predicate)
            .is_some_and(|p| p.arguments.len() >= 2 && c.arguments.len() < p.arguments.len())
    });
    if !absent_identifiers.is_empty() || short_of_declared {
        GroundingStatus::Incomplete
    } else {
        GroundingStatus::Ambiguous
    }
}

/// A predicate distinguishes its slots when it declares more than one slot and
/// the slot names are pairwise distinct — the signature author's own statement
/// that the slots mean different things.
fn distinguishes_slots(pred: &PredicateDef) -> bool {
    let names: Vec<&str> = pred.arguments.iter().map(|a| a.name.as_str()).collect();
    if names.len() < 2 {
        return false;
    }
    let mut seen = std::collections::HashSet::new();
    names.iter().all(|n| seen.insert(*n))
}

fn has_repeated_constant(args: &[BoundArgument]) -> bool {
    let mut seen = std::collections::HashSet::new();
    args.iter().any(|a| !seen.insert(a.constant.as_str()))
}

// ── Absent identifiers ──

/// Longest token we are willing to consider a constant-shaped identifier.
const MAX_IDENTIFIER_LEN: usize = 12;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CharClass {
    Alpha,
    Digit,
    Other,
}

fn class_of(c: char) -> CharClass {
    if c.is_ascii_digit() {
        CharClass::Digit
    } else if c.is_alphabetic() {
        CharClass::Alpha
    } else {
        CharClass::Other
    }
}

/// Split a name into maximal runs of the same character class.
/// `T1.1` → [Alpha, Digit, Other, Digit].
fn name_shape(name: &str) -> Vec<CharClass> {
    let mut shape = Vec::new();
    for c in name.chars() {
        let class = class_of(c);
        if shape.last() != Some(&class) {
            shape.push(class);
        }
    }
    shape
}

/// Find tokens in the text that have the shape of a constant of a declared
/// type but are not provided by any constant of that type.
///
/// The shape is inferred from the signature itself: each constant type
/// contributes its longest constant name, and its run structure is the pattern
/// to match. This keeps the grounder domain-agnostic — no task-ID syntax is
/// hard-coded anywhere.
fn extract_absent_identifiers(nl: &str, sig: &Signature) -> Vec<String> {
    // Longest constant name per type; its shape is the type's identifier pattern.
    let mut patterns: Vec<(Vec<CharClass>, usize)> = Vec::new();
    for type_def in &sig.types {
        let Some(longest) = sig
            .constants
            .iter()
            .filter(|c| c.type_name == type_def.name)
            .map(|c| c.name.as_str())
            .max_by_key(|n| n.chars().count())
        else {
            continue;
        };
        let shape = name_shape(longest);
        // A shape with a single run (e.g. all-alphabetic names) carries no
        // structure to match on and would match ordinary words.
        if shape.len() < 2 {
            continue;
        }
        let max_len = longest.chars().count().min(MAX_IDENTIFIER_LEN);
        patterns.push((shape, max_len));
    }
    if patterns.is_empty() {
        return Vec::new();
    }

    let known: std::collections::HashSet<&str> =
        sig.constants.iter().map(|c| c.name.as_str()).collect();
    let chars: Vec<char> = nl.chars().collect();
    let mut absent: Vec<String> = Vec::new();

    // For each start position, try every length and shape; record the longest
    // match so that trailing punctuation is not appended to the identifier.
    let mut i = 0;
    while i < chars.len() {
        let mut best: Option<usize> = None;
        let limit = (chars.len() - i).min(MAX_IDENTIFIER_LEN);
        for len in 2..=limit {
            let candidate: String = chars[i..i + len].iter().collect();
            let shape = name_shape(&candidate);
            if fits_any(&shape, len, &patterns) {
                best = Some(len);
            }
        }
        match best {
            Some(len) => {
                let candidate: String = chars[i..i + len].iter().collect();
                if !known.contains(candidate.as_str()) && !absent.contains(&candidate) {
                    absent.push(candidate);
                }
                i += len;
            }
            None => i += 1,
        }
    }
    absent
}

/// Does a candidate's shape match one of the type patterns?
/// Identifiers start with a letter; this keeps us off numbers in prose.
fn fits_any(shape: &[CharClass], len: usize, patterns: &[(Vec<CharClass>, usize)]) -> bool {
    shape.first() == Some(&CharClass::Alpha)
        && patterns
            .iter()
            .any(|(pattern, max_len)| len <= *max_len && shape == pattern.as_slice())
}

/// Check if a constant appears in the NL text (by name or alias).
fn appears_in(c: &ConstantDef, nl: &str, lower: &str) -> bool {
    // Exact name match (e.g., "T2.1")
    if nl.contains(&c.name) {
        return true;
    }
    // Alias match (e.g., "migration" from task description)
    c.aliases.iter().any(|a| lower.contains(a.as_str()))
}

/// Assign arguments to slots based on predicate-specific heuristics.
fn assign_arguments(
    pred_name: &str,
    matched: &[&ConstantDef],
    nl: &str,
    lower: &str,
    allow_alias: bool,
) -> Vec<BoundArgument> {
    let find = |c: &ConstantDef, slice: &str| find_match(c, slice, allow_alias);
    let take = |n: usize| -> Vec<BoundArgument> {
        matched
            .iter()
            .take(n)
            .map(|c| BoundArgument::by_name(&c.name))
            .collect()
    };
    match pred_name {
        "BEFORE" => {
            // "X BEFORE Y" → earlier=X, later=Y
            // Find the split point around "before"
            if let Some(pos) = lower.find("before") {
                let before_text = &nl[..pos];
                let after_text = &nl[pos + 6..];
                let before_match = matched.iter().find_map(|c| find(c, before_text));
                let after_match = matched.iter().find_map(|c| find(c, after_text));
                let mut args = Vec::new();
                if let Some(b) = before_match {
                    args.push(b);
                }
                if let Some(a) = after_match {
                    args.push(a);
                }
                args
            } else {
                take(2)
            }
        }
        "AFTER" => {
            // "X AFTER Y" → earlier=Y, later=X
            if let Some(pos) = lower.find("after") {
                let before_text = &nl[..pos];
                let after_text = &nl[pos + 5..];
                let before_match = matched.iter().find_map(|c| find(c, before_text));
                let after_match = matched.iter().find_map(|c| find(c, after_text));
                let mut args = Vec::new();
                // AFTER: the thing before "after" is the later task
                if let Some(a) = after_match {
                    args.push(a); // earlier
                }
                if let Some(b) = before_match {
                    args.push(b); // later
                }
                args
            } else {
                take(2)
            }
        }
        "CONCURRENTLY" | "AT_MOST_ONE" => take(2),
        "IF_THEN" => {
            // "IF X fails THEN Y" → trigger=X, consequent=Y
            if let Some(if_pos) = lower.find("if") {
                let after_if = &lower[if_pos + 2..];
                let then_pos = after_if.find("then").unwrap_or(usize::MAX);
                let trigger_text = &nl[if_pos + 2..if_pos + 2 + then_pos.min(after_if.len())];
                let consequent_text = if then_pos < after_if.len() {
                    &nl[if_pos + 2 + then_pos + 4..]
                } else {
                    ""
                };
                let trigger = matched.iter().find_map(|c| find(c, trigger_text));
                let consequent = matched.iter().find_map(|c| find(c, consequent_text));
                let mut args = Vec::new();
                if let Some(t) = trigger {
                    args.push(t);
                }
                if let Some(c) = consequent {
                    args.push(c);
                }
                args
            } else {
                take(2)
            }
        }
        "ALWAYS" => take(1),
        _ => matched
            .iter()
            .map(|c| BoundArgument::by_name(&c.name))
            .collect(),
    }
}

/// Find how a constant occurs in a slice: by name when possible, else by alias.
/// Alias matching is skipped when `allow_alias` is false.
fn find_match(c: &ConstantDef, slice: &str, allow_alias: bool) -> Option<BoundArgument> {
    if slice.contains(&c.name) {
        return Some(BoundArgument::by_name(&c.name));
    }
    if !allow_alias {
        return None;
    }
    let lower = slice.to_lowercase();
    c.aliases
        .iter()
        .find(|a| lower.contains(a.as_str()))
        .map(|a| BoundArgument::by_alias(&c.name, a))
}

#[cfg(test)]
mod diagnostics_tests;
#[cfg(test)]
mod tests;
