//! Core types: Signature, GroundedAtom, GroundingResult, Directive.

use serde::{Deserialize, Serialize};

// ── System Signature (GinSign §2.2) ──

/// A many-sorted system signature: the vocabulary of a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub types: Vec<TypeDef>,
    pub predicates: Vec<PredicateDef>,
    pub constants: Vec<ConstantDef>,
}

impl Signature {
    pub fn summary(&self) -> SignatureSummary {
        SignatureSummary {
            tasks: self.constants.len(),
            predicates: self.predicates.len(),
            types: self.types.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureSummary {
    pub tasks: usize,
    pub predicates: usize,
    pub types: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredicateDef {
    pub name: String,
    pub arguments: Vec<ArgSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgSlot {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantDef {
    pub name: String,
    pub type_name: String,
    /// Alternative NL forms for fuzzy matching (from task descriptions).
    pub aliases: Vec<String>,
}

// ── Grounding ──

/// How a constant was recognized in the NL text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase", tag = "kind", content = "alias")]
pub enum MatchKind {
    /// The constant's name occurs literally in the text (e.g. "T1.1").
    Name,
    /// One of the constant's aliases occurs in the text (e.g. "migration").
    Alias(String),
}

/// A constant bound to one predicate slot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoundArgument {
    /// The constant's name.
    pub constant: String,
    /// How the constant was recognized.
    pub matched: MatchKind,
}

impl BoundArgument {
    pub fn by_name(constant: impl Into<String>) -> Self {
        Self {
            constant: constant.into(),
            matched: MatchKind::Name,
        }
    }

    pub fn by_alias(constant: impl Into<String>, alias: impl Into<String>) -> Self {
        Self {
            constant: constant.into(),
            matched: MatchKind::Alias(alias.into()),
        }
    }

    pub fn is_exact(&self) -> bool {
        self.matched == MatchKind::Name
    }
}

/// A grounded atom: predicate + concrete arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedAtom {
    pub predicate: String,
    /// Arguments in slot order.
    pub arguments: Vec<BoundArgument>,
    /// True when every argument matched by name (no alias was used).
    pub exact: bool,
    /// One-line explanation of why `confidence` was assigned, rendered from
    /// `exact`, the argument provenance, the predicate's slot names and the
    /// result's absent identifiers. Present for source compatibility with the
    /// published v0.1.0 shape; it is a convenience rendering, never the source
    /// of truth. Callers that need to act should read the structured fields.
    pub diagnostic: String,
    /// Structural confidence, not correctness. Values:
    ///
    /// | condition                                         | value |
    /// |---------------------------------------------------|-------|
    /// | every slot filled by name                         | 0.95  |
    /// | every slot filled, at least one by alias          | 0.90  |
    /// | every slot filled, predicate undeclared           | 0.80  |
    /// | slots filled by a repeated constant               | 0.50  |
    /// | argument list short, or longer than the signature | 0.50  |
    ///
    /// A candidate is only emitted with a confidence above 0.0. Identity and
    /// arity are reported explicitly by `exact` and by the argument list.
    pub confidence: f64,
}

impl GroundedAtom {
    /// The bound constants, in slot order.
    pub fn names(&self) -> Vec<&str> {
        self.arguments.iter().map(|a| a.constant.as_str()).collect()
    }
}

/// Overall status of a grounding attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GroundingStatus {
    /// At least one candidate with confidence >= 0.8.
    Grounded,
    /// The statement is right in kind but refers to something this signature
    /// cannot resolve. Two causes, both reported in data rather than here:
    ///
    /// - `absent_identifiers` is non-empty: the text names constants of a
    ///   declared type that no constant provides ("T2.6 is not a task of this
    ///   change").
    /// - `absent_identifiers` is empty: a declared predicate needing two or
    ///   more slots was only partially filled.
    ///
    /// Callers that need the cause must read the field:
    /// `incomplete && !absent_identifiers.is_empty()` is the foreign-reference
    /// case, `incomplete && absent_identifiers.is_empty()` the arity case. The
    /// status alone deliberately does not encode it, because the foreign case is
    /// a *fact about the signature*, not a property of the statement.
    Incomplete,
    /// Candidates exist but all below 0.8, and nothing better explains why.
    Ambiguous,
    /// A predicate received the same constant in slots that require different
    /// constants. The statement is well-formed but meaningless.
    Contradictory,
    /// No candidates found.
    Ungroundable,
}

impl GroundingStatus {
    /// Lower-case name used in CLI and JSON output.
    pub fn as_str(&self) -> &'static str {
        match self {
            GroundingStatus::Grounded => "grounded",
            GroundingStatus::Incomplete => "incomplete",
            GroundingStatus::Ambiguous => "ambiguous",
            GroundingStatus::Contradictory => "contradictory",
            GroundingStatus::Ungroundable => "ungroundable",
        }
    }
}

/// Result of grounding one NL span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingResult {
    pub candidates: Vec<GroundedAtom>,
    pub status: GroundingStatus,
    /// Constant-shaped tokens in the text that look like a constant of a type
    /// declared in the signature, but are not provided by any constant.
    /// E.g. "T2.6" when the signature only declares "T1.1".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub absent_identifiers: Vec<String>,
    /// The mirror of `absent_identifiers`: constants the text **names
    /// explicitly** that the grounding did not bind to any slot.
    ///
    /// Empty when no candidate was produced at all — there the diagnosis is the
    /// missing predicate, not the unused reference. Non-empty means the grounder
    /// dropped a task the author wrote, which is a grounding weakness, not a
    /// statement defect: the slots were filled from something weaker (an alias,
    /// or a constant that shares a prefix) while a named constant went unused.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unused_identifiers: Vec<String>,
}

// ── Directives (feedback to the LLM) ──

/// A close match suggestion for ungroundable text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseMatch {
    pub constant: String,
    pub alias: String,
    pub similarity: f64,
}

