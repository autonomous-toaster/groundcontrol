//! Diagnostics tests: absent identifiers, slot distinctness, match provenance.

use super::tests::test_sig;
use super::*;
use crate::types::{ArgSlot, ConstantDef, MatchKind, PredicateDef, Signature, TypeDef};

// ── D1: absent identifiers ──

/// Signature declaring only T1.1 — mirrors a change that references a task
/// belonging to an archived change.
fn sig_without_t2_6() -> Signature {
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
            aliases: vec!["renames".into(), "noun".into(), "1.1".into()],
        }],
    }
}

#[test]
fn test_absent_identifier_is_named() {
    let sig = sig_without_t2_6();
    let result = RuleGrounder.ground(
        "T2.6 SHALL publish the port BEFORE T1.1 renames the noun.",
        &sig,
    );
    assert_eq!(result.absent_identifiers, vec!["T2.6".to_string()]);
}

#[test]
fn test_absent_identifier_does_not_become_ambiguous() {
    let sig = sig_without_t2_6();
    let result = RuleGrounder.ground(
        "T2.6 SHALL publish the port BEFORE T1.1 renames the noun.",
        &sig,
    );
    assert_ne!(result.status, GroundingStatus::Ambiguous);
    assert_eq!(result.status, GroundingStatus::Incomplete);
}

#[test]
fn test_absent_identifier_without_temporal_keyword() {
    let sig = sig_without_t2_6();
    let result = RuleGrounder.ground("T2.6 SHALL publish the port", &sig);
    assert_eq!(result.absent_identifiers, vec!["T2.6".to_string()]);
    assert_eq!(result.status, GroundingStatus::Incomplete);
}

#[test]
fn test_absent_identifier_ignores_declared_constants() {
    let sig = test_sig();
    let result = RuleGrounder.ground("T1.1 SHALL complete BEFORE T2.1 SHALL run", &sig);
    assert!(result.absent_identifiers.is_empty());
}

#[test]
fn test_absent_identifier_ignores_ordinary_words() {
    let sig = test_sig();
    // "SHALL" has the same character-class shape as a task ID prefix but is
    // longer than the longest constant name, so it must not be reported.
    let result = RuleGrounder.ground("T1.1 SHALL complete BEFORE T1.2 SHALL run", &sig);
    assert!(result.absent_identifiers.is_empty());
}

#[test]
fn test_absent_identifier_needs_an_inferable_shape() {
    // Constants whose names form a single character run carry no pattern.
    let sig = Signature {
        types: vec![TypeDef {
            name: "flag".into(),
        }],
        predicates: vec![],
        constants: vec![ConstantDef {
            name: "alpha".into(),
            type_name: "flag".into(),
            aliases: vec![],
        }],
    };
    let result = RuleGrounder.ground("T9.9 SHALL be set", &sig);
    assert!(result.absent_identifiers.is_empty());
}

#[test]
fn test_absent_identifier_suppresses_alias_guessing() {
    // "re s" (from "renames") is an alias of T1.1, but T2.6 is the actual
    // subject. Binding T1.1 twice would be a false contradiction.
    let sig = sig_without_t2_6();
    let result = RuleGrounder.ground(
        "T2.6 SHALL publish the port BEFORE T1.1 renames the noun.",
        &sig,
    );
    assert_eq!(result.status, GroundingStatus::Incomplete);
    assert_eq!(result.candidates[0].names(), vec!["T1.1"]);
}

#[test]
fn test_absent_identifier_does_not_hide_a_real_contradiction() {
    let sig = sig_without_t2_6();
    let mut sig = sig;
    sig.constants[0].aliases.clear();
    let result = RuleGrounder.ground(
        "T1.1 SHALL write the header BEFORE T1.1 SHALL seal it. T2.6 SHALL follow.",
        &sig,
    );
    assert_eq!(result.absent_identifiers, vec!["T2.6".to_string()]);
    assert_eq!(result.status, GroundingStatus::Contradictory);
}

// ── D2: repeated constants in distinct slots ──

#[test]
fn test_before_with_same_task_twice_is_not_grounded() {
    let sig = test_sig();
    let result = RuleGrounder.ground(
        "T1.1 SHALL write the header BEFORE T1.1 SHALL seal it",
        &sig,
    );
    assert_ne!(result.status, GroundingStatus::Grounded);
    assert_eq!(result.status, GroundingStatus::Contradictory);
}

