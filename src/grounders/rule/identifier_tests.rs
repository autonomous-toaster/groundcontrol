//! Identifier tokenisation: multi-digit ids, byte-identity, boundary coverage.
//! Regression tests for kata 08p0 (B1) and cdzv (B2).

use super::*;
use crate::types::{ArgSlot, ConstantDef, GroundingStatus, PredicateDef, Signature, TypeDef};

// ── B1: multi-digit identifiers must not be truncated (issue 08p0) ──

fn before_sig(names: &[&str]) -> Signature {
    Signature {
        types: vec![TypeDef {
            name: "task_id".into(),
        }],
        predicates: vec![PredicateDef {
            name: "BEFORE".into(),
            arguments: vec![
                ArgSlot {
                    name: "earlier".into(),
                    type_name: "task_id".into(),
                },
                ArgSlot {
                    name: "later".into(),
                    type_name: "task_id".into(),
                },
            ],
        }],
        constants: names
            .iter()
            .map(|n| ConstantDef {
                name: (*n).to_string(),
                type_name: "task_id".into(),
                aliases: vec![],
            })
            .collect(),
    }
}

#[test]
fn test_absent_identifier_keeps_multi_digit_after_dot() {
    // Repro A: a signature whose longest id is 4 chars must still recognise
    // the 5-char T2.10 instead of truncating it to T2.1.
    let sig = before_sig(&["T1.1", "T1.2"]);
    let result = RuleGrounder.ground("T2.10 SHALL complete BEFORE T1.1 SHALL start.", &sig);
    assert_eq!(result.absent_identifiers, vec!["T2.10".to_string()]);
}

#[test]
fn test_absent_identifier_reports_foreign_multi_digit_reference() {
    // Repro B: T1.10 is genuinely foreign. It must be named, not swallowed,
    // and the caller must not fall back to an arity message.
    let sig = before_sig(&["T1.1"]);
    let result = RuleGrounder.ground("T1.10 SHALL complete BEFORE T1.1 SHALL start.", &sig);
    assert_eq!(result.absent_identifiers, vec!["T1.10".to_string()]);
    assert_eq!(result.status, GroundingStatus::Incomplete);
}

#[test]
fn test_absent_identifier_multi_digit_on_both_sides() {
    let sig = before_sig(&["T1.1"]);
    let result = RuleGrounder.ground("T10.123 SHALL run BEFORE T1.1 SHALL stop", &sig);
    assert_eq!(result.absent_identifiers, vec!["T10.123".to_string()]);
}

#[test]
fn test_reported_identifier_is_byte_identical_to_the_input() {
    let sig = before_sig(&["T1.1"]);
    let text = "T2.10 SHALL complete BEFORE T1.1 SHALL start.";
    let result = RuleGrounder.ground(text, &sig);
    for id in &result.absent_identifiers {
        assert!(
            text.contains(id.as_str()),
            "reported identifier {id:?} does not occur verbatim in the input",
        );
    }
}

#[test]
fn test_trailing_sentence_dot_is_not_part_of_an_identifier() {
    let sig = before_sig(&["T1.1"]);
    let result = RuleGrounder.ground("T2.10 SHALL run BEFORE T1.1 SHALL stop.", &sig);
    assert_eq!(result.absent_identifiers, vec!["T2.10".to_string()]);
}

// ── B2: prove T1.1 and T1.10 are never confused (issue cdzv) ──

#[test]
fn test_t1_1_is_not_bound_when_the_text_cites_only_t1_10() {
    let sig = before_sig(&["T1.1", "T1.2", "T1.10"]);
    let result = RuleGrounder.ground("T1.10 SHALL complete BEFORE T1.2 SHALL start.", &sig);
    assert_eq!(result.candidates[0].names(), vec!["T1.10", "T1.2"]);
    assert!(
        !result.candidates[0].names().contains(&"T1.1"),
        "T1.1 must not be bound by a text that only cites T1.10",
    );
    assert!(result.absent_identifiers.is_empty());
}

