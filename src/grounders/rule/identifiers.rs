//! Identifier handling: what a constant name looks like, where it occurs in
//! text, and which referenced identifiers the signature does not declare.

use crate::types::{ConstantDef, Signature};

// ── Absent identifiers ──

/// Longest token we are willing to consider a constant-shaped identifier.
const MAX_IDENTIFIER_LEN: usize = 12;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CharClass {
    Alpha,
    Digit,
    Other,
}

pub(super) fn class_of(c: char) -> CharClass {
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
pub(super) fn name_shape(name: &str) -> Vec<CharClass> {
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
pub(super) fn extract_absent_identifiers(nl: &str, sig: &Signature) -> Vec<String> {
    // One pattern per declared constant type, taken from that type's longest
    // constant name. `name_shape` collapses runs, so T1.1 and T2.10 share a
    // pattern: the run *lengths* are not part of the shape, and must never be,
    // because a signature that happens to contain only short ids would
    // otherwise cap the length of the ids it can recognise.
    let mut patterns: Vec<Vec<CharClass>> = Vec::new();
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
        patterns.push(shape);
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
    //
    // Boundaries are enforced here, not just in `contains_name`: without them
    // "T2.10" is reported as its prefix "T2.1", which accuses the author of
    // citing a task they did not write — and, when the prefix happens to be
    // declared, silently swallows the real reference instead.
    let mut i = 0;
    while i < chars.len() {
        let mut best: Option<usize> = None;
        let limit = (chars.len() - i).min(MAX_IDENTIFIER_LEN);
        for len in 2..=limit {
            let end = i + len;
            let candidate: String = chars[i..end].iter().collect();
            let shape = name_shape(&candidate);
            if fits_any(&shape, &patterns) && boundary_before(nl, i) && boundary_after(nl, end) {
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
pub(super) fn fits_any(shape: &[CharClass], patterns: &[Vec<CharClass>]) -> bool {
    shape.first() == Some(&CharClass::Alpha) && patterns.iter().any(|p| shape == p.as_slice())
}

/// Check if a constant appears in the NL text (by name or alias).
pub(super) fn appears_in(c: &ConstantDef, nl: &str, lower: &str) -> bool {
    if contains_name(nl, &c.name) {
        return true;
    }
    // Alias match (e.g., "migration" from task description)
    c.aliases
        .iter()
        .any(|a| contains_name(lower, &a.to_lowercase()))
}

/// Does `name` occur in `text` as a whole identifier?
///
/// Plain substring containment is wrong for identifiers whose names share a
/// prefix: `T5.1` is a substring of `T5.15`, so "T5.15 SHALL ... BEFORE T5.16"
/// would match T5.1 twice and certify a self-ordering constraint. A match only
/// counts when neither side continues the identifier.
pub(super) fn contains_name(text: &str, name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut from = 0;
    while let Some(rel) = text[from..].find(name) {
        let start = from + rel;
        let end = start + name.len();
        if boundary_before(text, start) && boundary_after(text, end) {
            return true;
        }
        // Advance one character, not one match, so overlapping runs are tried.
        match text[start..].chars().next() {
            Some(c) => from = start + c.len_utf8(),
            None => break,
        }
    }
    false
}

/// The identifier must not be glued to a preceding one ("XT5.1").
pub(super) fn boundary_before(text: &str, at: usize) -> bool {
    match text[..at].chars().next_back() {
        None => true,
        Some(c) => !is_ident_char(c),
    }
}

/// The identifier must not be glued to a following one ("T5.15", "T5.1.5").
/// A trailing "." is sentence punctuation — "T5.1." ends a sentence, "T5.1.5"
/// continues an identifier — so the dot only counts when no digit follows.
pub(super) fn boundary_after(text: &str, at: usize) -> bool {
    let mut it = text[at..].chars();
    match it.next() {
        None => true,
        Some('.') => !it.next().is_some_and(|n| n.is_ascii_digit()),
        Some(c) => !is_ident_char(c),
    }
}

pub(super) fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
