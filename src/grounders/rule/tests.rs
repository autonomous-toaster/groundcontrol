#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::string_slice,
    clippy::panic
)]
use super::*;
use crate::types::{ArgSlot, ConstantDef, MatchKind, PredicateDef, Signature, TypeDef};

pub(super) fn test_sig() -> Signature {
    Signature {
        types: vec![TypeDef {
            name: "task_id".into(),
        }],
        predicates: vec![
            PredicateDef {
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
            },
            PredicateDef {
                name: "AFTER".into(),
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
            },
            PredicateDef {
                name: "CONCURRENTLY".into(),
                arguments: vec![
                    ArgSlot {
                        name: "a".into(),
                        type_name: "task_id".into(),
                    },
                    ArgSlot {
                        name: "b".into(),
                        type_name: "task_id".into(),
                    },
                ],
            },
            PredicateDef {
                name: "IF_THEN".into(),
                arguments: vec![
                    ArgSlot {
                        name: "trigger".into(),
                        type_name: "task_id".into(),
                    },
                    ArgSlot {
                        name: "consequent".into(),
                        type_name: "task_id".into(),
                    },
                ],
            },
            PredicateDef {
                name: "ALWAYS".into(),
                arguments: vec![ArgSlot {
                    name: "target".into(),
                    type_name: "task_id".into(),
                }],
            },
            PredicateDef {
                name: "AT_MOST_ONE".into(),
                arguments: vec![
                    ArgSlot {
                        name: "a".into(),
                        type_name: "task_id".into(),
                    },
                    ArgSlot {
                        name: "b".into(),
                        type_name: "task_id".into(),
                    },
                ],
            },
        ],
        constants: vec![
            ConstantDef {
                name: "T1.1".into(),
                type_name: "task_id".into(),
                aliases: vec!["setup".into(), "create project".into(), "1.1".into()],
            },
            ConstantDef {
                name: "T1.2".into(),
                type_name: "task_id".into(),
                aliases: vec!["deps".into(), "add dependencies".into(), "1.2".into()],
            },
            ConstantDef {
                name: "T2.1".into(),
                type_name: "task_id".into(),
                aliases: vec!["migrate".into(), "migration".into(), "2.1".into()],
            },
            ConstantDef {
                name: "T2.2".into(),
                type_name: "task_id".into(),
                aliases: vec!["test".into(), "testing".into(), "2.2".into()],
            },
            ConstantDef {
                name: "T3.1".into(),
                type_name: "task_id".into(),
                aliases: vec!["deploy".into(), "deployment".into(), "3.1".into()],
            },
        ],
    }
}

#[test]
fn test_ground_before_explicit_ids() {
    let sig = test_sig();
    let grounder = RuleGrounder;
    let result = grounder.ground("T1.1 SHALL complete BEFORE T1.2 SHALL run", &sig);
    assert_eq!(result.status, GroundingStatus::Grounded);
    assert_eq!(result.candidates[0].predicate, "BEFORE");
    assert_eq!(result.candidates[0].names(), vec!["T1.1", "T1.2"]);
    assert!(result.candidates[0].confidence >= 0.8);
    assert!(result.candidates[0].exact);
}

#[test]
fn test_ground_before_with_aliases() {
    let sig = test_sig();
    let grounder = RuleGrounder;
    let result = grounder.ground("The migration must finish before testing starts", &sig);
    // Both args found via aliases (migration→T2.1, testing→T2.2) → Grounded
    assert_eq!(result.status, GroundingStatus::Grounded);
    assert_eq!(result.candidates[0].predicate, "BEFORE");
    assert!(result.candidates[0].names().contains(&"T2.1"));
    assert!(result.candidates[0].names().contains(&"T2.2"));
    // Both matched by alias: the provenance records it, and the atom is not exact.
    assert!(!result.candidates[0].exact);
    assert!(
        result.candidates[0]
            .arguments
            .iter()
            .all(|a| matches!(a.matched, MatchKind::Alias(_)))
    );
}

#[test]
fn test_ground_after_explicit_ids() {
    let sig = test_sig();
    let grounder = RuleGrounder;
    let result = grounder.ground("T2.2 SHALL run AFTER T2.1 SHALL complete", &sig);
    assert_eq!(result.status, GroundingStatus::Grounded);
    assert_eq!(result.candidates[0].predicate, "AFTER");
    // AFTER: earlier is the task after "after", later is the task before "after"
    assert_eq!(result.candidates[0].names(), vec!["T2.1", "T2.2"]);
}