#[test]
fn test_after_with_same_task_twice_is_contradictory() {
    let sig = test_sig();
    let result = RuleGrounder.ground("T1.1 SHALL run AFTER T1.1 SHALL complete", &sig);
    assert_eq!(result.status, GroundingStatus::Contradictory);
}

#[test]
fn test_concurrently_with_same_task_twice_is_contradictory() {
    let sig = test_sig();
    // Two distinct constants are present so the argument list is full; the
    // "a"/"b" slots still hold the same constant and must be rejected.
    let mut sig = sig;
    sig.constants[0].aliases.push("t1.1".into());
    let result = RuleGrounder.ground("T1.1 and T2.1 SHALL run concurrently", &sig);
    assert_eq!(result.status, GroundingStatus::Grounded);

    let result = RuleGrounder.ground("T1.1 and T1.1 SHALL run concurrently", &sig);
    assert_ne!(result.status, GroundingStatus::Grounded);
}

#[test]
fn test_at_most_one_with_same_task_twice_is_contradictory() {
    let sig = test_sig();
    let result = RuleGrounder.ground("At most one of T1.1, T2.1 SHALL be active", &sig);
    assert_eq!(result.status, GroundingStatus::Grounded);

    let mut both = test_sig();
    both.constants.push(ConstantDef {
        name: "T1.1bis".into(),
        type_name: "task_id".into(),
        aliases: vec![],
    });
    let result = both_ground(&mut both);
    assert_ne!(result.status, GroundingStatus::Grounded);
}

#[test]
fn test_if_then_with_same_task_twice_is_contradictory() {
    let sig = test_sig();
    let result = RuleGrounder.ground("IF T1.1 fails THEN T1.1 SHALL run", &sig);
    assert_eq!(result.status, GroundingStatus::Contradictory);
}

#[test]
fn test_contradictory_atom_keeps_arguments_and_low_confidence() {
    let sig = test_sig();
    let result = RuleGrounder.ground("T1.1 SHALL write BEFORE T1.1 SHALL seal", &sig);
    let atom = result
        .candidates
        .iter()
        .find(|c| c.predicate == "BEFORE")
        .expect("BEFORE candidate");
    assert_eq!(atom.names(), vec!["T1.1", "T1.1"]);
    assert_eq!(atom.confidence, 0.5);
}

#[test]
fn test_distinct_tasks_are_not_contradictory() {
    let sig = test_sig();
    let result = RuleGrounder.ground(
        "T1.1 SHALL write the header BEFORE T2.1 SHALL seal it",
        &sig,
    );
    assert_eq!(result.status, GroundingStatus::Grounded);
    assert_eq!(result.candidates[0].names(), vec!["T1.1", "T2.1"]);
}

#[test]
fn test_single_slot_predicate_cannot_contradict() {
    let sig = test_sig();
    let result = RuleGrounder.ground("T1.1 SHALL ALWAYS be available", &sig);
    assert_eq!(result.status, GroundingStatus::Grounded);
}

#[test]
fn test_undeclared_predicate_skips_distinctness() {
    // No ALWAYS in the signature: no slot names, so no distinctness rule.
    let sig = Signature {
        types: vec![TypeDef {
            name: "task_id".into(),
        }],
        predicates: vec![],
        constants: vec![ConstantDef {
            name: "T1.1".into(),
            type_name: "task_id".into(),
            aliases: vec![],
        }],
    };
    let result = RuleGrounder.ground("T1.1 SHALL ALWAYS be available", &sig);
    assert_ne!(result.status, GroundingStatus::Contradictory);
}

#[test]
fn test_reflexive_predicate_slots_cannot_contradict() {
    // A predicate declaring two slots with the same name is legitimately reflexive.
    let mut sig = test_sig();
    sig.predicates.push(PredicateDef {
        name: "SAME".into(),
        arguments: vec![
            ArgSlot {
                name: "x".into(),
                type_name: "task_id".into(),
            },
            ArgSlot {
                name: "x".into(),
                type_name: "task_id".into(),
            },
        ],
    });
    assert!(!distinguishes_slots(
        &sig.predicates.last().unwrap().clone()
    ));
}

#[test]
fn test_distinguishes_slots_reads_slot_names() {
    let sig = test_sig();
    for name in ["BEFORE", "AFTER", "CONCURRENTLY", "IF_THEN", "AT_MOST_ONE"] {
        let pred = sig.predicates.iter().find(|p| p.name == name).unwrap();
        assert!(distinguishes_slots(pred), "{name} distinguishes its slots");
    }
    let always = sig.predicates.iter().find(|p| p.name == "ALWAYS").unwrap();
    assert!(!distinguishes_slots(always));
}

