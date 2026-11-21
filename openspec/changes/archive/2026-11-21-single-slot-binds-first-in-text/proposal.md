## Why

The client (B3, kata 382j) reported that a single-slot predicate binds in *signature* order,
so the vocabulary dictates how a sentence is read:

```
Signature T1.1, T1.10.   Body: T1.10 SHALL ALWAYS mention T1.1
binds T1.10 ... wait, binds the signature-first constant, leaving the subject unbound
```

Concretely, `take(1)` took the first constant in signature order, so the subject of the
sentence could go unbound and be reported as `unused_identifiers`. The consumer renders that as
"the text names T1.1 but no slot used it", which is a false statement about the author's
sentence — and one they cannot argue with, because both ids exist.

## What Changes

- A single-slot predicate binds the constant whose mention occurs earliest in the text, with
  signature order only as a deterministic tie-break. Statements that name one task, or none,
  are unaffected.
- The chosen constant is bound through `bind_slot`, so the reported `MatchKind` is the real one.
  `take(1)` previously stamped `BoundArgument::by_name` on every argument and ignored
  `allow_alias`, so an alias-only match was reported as a name match at 0.95 and bound even when
  the statement cited an undeclared task (kata gw3m). ALWAYS now obeys the same alias-suppression
  rule as every other predicate.

## Capabilities

### New Capabilities

<!-- None -->

### Modified Capabilities

- `argument-assignment`: adds the single-slot text-order requirement.

## Measured effect

Veriplan's 24 spec files, 429 groundable statements, ground against its active change's tasks.md
(a deliberately mismatched signature, so absent identifiers are common): 9 statements change.
4 move to the task the text names first (the intended move), 2 keep the same constant with the
correct match kind at 0.90/inexact, 3 stop grounding on an alias-only constant and report the
undeclared id instead (kata gw3m).

## Non-goals

- No change to `GroundingStatus`, `GroundedAtom`, `BoundArgument`, `MatchKind` or `confidence`.
- No change to CONCURRENTLY / AT_MOST_ONE two-slot selection order, which would reorder
  symmetric operands without changing meaning, and would move corpus output for no benefit.

## Impact

- `src/grounders/rule.rs`, `src/grounders/rule/identifiers.rs`, tests in
  `src/grounders/rule/identifier_tests.rs`.
