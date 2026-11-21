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
    MatchKind, PredicateDef, Signature,
};

use identifiers::{appears_in, contains_name, extract_absent_identifiers};

mod identifiers;

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
            // receive the same constant twice. Only a *name* match in both slots
            // is a contradiction — the author wrote one task where two are
            // required. An alias that wins both slots is a weak binding, which
            // is our fault and is reported as low confidence instead of being
            // blamed on the spec.
            // Either kind of repeat is a weak binding, so confidence drops; only
            // a name repeat is classified Contradictory (see `classify`).
            let repeats = has_repeated_constant(&args);

            // 6. Structural confidence scoring
            let exact = !args.is_empty() && args.iter().all(BoundArgument::is_exact);
            let confidence = if args.is_empty() {
                0.0
            } else if repeats {
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
                let diagnostic = diagnostic_for(pred_name, pred_def, &args, &absent_identifiers);
                candidates.push(GroundedAtom {
                    predicate: pred_name.to_string(),
                    arguments: args,
                    exact,
                    confidence,
                    diagnostic,
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
        let unused_identifiers = unused_identifiers(sig, nl, &candidates);

        GroundingResult {
            candidates,
            status,
            absent_identifiers,
            unused_identifiers,
        }
    }
}

/// Render a one-line explanation of why this confidence was assigned.
///
/// `GroundedAtom` is the published v0.1.0 shape, so the field stays for source
/// compatibility with existing consumers. It is a *rendering* of the structured
/// fields (`exact`, the argument provenance, the slot names) — never the source
/// of truth, and never re-parsed.
fn diagnostic_for(
    pred_name: &str,
    pred_def: Option<&PredicateDef>,
    args: &[BoundArgument],
    absent_identifiers: &[String],
) -> String {
    if args.is_empty() {
        return format!("predicate {} matched but no argument was found", pred_name);
    }
    if has_repeated_constant(args) {
        let Some((first, second)) = name_repeated_pair(args) else {
            // Two slots bound the same constant, but at least one by alias:
            // the binding is too weak to fill the slots, so say so honestly
            // instead of blaming the statement.
            return format!(
                "one constant ('{}') was bound to both slots, at least one by alias — \
                 name the tasks explicitly",
                args[0].constant,
            );
        };
        let slot = |i: usize| {
            pred_def
                .and_then(|p| p.arguments.get(i))
                .map(|a| a.name.as_str())
                .unwrap_or("?")
        };
        return format!(
            "slots '{}' and '{}' require different tasks but '{}' fills both",
            slot(first),
            slot(second),
            args[first].constant,
        );
    }
    let expected = pred_def.map(|p| p.arguments.len()).unwrap_or(0);
    let short = expected > 0 && args.len() < expected;
    if short {
        let slots = pred_def
            .map(|p| {
                p.arguments
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        let mut msg = format!(
            "expected {} argument(s) ({}) but found {}",
            expected,
            slots,
            args.len()
        );
        if !absent_identifiers.is_empty() {
            msg.push_str(&format!(
                "; {} is not declared by this signature",
                absent_identifiers.join(", ")
            ));
        }
        return msg;
    }
    if args.iter().all(BoundArgument::is_exact) {
        return format!("all {} argument(s) matched by explicit name", args.len());
    }
    let aliases: Vec<String> = args
        .iter()
        .filter_map(|a| match &a.matched {
            MatchKind::Name => None,
            MatchKind::Alias(alias) => Some(format!("{} as '{}'", a.constant, alias)),
        })
        .collect();
    format!("matched by alias: {}", aliases.join(", "))
}

/// Constants the text names explicitly that the grounding left unbound.
///
/// The mirror of `absent_identifiers`, and the signal that exposes a slot filled
/// from something weaker while a task the author actually wrote went unused.
/// Empty when no candidate was produced: there the missing predicate is the
/// diagnosis, and every named constant would be "unused" for the wrong reason.
fn unused_identifiers(sig: &Signature, nl: &str, candidates: &[GroundedAtom]) -> Vec<String> {
    if candidates.is_empty() {
        return Vec::new();
    }
    let bound: std::collections::HashSet<&str> = candidates
        .iter()
        .flat_map(|c| c.arguments.iter().map(|a| a.constant.as_str()))
        .collect();
    sig.constants
        .iter()
        .filter(|c| contains_name(nl, &c.name) && !bound.contains(c.name.as_str()))
        .map(|c| c.name.clone())
        .collect()
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
            && name_repeated_pair(&c.arguments).is_some()
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

/// The first pair of slots that hold the same constant *and both matched by
/// name* — i.e. the author explicitly named one task twice.
fn name_repeated_pair(args: &[BoundArgument]) -> Option<(usize, usize)> {
    args.iter().enumerate().find_map(|(i, a)| {
        if !a.is_exact() {
            return None;
        }
        args.iter()
            .enumerate()
            .skip(i + 1)
            .find(|(_, b)| b.constant == a.constant && b.is_exact())
            .map(|(j, _)| (i, j))
    })
}

fn assign_arguments(
    pred_name: &str,
    matched: &[&ConstantDef],
    nl: &str,
    lower: &str,
    allow_alias: bool,
) -> Vec<BoundArgument> {
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
                let before_match = bind_slot(matched, before_text, allow_alias);
                let after_match = bind_slot(matched, after_text, allow_alias);
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
                let before_match = bind_slot(matched, before_text, allow_alias);
                let after_match = bind_slot(matched, after_text, allow_alias);
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
                let trigger = bind_slot(matched, trigger_text, allow_alias);
                let consequent = bind_slot(matched, consequent_text, allow_alias);
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

/// Bind one slot to a constant, preferring an explicit name over an alias.
///
/// Two passes: a constant the text names literally always wins, so an
/// alias-only constant can never displace the one the author wrote. Alias
/// matching is skipped entirely when `allow_alias` is false.
fn bind_slot(matched: &[&ConstantDef], slice: &str, allow_alias: bool) -> Option<BoundArgument> {
    if let Some(c) = matched
        .iter()
        .find(|c| contains_name(slice, &c.name))
        .copied()
    {
        return Some(BoundArgument::by_name(&c.name));
    }
    if !allow_alias {
        return None;
    }
    let lower = slice.to_lowercase();
    matched.iter().find_map(|c| {
        c.aliases
            .iter()
            .find(|a| contains_name(&lower, &a.to_lowercase()))
            .map(|a| BoundArgument::by_alias(&c.name, a))
    })
}

#[cfg(test)]
mod diagnostics_tests;
#[cfg(test)]
mod identifier_tests;
#[cfg(test)]
mod tests;