#[test]
fn test_ground_concurrently() {
    let sig = test_sig();
    let grounder = RuleGrounder;
    let result = grounder.ground("T2.1 and T2.2 SHALL run concurrently", &sig);
    assert_eq!(result.status, GroundingStatus::Grounded);
    assert_eq!(result.candidates[0].predicate, "CONCURRENTLY");
    assert!(result.candidates[0].names().contains(&"T2.1"));
    assert!(result.candidates[0].names().contains(&"T2.2"));
}

#[test]
fn test_ground_if_then() {
    let sig = test_sig();
    let grounder = RuleGrounder;
    let result = grounder.ground("IF T1.1 fails THEN T3.1 SHALL run", &sig);
    assert_eq!(result.status, GroundingStatus::Grounded);
    assert_eq!(result.candidates[0].predicate, "IF_THEN");
    assert_eq!(result.candidates[0].names(), vec!["T1.1", "T3.1"]);
}

#[test]
fn test_ground_always() {
    let sig = test_sig();
    let grounder = RuleGrounder;
    let result = grounder.ground("T3.1 SHALL ALWAYS be available", &sig);
    assert_eq!(result.status, GroundingStatus::Grounded);
    assert_eq!(result.candidates[0].predicate, "ALWAYS");
    assert_eq!(result.candidates[0].names(), vec!["T3.1"]);
}

#[test]
fn test_ground_at_most_one() {
    let sig = test_sig();
    let grounder = RuleGrounder;
    let result = grounder.ground("At most one of T2.1, T2.2 SHALL be active", &sig);
    assert_eq!(result.status, GroundingStatus::Grounded);
    assert_eq!(result.candidates[0].predicate, "AT_MOST_ONE");
    assert!(result.candidates[0].names().contains(&"T2.1"));
    assert!(result.candidates[0].names().contains(&"T2.2"));
}

#[test]
fn test_ground_ungroundable() {
    let sig = test_sig();
    let grounder = RuleGrounder;
    let result = grounder.ground("The system SHALL be user-friendly", &sig);
    assert_eq!(result.status, GroundingStatus::Ungroundable);
    assert!(result.candidates.is_empty());
}

#[test]
fn test_ground_no_keyword() {
    let sig = test_sig();
    let grounder = RuleGrounder;
    let result = grounder.ground("T1.1 does something", &sig);
    assert_eq!(result.status, GroundingStatus::Ungroundable);
}

#[test]
fn test_ground_batch() {
    let sig = test_sig();
    let grounder = RuleGrounder;
    let inputs = vec!["T1.1 BEFORE T1.2", "The system SHALL be user-friendly"];
    let results = grounder.ground_batch(&inputs, &sig);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].status, GroundingStatus::Grounded);
    assert_eq!(results[1].status, GroundingStatus::Ungroundable);
}

#[test]
fn test_ground_after_with_aliases() {
    let sig = test_sig();
    let grounder = RuleGrounder;
    let result = grounder.ground("Testing SHALL start after migration completes", &sig);
    // Both args found via aliases → Grounded
    assert_eq!(result.status, GroundingStatus::Grounded);
    assert_eq!(result.candidates[0].predicate, "AFTER");
}

#[test]
fn test_ground_if_then_with_aliases() {
    let sig = test_sig();
    let grounder = RuleGrounder;
    let result = grounder.ground("If setup fails then deployment SHALL run", &sig);
    // Both args found via aliases → Grounded
    assert_eq!(result.status, GroundingStatus::Grounded);
    assert_eq!(result.candidates[0].predicate, "IF_THEN");
}

#[test]
fn test_ground_multiple_predicates() {
    let sig = test_sig();
    let grounder = RuleGrounder;
    // Text contains both BEFORE and ALWAYS keywords
    let result = grounder.ground(
        "T1.1 SHALL complete BEFORE T1.2. T3.1 SHALL ALWAYS be available.",
        &sig,
    );
    // Should find both predicates
    assert!(result.candidates.iter().any(|c| c.predicate == "BEFORE"));
    assert!(result.candidates.iter().any(|c| c.predicate == "ALWAYS"));
}