/// A structured directive telling the LLM what to fix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directive {
    pub severity: String,
    pub element: String,
    pub detail: String,
    pub suggested_action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_matches: Option<Vec<CloseMatch>>,
}

// ── Check output ──

/// One grounded input with its directives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedInput {
    pub source: String,
    pub text: String,
    pub groundings: Vec<GroundedAtom>,
    pub status: GroundingStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub absent_identifiers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unused_identifiers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub directives: Vec<Directive>,
}

/// Summary counts, one bucket per `GroundingStatus`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckSummary {
    pub total: usize,
    pub grounded: usize,
    pub ambiguous: usize,
    pub ungroundable: usize,
    #[serde(default)]
    pub incomplete: usize,
    #[serde(default)]
    pub contradictory: usize,
}

/// Full check output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckOutput {
    pub plan_name: String,
    pub signature: SignatureSummary,
    pub inputs: Vec<GroundedInput>,
    pub summary: CheckSummary,
}

// ── Grounder trait ──

/// Pluggable grounding strategy.
pub trait Grounder: Send + Sync {
    /// Ground one NL span against a signature.
    fn ground(&self, nl: &str, sig: &Signature) -> GroundingResult;

    /// Ground multiple NL spans.
    #[allow(dead_code)] // convenience for library callers; the CLI loops explicitly
    fn ground_batch(&self, inputs: &[&str], sig: &Signature) -> Vec<GroundingResult> {
        inputs.iter().map(|nl| self.ground(nl, sig)).collect()
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
    use super::*;

    #[test]
    fn test_signature_json_roundtrip() {
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
                name: "T1.1".into(),
                type_name: "task_id".into(),
                aliases: vec!["setup".into()],
            }],
        };
        let json = serde_json::to_string_pretty(&sig).unwrap();
        let back: Signature = serde_json::from_str(&json).unwrap();
        assert_eq!(back.types.len(), 1);
        assert_eq!(back.predicates.len(), 1);
        assert_eq!(back.constants.len(), 1);
        assert_eq!(back.constants[0].name, "T1.1");
    }

    #[test]
    fn test_signature_summary() {
        let sig = Signature {
            types: vec![TypeDef { name: "t".into() }],
            predicates: vec![],
            constants: vec![
                ConstantDef {
                    name: "T1.1".into(),
                    type_name: "t".into(),
                    aliases: vec![],
                },
                ConstantDef {
                    name: "T1.2".into(),
                    type_name: "t".into(),
                    aliases: vec![],
                },
            ],
        };
        let s = sig.summary();
        assert_eq!(s.tasks, 2);
        assert_eq!(s.predicates, 0);
        assert_eq!(s.types, 1);
    }

    #[test]
    fn test_grounding_status_equality() {
        assert_eq!(GroundingStatus::Grounded, GroundingStatus::Grounded);
        assert_ne!(GroundingStatus::Grounded, GroundingStatus::Ambiguous);
        assert_ne!(GroundingStatus::Ambiguous, GroundingStatus::Ungroundable);
        assert_ne!(GroundingStatus::Incomplete, GroundingStatus::Ambiguous);
        assert_ne!(GroundingStatus::Contradictory, GroundingStatus::Incomplete);
    }

    #[test]
    fn test_grounding_status_strings_are_stable() {
        assert_eq!(GroundingStatus::Grounded.as_str(), "grounded");
        assert_eq!(GroundingStatus::Incomplete.as_str(), "incomplete");
        assert_eq!(GroundingStatus::Ambiguous.as_str(), "ambiguous");
        assert_eq!(GroundingStatus::Contradictory.as_str(), "contradictory");
        assert_eq!(GroundingStatus::Ungroundable.as_str(), "ungroundable");
    }

    #[test]
    fn test_result_roundtrip_preserves_absent_identifiers() {
        let result = GroundingResult {
            candidates: vec![],
            status: GroundingStatus::Incomplete,
            absent_identifiers: vec!["T2.6".into()],
            unused_identifiers: vec![],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"T2.6\""));
        assert!(json.contains("\"incomplete\""));
        let back: GroundingResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.absent_identifiers, vec!["T2.6".to_string()]);
        assert_eq!(back.status, GroundingStatus::Incomplete);
    }

    #[test]
    fn test_result_omits_empty_absent_identifiers() {
        let result = GroundingResult {
            candidates: vec![],
            status: GroundingStatus::Ungroundable,
            absent_identifiers: vec![],
            unused_identifiers: vec![],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("absent_identifiers"));
    }

    #[test]
    fn test_match_kind_serialization() {
        let name = serde_json::to_string(&BoundArgument::by_name("T1.1")).unwrap();
        assert!(name.contains("\"name\""));
        let alias = serde_json::to_string(&BoundArgument::by_alias("T1.1", "setup")).unwrap();
        assert!(alias.contains("\"alias\""));
        assert!(alias.contains("\"setup\""));
    }

    #[test]
    fn test_grounded_atom_construction() {
        let atom = GroundedAtom {
            predicate: "BEFORE".into(),
            arguments: vec![
                BoundArgument::by_name("T1.1"),
                BoundArgument::by_alias("T1.2", "deps"),
            ],
            exact: false,
            confidence: 0.95,
            diagnostic: "matched by alias: T1.2 as 'deps'".into(),
        };
        assert_eq!(atom.predicate, "BEFORE");
        assert_eq!(atom.arguments.len(), 2);
        assert_eq!(atom.names(), vec!["T1.1", "T1.2"]);
        assert!(atom.confidence > 0.9);
        assert!(!atom.exact);
        assert!(atom.diagnostic.contains("alias"));
    }

    #[test]
    fn test_directive_construction() {
        let d = Directive {
            severity: "blocker".into(),
            element: "spec.md#L3".into(),
            detail: "No match found".into(),
            suggested_action: "Add task ID".into(),
            close_matches: Some(vec![CloseMatch {
                constant: "T1.1".into(),
                alias: "setup".into(),
                similarity: 0.85,
            }]),
        };
        assert_eq!(d.severity, "blocker");
        assert!(d.close_matches.is_some());
        assert_eq!(d.close_matches.unwrap().len(), 1);
    }

    #[test]
    fn test_check_output_serialization() {
        let output = CheckOutput {
            plan_name: "test".into(),
            signature: SignatureSummary {
                tasks: 1,
                predicates: 1,
                types: 1,
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
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("plan_name"));
        assert!(json.contains("signature"));
        assert!(json.contains("summary"));
    }
}
