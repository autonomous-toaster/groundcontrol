## Why

`absent_identifiers` reports references the signature cannot satisfy. The mirror case has no
signal: a constant the text **names explicitly** that the grounding never binds, because the
slots were filled from something weaker. That is how the prefix-collision defect shipped —
`T5.15` and `T5.16` were written in the body and silently replaced by `T5.1` twice, and
nothing in the result said a named task had been dropped.

Identifier boundaries remove the specific cause, but not the class: any future weakness in slot
filling (an alias winning a slot, a predicate with fewer slots than the statement names, a new
heuristic) has the same signature — a task the author wrote, present in the text, absent from
the arguments — and today it is invisible.

## What Changes

- `GroundingResult.unused_identifiers: Vec<String>` — constants the text names that no
  candidate bound. Empty when no candidate was produced (there the missing predicate is the
  diagnosis).
- `GroundedInput.unused_identifiers` carries it to the CLI, plus a directive naming the task.
- The CLI reports it without failing the check: a grounding weakness is not a spec defect, so
  it is printed and exits 0 rather than turning a green corpus red.

## Capabilities

### New Capabilities

<!-- None -->

### Modified Capabilities

- `grounding-outcome`: adds the unbound-identifier requirement.

## Non-goals

- No change to `status` or `confidence`: the new data is informational, and a caller decides
  whether an unbound reference is a failure. veriplan already has strictness profiles.
- No sixth `GroundingStatus` variant.
- No change to `GroundedAtom`, `BoundArgument` or `MatchKind`: the field is additive on
  `GroundingResult`, which the consumer reads but never constructs.

## Impact

- `src/types.rs`, `src/grounders/rule.rs`, `src/directives.rs`, `src/main.rs`.
- Source-compatible for any caller that reads results; a caller that *constructs*
  `GroundingResult` must add the field.
