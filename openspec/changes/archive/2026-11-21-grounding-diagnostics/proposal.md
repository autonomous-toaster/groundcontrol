## Why

`RuleGrounder` reports three statuses (Grounded / Ambiguous / Ungroundable) and a single
opaque `confidence` float. Callers cannot tell *why* a statement failed to ground, and the
distinctions that matter are collapsed:

- A statement referencing a constant-shaped token that is **not in the Signature**
  (e.g. `T2.6` when the change only declares `T1.1`) is reported as a **short argument
  list** (arity fallback, confidence 0.5, status Ambiguous). The caller then tells the author
  to "add explicit task IDs", which the author already did. This is the blocker that stops
  spec-debt pay-down in real repositories.
- A statement reusing the **same constant in two semantically distinct slots**
  (`T1.1 SHALL write BEFORE T1.1 SHALL seal`) is reported as **Grounded at 0.95**,
  because `appears_in` is a `contains` test, not a token scan. The tool certifies a
  self-ordering constraint that no model can meaningfully satisfy.
- Because every word of a task description becomes an alias, a statement can ground at 0.95
  on an **incidental shared word**. The result carries no record of how each argument was
  matched, so a caller cannot discount an alias hit.

The fix is diagnostic surface, not new heuristics: name the failure, and say how each
argument was found.

## What Changes

- **Absent identifiers**: `GroundingResult` gains `absent_identifiers` — constant-shaped
  tokens in the text whose type is declared in the Signature but which no
  `ConstantDef` provides. Extraction is driven by `TypeDef` naming, not hard-coded syntax.
- **Match provenance**: `GroundedAtom.arguments` becomes `Vec<BoundArgument>` carrying the
  constant name and how it matched (`name` / `alias` plus the matched literal).
- **Slot distinctness**: a predicate whose arg slots are typed the same but semantically
  distinct must not receive the same constant twice. Violations produce
  `GroundingStatus::Contradictory`, never Grounded.
- **New statuses**: `Incomplete` (predicate found, argument list short or absent ids present)
  and `Contradictory` (distinctness/duplicate violation).
- **Confidence semantics**: `confidence` is documented as *structural* confidence
  (slots filled, arity exact, identity match) and `GroundedAtom.exact` records whether every
  argument matched by name.
- CLI directives and JSON output carry the new fields; a JSON round-trip preserves them.

## Capabilities

### New Capabilities

- `grounding-outcome`: the data a grounding call returns — statuses, absent identifiers,
  per-argument provenance, and the confidence contract.
- `argument-assignment`: how constants are bound to predicate slots, and when a binding is
  rejected as contradictory.

### Modified Capabilities

<!-- None: openspec/specs/ is empty -->

## Non-goals

- No change to `PREDICATE_KEYWORDS` or to the positional assignment heuristics.
- No new predicates, no new temporal operators.
- No caller-specific code: groundcontrol stays domain-agnostic, veriplan adapts.
- No attempt to fix veriplan's `[] ( true )` LTL rendering; that is a veriplan-side bug.

## Impact

- `src/types.rs` (result/atom types), `src/grounders/rule.rs` (extraction + assignment),
  `src/main.rs` (directives, human + JSON output).
- Public API break for `GroundedAtom.arguments` and `GroundingStatus`; veriplan must update.
- No new dependencies.
