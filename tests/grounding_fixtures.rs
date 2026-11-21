//! Golden-file tests: the grounding result for a fixed set of cases, as JSON.
//!
//! The fixture in `tests/fixtures/grounding_cases.json` is the contract this
//! crate publishes to its consumers: each case pins the full result (status,
//! predicate, arguments with their match kind, exactness, confidence, absent and
//! unused identifiers) for a hand-built signature and statement. A consumer that
//! cannot observe a behaviour through its own CLI can diff this file instead of
//! trusting a changelog.
//!
//! Regenerate after an intended behaviour change:
//!
//!     UPDATE_FIXTURES=1 cargo test --test grounding_fixtures
//!
//! then read the diff of `grounding_cases.json` before committing it.

use groundcontrol::grounders::RuleGrounder;
use groundcontrol::types::*;
use serde_json::{Value, json};
use std::path::PathBuf;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/grounding_cases.json")
}

fn task(n: &str) -> ConstantDef {
    ConstantDef {
        name: n.to_string(),
        type_name: "task_id".into(),
        aliases: vec![],
    }
}

fn task_with_aliases(n: &str, aliases: &[&str]) -> ConstantDef {
    ConstantDef {
        name: n.to_string(),
        type_name: "task_id".into(),
        aliases: aliases.iter().map(|a| a.to_string()).collect(),
    }
}

fn slot(name: &str) -> ArgSlot {
    ArgSlot {
        name: name.into(),
        type_name: "task_id".into(),
    }
}

fn two_slot(predicate: &str) -> PredicateDef {
    PredicateDef {
        name: predicate.into(),
        arguments: vec![slot("earlier"), slot("later")],
    }
}

fn one_slot(predicate: &str) -> PredicateDef {
    PredicateDef {
        name: predicate.into(),
        arguments: vec![slot("target")],
    }
}

fn signature(predicates: Vec<PredicateDef>, constants: Vec<ConstantDef>) -> Signature {
    Signature {
        types: vec![TypeDef {
            name: "task_id".into(),
        }],
        predicates,
        constants,
    }
}

/// Every published behaviour, in one place.
fn cases() -> Vec<(&'static str, Signature, &'static str)> {
    vec![
        // B3: a single-slot predicate follows the text, not the signature.
        (
            "single_slot_subject_is_later_in_signature",
            signature(
                vec![one_slot("ALWAYS")],
                vec![task("T1.1"), task("T1.10")],
            ),
            "T1.10 SHALL ALWAYS mention T1.1",
        ),
        (
            "single_slot_reversed_sentence",
            signature(
                vec![one_slot("ALWAYS")],
                vec![task("T1.1"), task("T1.10")],
            ),
            "T1.1 SHALL ALWAYS mention T1.10",
        ),
        // B1: a foreign multi-digit identifier is reported verbatim.
        (
            "absent_multi_digit_identifier",
            signature(
                vec![two_slot("BEFORE")],
                vec![task("T1.1"), task("T1.2")],
            ),
            "T2.10 SHALL complete BEFORE T1.1 SHALL start.",
        ),
        (
            "absent_multi_digit_matches_declared_prefix",
            signature(vec![two_slot("BEFORE")], vec![task("T1.1")]),
            "T1.10 SHALL complete BEFORE T1.1 SHALL start.",
        ),
        // B2: a prefix of a longer identifier is never bound.
        (
            "boundary_does_not_bind_a_prefix",
            signature(
                vec![two_slot("BEFORE")],
                vec![task("T1.1"), task("T1.2"), task("T1.10")],
            ),
            "T1.10 SHALL complete BEFORE T1.2 SHALL start.",
        ),
        // gw3m: an alias match is reported as an alias, never stamped as a name.
        (
            "alias_match_is_reported_as_alias",
            signature(
                vec![two_slot("BEFORE")],
                vec![
                    task_with_aliases("T2.1", &["migration"]),
                    task_with_aliases("T2.2", &["testing"]),
                ],
            ),
            "The migration must finish before testing starts",
        ),
        // D2: a self-relation is contradictory, not a low-confidence match.
        (
            "self_relation_is_contradictory",
            signature(
                vec![two_slot("BEFORE")],
                vec![task("T1.1"), task("T1.2")],
            ),
            "T1.1 SHALL write the header BEFORE T1.1 SHALL seal it",
        ),
    ]
}

fn render(name: &str, sig: &Signature, text: &str) -> Value {
    let result = RuleGrounder.ground(text, sig);
    let candidates: Vec<Value> = result
        .candidates
        .iter()
        .map(|c| {
            let args: Vec<Value> = c
                .arguments
                .iter()
                .map(|a| match &a.matched {
                    MatchKind::Name => json!({ "constant": a.constant, "kind": "name" }),
                    MatchKind::Alias(alias) => {
                        json!({ "constant": a.constant, "kind": "alias", "alias": alias })
                    }
                })
                .collect();
            json!({
                "predicate": c.predicate,
                "arguments": args,
                "exact": c.exact,
                "confidence": c.confidence,
            })
        })
        .collect();
    json!({
        "name": name,
        "text": text,
        "expected": {
            "status": result.status.as_str(),
            "candidates": candidates,
            "absent_identifiers": result.absent_identifiers,
            "unused_identifiers": result.unused_identifiers,
        },
    })
}

#[test]
fn grounding_cases_match_the_published_fixture() {
    let rendered: Vec<Value> = cases()
        .iter()
        .map(|(name, sig, text)| render(name, sig, text))
        .collect();
    let actual = serde_json::to_string_pretty(&Value::Array(rendered)).unwrap() + "\n";
    let path = fixture_path();

    if std::env::var("UPDATE_FIXTURES").is_ok() {
        std::fs::write(&path, &actual).expect("write fixture");
        return;
    }

    let expected = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "cannot read {}: {e}\nRegenerate with: UPDATE_FIXTURES=1 cargo test --test grounding_fixtures",
            path.display(),
        )
    });
    assert_eq!(
        expected, actual,
        "grounding behaviour differs from tests/fixtures/grounding_cases.json; if the change is \
         intended, regenerate the fixture and review its diff"
    );
}
