//! Tests for directive generation, close matches, and string similarity.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::string_slice,
    clippy::panic
)]
use super::*;
use crate::Statement;
use crate::types::{
    ArgSlot, BoundArgument, ConstantDef, GroundedAtom, GroundingResult, GroundingStatus,
    PredicateDef, Signature, TypeDef,
};

#[test]
fn test_find_close_matches() {
    let sig = Signature {
        types: vec![TypeDef {
            name: "task_id".into(),
        }],
        predicates: vec![],
        constants: vec![
            ConstantDef {
                name: "T2.1".into(),
                type_name: "task_id".into(),
                aliases: vec!["migration".into(), "migrate".into()],
            },
            ConstantDef {
                name: "T3.1".into(),
                type_name: "task_id".into(),
                aliases: vec!["deployment".into(), "deploy".into()],
            },
        ],
    };
    let matches = find_close_matches("The migration step", &sig);
    assert!(!matches.is_empty());
    assert!(matches.iter().any(|m| m.constant == "T2.1"));
}

#[test]
fn test_find_close_matches_no_match() {
    let sig = Signature {
        types: vec![TypeDef {
            name: "task_id".into(),
        }],
        predicates: vec![],
        constants: vec![ConstantDef {
            name: "T1.1".into(),
            type_name: "task_id".into(),
            aliases: vec!["setup".into()],
        }],
    };
    let matches = find_close_matches("xyzzy", &sig);
    assert!(matches.is_empty());
}

#[test]
fn test_find_close_matches_truncated() {
    let mut constants = Vec::new();
    for i in 0..20 {
        constants.push(ConstantDef {
            name: format!("T{}.1", i),
            type_name: "task_id".into(),
            aliases: vec![format!("task{}", i)],
        });
    }
    let sig = Signature {
        types: vec![TypeDef {
            name: "task_id".into(),
        }],
        predicates: vec![],
        constants,
    };
    let matches = find_close_matches("task5", &sig);
    assert!(matches.len() <= 5);
    assert!(matches.iter().any(|m| m.constant == "T5.1"));
}

#[test]

fn test_generate_directives_grounded() {
    let result = GroundingResult {
        candidates: vec![GroundedAtom {
            predicate: "BEFORE".into(),
            arguments: vec![
                BoundArgument::by_name("T1.1"),
                BoundArgument::by_name("T1.2"),
            ],
            exact: true,
            confidence: 0.95,
            diagnostic: String::new(),
        }],
        status: GroundingStatus::Grounded,
        absent_identifiers: vec![],
        unused_identifiers: vec![],
    };
    let stmt = Statement {
        source: "test.md#L1".into(),
        text: "T1.1 BEFORE T1.2".into(),
    };
    let sig = Signature {
        types: vec![],
        predicates: vec![],
        constants: vec![],
    };
    let dirs = generate_directives(&result, &stmt, &sig);
    assert!(dirs.is_empty());
}

#[test]
fn test_generate_directives_ungroundable() {
    let result = GroundingResult {
        candidates: vec![],
        status: GroundingStatus::Ungroundable,
        absent_identifiers: vec![],
        unused_identifiers: vec![],
    };
    let stmt = Statement {
        source: "test.md#L1".into(),
        text: "The system SHALL be robust".into(),
    };
    let sig = Signature {
        types: vec![],
        predicates: vec![],
        constants: vec![],
    };
    let dirs = generate_directives(&result, &stmt, &sig);
    assert_eq!(dirs.len(), 1);
    assert_eq!(dirs[0].severity, "blocker");
}