// ── D3: match provenance ──

#[test]
fn test_name_match_is_exact() {
    let sig = test_sig();
    let result = RuleGrounder.ground("T1.1 SHALL complete BEFORE T2.1 SHALL run", &sig);
    let atom = &result.candidates[0];
    assert!(atom.exact);
    assert_eq!(atom.confidence, 0.95);
    assert_eq!(atom.arguments[0].matched, MatchKind::Name);
}

#[test]
fn test_alias_match_is_visible_and_not_exact() {
    let sig = test_sig();
    let result = RuleGrounder.ground("The migration must finish before testing starts", &sig);
    let atom = &result.candidates[0];
    assert!(!atom.exact);
    assert_eq!(atom.confidence, 0.9);
    assert!(
        atom.arguments
            .iter()
            .all(|a| !matches!(a.matched, MatchKind::Name))
    );
}

#[test]
fn test_mixed_match_kinds() {
    let sig = test_sig();
    let result = RuleGrounder.ground("T2.1 must finish before testing starts", &sig);
    let atom = &result.candidates[0];
    assert!(!atom.exact);
    assert_eq!(atom.arguments[0].matched, MatchKind::Name);
    assert!(matches!(atom.arguments[1].matched, MatchKind::Alias(_)));
}

#[test]
fn test_short_argument_list_is_incomplete() {
    let sig = test_sig();
    let result = RuleGrounder.ground("T1.1 SHALL complete before", &sig);
    assert_eq!(result.status, GroundingStatus::Incomplete);
}

#[test]
fn test_undeclared_predicate_is_not_contradictory_and_not_incomplete() {
    // ALWAYS is not declared, so there is no arity to fall short of, and no
    // distinctness rule applies (no slot names): an undeclared predicate is
    // never reported Contradictory or Incomplete.
    //
    // The slot is filled by an alias here. This used to score 0.8 because
    // `take(1)` built every argument with `BoundArgument::by_name` regardless
    // of how it matched — the text never contains "T1.1". Since single-slot
    // binding now reports the real match kind, the atom is correctly marked
    // inexact and scores 0.5, which is Ambiguous rather than Grounded.
    let sig = Signature {
        types: vec![TypeDef {
            name: "task_id".into(),
        }],
        predicates: vec![],
        constants: vec![ConstantDef {
            name: "T1.1".into(),
            type_name: "task_id".into(),
            aliases: vec!["alpha".into()],
        }],
    };
    let result = RuleGrounder.ground("alpha SHALL ALWAYS be available", &sig);
    assert_eq!(result.candidates[0].confidence, 0.5);
    assert!(!result.candidates[0].exact);
    assert_eq!(result.status, GroundingStatus::Ambiguous);
    assert_ne!(result.status, GroundingStatus::Contradictory);
    assert_ne!(result.status, GroundingStatus::Incomplete);
}

/// Ground the AT_MOST_ONE reproducer against a signature where the second
/// constant exists but cannot be reached, leaving one constant for two slots.
fn both_ground(sig: &mut Signature) -> crate::types::GroundingResult {
    sig.constants.pop();
    RuleGrounder.ground("At most one of T1.1, T1.1 SHALL be active", sig)
}

// ── Identifier boundaries (veriplan corpus regression) ──

/// A phase's tasks plus its first task, which is a prefix of the rest.
fn prefix_sig() -> Signature {
    let mk = |n: &str| ConstantDef {
        name: n.into(),
        type_name: "task_id".into(),
        aliases: vec![],
    };
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
        constants: vec![
            mk("T5.1"),
            mk("T5.10"),
            mk("T5.12"),
            mk("T5.15"),
            mk("T5.16"),
        ],
    }
}

#[test]
fn test_prefix_name_does_not_match_inside_longer_id() {
    let sig = prefix_sig();
    // T5.1 is a prefix of T5.15/T5.16: it must not be considered referenced.
    let result = RuleGrounder.ground(
        "T5.15 SHALL admit an input under a required idempotency key \
         BEFORE T5.16 SHALL reject a duplicate key",
        &sig,
    );
    assert_eq!(result.candidates[0].names(), vec!["T5.15", "T5.16"]);
    assert_eq!(result.status, GroundingStatus::Grounded);
}

