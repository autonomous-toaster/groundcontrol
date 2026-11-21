## Why

The client (veriplan, pinned at v0.2.0) reported two defects with the same cause, both worse than
the behaviour they replaced because the tool now names a wrong identifier instead of staying
silent.

```
Signature T1.1, T1.2.  Body: T2.10 SHALL complete BEFORE T1.1 SHALL start.
reported: absent_identifiers = ["T2.1"]      <- truncated, accuses a task the author never wrote

Signature T1.1.       Body: T1.10 SHALL complete BEFORE T1.1 SHALL start.
reported: absent_identifiers = []            <- the real foreign reference is swallowed
```

`extract_absent_identifiers` bounded candidate length by the longest *known* constant name and
took the longest fit under that bound, with no boundary check. With T1.1 as the longest constant,
`T2.10` could not fit, so its prefix `T2.1` was reported; and `T1.10` resolved to the declared
`T1.1` and vanished.

`contains_name` was already boundary-aware, but this path never called it — so the boundary rule
covered slot binding and left the diagnostic tokenizer untouched.

## What Changes

- Candidate length is bounded by a constant sanity cap, not by the signature's longest name.
- Candidates must satisfy the same before/after boundary predicates as `contains_name`, so a
  prefix of a longer identifier is never reported and a reported token is byte-identical to the
  input.
- The multi-digit cases are covered by regression tests, including the B2 acceptance case.

## Capabilities

### New Capabilities

<!-- None -->

### Modified Capabilities

- `grounding-outcome`: the identifier-shape requirement now bounds length by a cap rather than by
  the longest declared name, and requires boundary-identical tokens.

## Non-goals

- No change to `GroundingStatus`, `GroundedAtom`, `BoundArgument`, `MatchKind` or `confidence`.
- No change to which constant wins a slot when several are named: `take(n)` still selects in
  signature order for single-slot predicates. Reported to the client as an observation, not
  fixed here, because it is not part of B1/B2 and would move corpus results.

## Impact

- `src/grounders/rule/identifiers.rs`, new `src/grounders/rule/identifier_tests.rs`.