#[test]
fn test_generate_directives_ambiguous() {
    let result = GroundingResult {
        candidates: vec![GroundedAtom {
            predicate: "BEFORE".into(),
            arguments: vec![
                BoundArgument::by_alias("T2.1", "migration"),
                BoundArgument::by_alias("T2.2", "testing"),
            ],
            exact: false,
            confidence: 0.9,
            diagnostic: String::new(),
        }],
        status: GroundingStatus::Ambiguous,
        absent_identifiers: vec![],
        unused_identifiers: vec![],
    };
    let stmt = Statement {
        source: "test.md#L1".into(),
        text: "migration before testing".into(),
    };
    let sig = Signature {
        types: vec![TypeDef {
            name: "task_id".into(),
        }],
        predicates: vec![],
        constants: vec![
            ConstantDef {
                name: "T2.1".into(),
                type_name: "task_id".into(),
                aliases: vec!["migration".into()],
            },
            ConstantDef {
                name: "T2.2".into(),
                type_name: "task_id".into(),
                aliases: vec!["testing".into()],
            },
        ],
    };
    let dirs = generate_directives(&result, &stmt, &sig);
    assert!(!dirs.is_empty());
    assert_eq!(dirs[0].severity, "warning");
}

#[test]
fn test_find_close_matches_by_constant_name() {
    let sig = Signature {
        types: vec![TypeDef {
            name: "task_id".into(),
        }],
        predicates: vec![],
        constants: vec![ConstantDef {
            name: "T1.3".into(),
            type_name: "task_id".into(),
            aliases: vec!["implement appconfig".into()],
        }],
    };
    // The text "T1.3" should match the constant name directly
    let matches = find_close_matches("T1.3", &sig);
    assert!(!matches.is_empty());
    assert!(matches.iter().any(|m| m.constant == "T1.3"));
}

#[test]
fn test_generate_directives_before_one_arg() {
    let result = GroundingResult {
        candidates: vec![GroundedAtom {
            predicate: "BEFORE".into(),
            arguments: vec![BoundArgument::by_name("T3.1")],
            exact: true,
            confidence: 0.5,
            diagnostic: String::new(),
        }],
        status: GroundingStatus::Incomplete,
        absent_identifiers: vec![],
        unused_identifiers: vec![],
    };
    let stmt = Statement {
        source: "test.md#L1".into(),
        text: "T3.1 SHALL complete before any pool is used".into(),
    };
    let sig = Signature {
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
        constants: vec![ConstantDef {
            name: "T3.1".into(),
            type_name: "task_id".into(),
            aliases: vec!["setup".into()],
        }],
    };
    let dirs = generate_directives(&result, &stmt, &sig);
    assert!(!dirs.is_empty());
    assert!(
        dirs.iter()
            .any(|d| d.detail.contains("BEFORE takes 2 tasks"))
    );
}

// ── Diagnostic directives ──

// ── Diagnostic directives ──

fn diagnostics_sig() -> Signature {
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
        constants: vec![ConstantDef {
            name: "T1.1".into(),
            type_name: "task_id".into(),
            aliases: vec!["publish".into(), "header".into()],
        }],
    }
}

#[test]
fn test_absent_identifier_directive_names_the_identifier() {
    let result = GroundingResult {
        candidates: vec![],
        status: GroundingStatus::Incomplete,
        absent_identifiers: vec!["T2.6".into()],
        unused_identifiers: vec![],
    };
    let stmt = Statement {
        source: "spec.md#L9".into(),
        text: "T2.6 SHALL publish the port BEFORE T1.1 SHALL rename".into(),
    };
    let dirs = generate_directives(&result, &stmt, &diagnostics_sig());
    let detail = dirs
        .iter()
        .map(|d| d.detail.clone())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(detail.contains("T2.6"));
    assert!(detail.contains("is not a task of this signature"));
}