#[test]
fn test_prefix_name_does_not_create_a_false_contradiction() {
    let sig = prefix_sig();
    let mut sig = sig;
    sig.constants.push(ConstantDef {
        name: "T3.12".into(),
        type_name: "task_id".into(),
        aliases: vec![],
    });
    sig.constants.push(ConstantDef {
        name: "T3.13".into(),
        type_name: "task_id".into(),
        aliases: vec![],
    });
    let result = RuleGrounder.ground(
        "T3.12 SHALL serve the OpenAPI document BEFORE T3.13 SHALL serve the schema",
        &sig,
    );
    assert_ne!(result.status, GroundingStatus::Contradictory);
    assert_eq!(result.candidates[0].names(), vec!["T3.12", "T3.13"]);
}

#[test]
fn test_trailing_sentence_dot_still_matches() {
    let sig = prefix_sig();
    let result = RuleGrounder.ground("T5.15 SHALL run BEFORE T5.16 SHALL stop.", &sig);
    assert_eq!(result.candidates[0].names(), vec!["T5.15", "T5.16"]);
}

#[test]
fn test_name_match_beats_alias_match_in_a_slot() {
    // T5.1's alias describes the first phrase; T5.15 is named outright.
    let mut sig = prefix_sig();
    sig.constants[0].aliases = vec!["admit an input".into()];
    let result = RuleGrounder.ground(
        "T5.15 SHALL admit an input BEFORE T5.16 SHALL reject it",
        &sig,
    );
    assert_eq!(result.candidates[0].names(), vec!["T5.15", "T5.16"]);
}

#[test]
fn test_alias_double_binding_is_not_a_contradiction() {
    // Both slots can only be filled by the same alias: a weak binding, not a
    // spec bug. It must not be reported as Contradictory.
    let mut sig = prefix_sig();
    sig.constants = vec![ConstantDef {
        name: "T5.1".into(),
        type_name: "task_id".into(),
        aliases: vec!["admit".into()],
    }];
    let result = RuleGrounder.ground("admit the input BEFORE admit the output", &sig);
    assert_ne!(result.status, GroundingStatus::Contradictory);
    assert_ne!(result.status, GroundingStatus::Grounded);
    assert_eq!(result.status, GroundingStatus::Ambiguous);
    assert!(result.candidates[0].diagnostic.contains("by alias"));
}

#[test]
fn test_named_twice_is_still_a_contradiction() {
    let mut sig = prefix_sig();
    sig.constants.push(ConstantDef {
        name: "T5.15".into(),
        type_name: "task_id".into(),
        aliases: vec![],
    });
    let result = RuleGrounder.ground("T5.15 SHALL write BEFORE T5.15 SHALL seal", &sig);
    assert_eq!(result.status, GroundingStatus::Contradictory);
}

// ── Inverse diagnostic: named but unbound ──

#[test]
fn test_unused_identifier_reports_a_shadowed_named_task() {
    // ALWAYS takes one slot; the text names two tasks. The second is named
    // but cannot be bound, and the caller must be able to see that.
    let sig = prefix_sig();
    let result = RuleGrounder.ground("T5.15 SHALL ALWAYS mention T5.16", &sig);
    assert_eq!(result.unused_identifiers, vec!["T5.16".to_string()]);
    assert!(result.absent_identifiers.is_empty());
}

#[test]
fn test_unused_identifier_is_empty_when_every_named_task_is_bound() {
    let sig = prefix_sig();
    let result = RuleGrounder.ground("T5.15 SHALL run BEFORE T5.16 SHALL stop", &sig);
    assert!(result.unused_identifiers.is_empty());
}

#[test]
fn test_unused_identifier_is_empty_without_a_candidate() {
    // No predicate: every named constant would be "unused" for the wrong
    // reason, so the field stays empty and the status carries the diagnosis.
    let sig = prefix_sig();
    let result = RuleGrounder.ground("T5.15 does something", &sig);
    assert_eq!(result.status, GroundingStatus::Ungroundable);
    assert!(result.unused_identifiers.is_empty());
}

#[test]
fn test_unused_identifier_ignores_prefix_collisions() {
    // T5.1 is a prefix of T5.15/T5.16 but is not named, so it is neither
    // bound nor reported as unused.
    let sig = prefix_sig();
    let result = RuleGrounder.ground(
        "T5.15 SHALL admit an input BEFORE T5.16 SHALL reject it",
        &sig,
    );
    assert!(result.unused_identifiers.is_empty());
}

#[test]
fn test_unused_identifier_survives_json() {
    let result = GroundingResult {
        candidates: vec![],
        status: GroundingStatus::Grounded,
        absent_identifiers: vec![],
        unused_identifiers: vec!["T5.16".into()],
    };
    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("unused_identifiers"));
    assert!(json.contains("T5.16"));
}