#[test]
fn test_t1_1_and_t1_10_are_never_confused_in_either_direction() {
    let sig = before_sig(&["T1.1", "T1.2", "T1.10"]);
    // Citing T1.1 must not bind T1.10 and vice versa.
    let a = RuleGrounder.ground("T1.1 SHALL complete BEFORE T1.10 SHALL start", &sig);
    assert_eq!(a.candidates[0].names(), vec!["T1.1", "T1.10"]);
    let b = RuleGrounder.ground("T1.10 SHALL complete BEFORE T1.1 SHALL start", &sig);
    assert_eq!(b.candidates[0].names(), vec!["T1.10", "T1.1"]);
    // A single-slot predicate leaves one of the two named tasks unbound. Which
    // one is a separate question (see issue cdzv note: `take(n)` picks in
    // signature order, not text order); what matters here is that the reported
    // string is one of the tokens actually written, never a truncation.
    let text = "T1.10 SHALL ALWAYS mention T1.1";
    let c = RuleGrounder.ground(text, &sig);
    assert_eq!(c.unused_identifiers.len(), 1);
    assert!(
        text.contains(c.unused_identifiers[0].as_str()),
        "unused identifier {:?} is not verbatim in the input",
        c.unused_identifiers[0],
    );
}

// ── B3: single-slot predicates bind the first mention in the text ──

fn always_sig(names: &[&str]) -> Signature {
    Signature {
        types: vec![TypeDef {
            name: "task_id".into(),
        }],
        predicates: vec![PredicateDef {
            name: "ALWAYS".into(),
            arguments: vec![ArgSlot {
                name: "target".into(),
                type_name: "task_id".into(),
            }],
        }],
        constants: names
            .iter()
            .map(|n| ConstantDef {
                name: (*n).to_string(),
                type_name: "task_id".into(),
                aliases: vec![],
            })
            .collect(),
    }
}

#[test]
fn test_single_slot_binds_the_subject_not_the_signature_first() {
    let sig = always_sig(&["T1.1", "T1.10"]);
    let result = RuleGrounder.ground("T1.10 SHALL ALWAYS mention T1.1", &sig);
    assert_eq!(result.candidates[0].names(), vec!["T1.10"]);
    assert_eq!(result.unused_identifiers, vec!["T1.1".to_string()]);
    assert!(result.candidates[0].exact);
}

#[test]
fn test_single_slot_follows_the_text_in_the_other_ordering() {
    // Same signature, same pair, reversed sentence: the binding must follow
    // the text, and the unbound one must now be the later mention.
    let sig = always_sig(&["T1.1", "T1.10"]);
    let result = RuleGrounder.ground("T1.1 SHALL ALWAYS mention T1.10", &sig);
    assert_eq!(result.candidates[0].names(), vec!["T1.1"]);
    assert_eq!(result.unused_identifiers, vec!["T1.10".to_string()]);
}

#[test]
fn test_single_slot_unchanged_when_only_one_task_is_named() {
    let sig = always_sig(&["T1.1", "T1.2"]);
    let result = RuleGrounder.ground("T1.2 SHALL ALWAYS be available", &sig);
    assert_eq!(result.candidates[0].names(), vec!["T1.2"]);
    assert!(result.unused_identifiers.is_empty());
    assert_eq!(result.status, GroundingStatus::Grounded);
}

#[test]
fn test_single_slot_binding_by_alias_is_reported_as_an_alias() {
    let mut sig = always_sig(&["T1.1", "T2.2"]);
    sig.constants[1].aliases = vec!["rollback".into()];
    let result = RuleGrounder.ground("rollback SHALL ALWAYS be available", &sig);
    assert_eq!(result.candidates[0].names(), vec!["T2.2"]);
    assert!(!result.candidates[0].exact);
    assert_eq!(result.candidates[0].confidence, 0.9);
    assert_eq!(result.status, GroundingStatus::Grounded);
}

