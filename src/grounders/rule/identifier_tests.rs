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