#[test]
fn test_absent_identifier_suppresses_add_task_ids_advice() {
    let result = GroundingResult {
        candidates: vec![GroundedAtom {
            predicate: "BEFORE".into(),
            arguments: vec![BoundArgument::by_name("T1.1")],
            exact: true,
            confidence: 0.5,
            diagnostic: String::new(),
        }],
        status: GroundingStatus::Incomplete,
        absent_identifiers: vec!["T2.6".into()],
        unused_identifiers: vec![],
    };
    let stmt = Statement {
        source: "spec.md#L9".into(),
        text: "T2.6 SHALL publish the port BEFORE T1.1 SHALL rename".into(),
    };
    let dirs = generate_directives(&result, &stmt, &diagnostics_sig());
    let actions = dirs
        .iter()
        .map(|d| d.suggested_action.clone())
        .collect::<Vec<_>>()
        .join(" ");
    assert!(actions.contains("T2.6"));
    assert!(!actions.contains("Add explicit task ID references"));
}

#[test]
fn test_contradiction_directive_names_slots() {
    let result = GroundingResult {
        candidates: vec![GroundedAtom {
            predicate: "BEFORE".into(),
            arguments: vec![
                BoundArgument::by_name("T1.1"),
                BoundArgument::by_name("T1.1"),
            ],
            exact: true,
            confidence: 0.5,
            diagnostic: String::new(),
        }],
        status: GroundingStatus::Contradictory,
        absent_identifiers: vec![],
        unused_identifiers: vec![],
    };
    let stmt = Statement {
        source: "spec.md#L4".into(),
        text: "T1.1 SHALL write the header BEFORE T1.1 SHALL seal it".into(),
    };
    let dirs = generate_directives(&result, &stmt, &diagnostics_sig());
    let detail = dirs
        .iter()
        .map(|d| d.detail.clone())
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(dirs[0].severity, "blocker");
    assert!(detail.contains("T1.1"));
    assert!(detail.contains("earlier"));
    assert!(detail.contains("later"));
}

#[test]
fn test_alias_directive_comes_from_provenance() {
    let result = GroundingResult {
        candidates: vec![GroundedAtom {
            predicate: "BEFORE".into(),
            arguments: vec![
                BoundArgument::by_alias("T1.1", "header"),
                BoundArgument::by_alias("T2.1", "seal"),
            ],
            exact: false,
            confidence: 0.9,
            diagnostic: String::new(),
        }],
        status: GroundingStatus::Grounded,
        absent_identifiers: vec![],
        unused_identifiers: vec![],
    };
    let stmt = Statement {
        source: "spec.md#L4".into(),
        text: "the header SHALL be written before the seal".into(),
    };
    let dirs = alias_directives(&result.candidates[0], &stmt);
    assert_eq!(dirs.len(), 2);
    assert!(dirs[0].detail.contains("'header'"));
    assert!(dirs[0].detail.contains("T1.1"));
}

#[test]
fn test_name_match_emits_no_alias_directive() {
    let stmt = Statement {
        source: "spec.md#L4".into(),
        text: "T1.1 BEFORE T2.1".into(),
    };
    let atom = GroundedAtom {
        predicate: "BEFORE".into(),
        arguments: vec![
            BoundArgument::by_name("T1.1"),
            BoundArgument::by_name("T2.1"),
        ],
        exact: true,
        confidence: 0.95,
        diagnostic: String::new(),
    };
    assert!(alias_directives(&atom, &stmt).is_empty());
}
#[test]
fn test_strsim_identical() {
    let s = strsim("hello", "hello");
    assert!((s - 1.0).abs() < 0.001);
}

#[test]
fn test_strsim_similar() {
    let s = strsim("migration", "migrate");
    assert!(s > 0.4);
}

#[test]
fn test_strsim_different() {
    let s = strsim("abc", "xyz");
    assert!(s < 0.1);
}

#[test]
fn test_strsim_empty() {
    assert_eq!(strsim("", "hello"), 0.0);
    assert_eq!(strsim("hello", ""), 0.0);
    assert_eq!(strsim("", ""), 0.0);
}

#[test]
fn test_strsim_short() {
    let s = strsim("a", "b");
    assert_eq!(s, 0.0);
}