// ── 9kvc: non-ASCII text must never panic the scanner ──

fn id_sig(names: &[&str]) -> Signature {
    Signature {
        types: vec![TypeDef {
            name: "task_id".into(),
        }],
        predicates: vec![PredicateDef {
            name: "BEFORE".into(),
            arguments: vec![
                ArgSlot {
                    name: "earlier".into(),
                    type_name: "task_id".into(),
                },
                ArgSlot {
                    name: "later".into(),
                    type_name: "task_id".into(),
                },
            ],
        }],
        constants: names
            .iter()
            .map(|n| ConstantDef {
                name: (*n).to_string(),
                type_name: "task_id".into(),
                aliases: vec![],
            })
            .collect(),
    }
}

#[test]
fn test_arrow_after_a_task_id_does_not_panic() {
    // The reported repro: a multi-byte character the scanner must step over.
    let sig = id_sig(&["T1.1"]);
    let result = RuleGrounder.ground(
        "T1.1 SHALL forward the upstream status during H2\u{2192}H1.1 transcoding.",
        &sig,
    );
    assert!(result.absent_identifiers.contains(&"H1.1".to_string()));
}

#[test]
fn test_em_dash_between_tasks_does_not_panic() {
    let sig = id_sig(&["T1.1", "T2.2"]);
    let result = RuleGrounder.ground("T1.1 SHALL complete \u{2014} BEFORE T2.2 SHALL start", &sig);
    assert_eq!(result.candidates[0].names(), vec!["T1.1", "T2.2"]);
}

#[test]
fn test_accented_text_around_identifiers_does_not_panic() {
    let sig = id_sig(&["T1.1", "T2.2"]);
    let result = RuleGrounder.ground(
        "T1.1 SHALL renommer la t\u{e2}che caf\u{e9} BEFORE T2.2 SHALL start",
        &sig,
    );
    assert_eq!(result.candidates[0].names(), vec!["T1.1", "T2.2"]);
}

#[test]
fn test_lowercase_expansion_does_not_panic() {
    // U+0130 lowercases to two chars, so an offset taken from a lowercased copy
    // overruns the original. The keyword sits at the very end to force it.
    let sig = id_sig(&["T1.1"]);
    for n in 1..10 {
        let text = format!("{} T1.1 SHALL complete before", "\u{130}".repeat(n));
        let _ = RuleGrounder.ground(&text, &sig);
    }
}

#[test]
fn test_no_panic_across_multibyte_padding_and_keyword_positions() {
    let sig = id_sig(&["T1.1"]);
    let pads = [
        "\u{2192}",
        "\u{2014}",
        "\u{e9}",
        "\u{4e00}",
        "\u{130}",
        "\u{1f600}",
    ];
    let shapes = [
        "{p}{p}{p} T1.1 SHALL complete before T2.2 SHALL start",
        "T1.1 SHALL complete before {p}{p}{p} T2.2 SHALL start",
        "{p}{p}{p} T1.1 SHALL complete before",
        "{p} T1.1 SHALL complete AFTER {p} T2.2 SHALL start",
        "{p}{p} IF T1.1 fails THEN {p} T2.2 SHALL start",
        "{p}{p}{p} T1.1 SHALL ALWAYS {p} hold",
    ];
    for pad in pads {
        for shape in shapes {
            let text = shape.replace("{p}", pad);
            let _ = RuleGrounder.ground(&text, &sig);
        }
    }
}

#[test]
fn test_multibyte_text_still_grounds_on_the_named_tasks() {
    // Correctness, not just absence of panic: the arrow must not hide a binding.
    let sig = id_sig(&["T1.1", "T2.2"]);
    let result = RuleGrounder.ground("T1.1 SHALL serve \u{2192} T2.2 SHALL start AFTER it", &sig);
    assert_eq!(result.candidates[0].predicate, "AFTER");
    assert!(result.candidates[0].names().contains(&"T1.1"));
}
