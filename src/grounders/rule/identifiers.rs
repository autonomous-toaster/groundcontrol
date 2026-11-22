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
    // The scanner walks in character indices so that a candidate cannot be cut
    // mid-character, but the boundary predicates take *byte* offsets (they slice
    // the string). Keep the mapping and never pass one where the other is meant:
    // mixing them panics inside a multi-byte character, e.g. on "H2→H1.1".
    let offsets: Vec<usize> = nl
        .char_indices()
        .map(|(byte, _)| byte)
        .chain(std::iter::once(nl.len()))
        .collect();
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
            let Some(candidate) = chars.get(i..end) else {
                continue;
            };
            let candidate: String = candidate.iter().collect();
            let shape = name_shape(&candidate);
            // `offsets` converts character indices to byte offsets. Both lookups
            // are total: a missing entry simply cannot form an identifier.
            let (Some(from), Some(to)) = (offsets.get(i), offsets.get(end)) else {
                continue;
            };
            if fits_any(&shape, &patterns) && boundary_before(nl, *from) && boundary_after(nl, *to)
            {
                best = Some(len);
            }
        }
        match best {
            Some(len) => {
                let Some(span) = chars.get(i..i + len) else {
                    i += 1;
                    continue;
                };
                let candidate: String = span.iter().collect();
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
    name_index(text, name).is_some()
}

/// Byte offset of the first case-insensitive occurrence of `needle` in `haystack`.
///
/// Case-insensitive matching must not go through a lowercased copy: lowercasing
/// can change byte length (U+0130 "İ" becomes "i̇", 2 bytes to 3), so an offset
/// found in the copy is not a valid offset in the original and slicing there
/// panics inside a multi-byte character. Everything here returns offsets in
/// `haystack` itself, and only ever slices at `char_indices` boundaries.
pub(super) fn find_ci(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }
    let wanted = needle.chars().count();
    for (byte, _) in haystack.char_indices() {
        let rest = haystack.get(byte..)?;
        let enough = rest.chars().count() >= wanted;
        let matched = enough
            && rest
                .chars()
                .zip(needle.chars())
                .all(|(h, n)| h.eq_ignore_ascii_case(&n));
        if matched {
            return Some(byte);
        }
    }
    None
}

/// Byte offset of the first boundary-valid occurrence of `name` in `text`.
///
/// Position, not just presence, is what lets a caller bind a slot by the order
/// the author wrote the tasks in rather than by the order the signature happens
/// to declare them.
pub(super) fn name_index(text: &str, name: &str) -> Option<usize> {
    if name.is_empty() {
        return None;
    }
    let mut from = 0;
    while let Some(tail) = text.get(from..) {
        let Some(rel) = tail.find(name) else {
            break;
        };
        let start = from + rel;
        let end = start + name.len();
        if boundary_before(text, start) && boundary_after(text, end) {
            return Some(start);
        }
        // Advance one character, not one match, so overlapping runs are tried.
        match text.get(start..).and_then(|t| t.chars().next()) {
            Some(c) => from = start + c.len_utf8(),
            None => break,
        }
    }
    None
}

/// The identifier must not be glued to a preceding one ("XT5.1").
pub(super) fn boundary_before(text: &str, at: usize) -> bool {
    // An offset that is not a char boundary cannot start an identifier, so the
    // total lookup answering None is the conservative answer, not a panic.
    let Some(head) = text.get(..at) else {
        return false;
    };
    match head.chars().next_back() {
        None => true,
        Some(c) => !is_ident_char(c),
    }
}

/// The identifier must not be glued to a following one ("T5.15", "T5.1.5").
/// A trailing "." is sentence punctuation — "T5.1." ends a sentence, "T5.1.5"
/// continues an identifier — so the dot only counts when no digit follows.
pub(super) fn boundary_after(text: &str, at: usize) -> bool {
    let Some(tail) = text.get(at..) else {
        return false;
    };
    let mut it = tail.chars();
    match it.next() {
        None => true,
        Some('.') => !it.next().is_some_and(|n| n.is_ascii_digit()),
        Some(c) => !is_ident_char(c),
    }
}

pub(super) fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
