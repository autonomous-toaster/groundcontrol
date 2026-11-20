## Why

A consumer (veriplan) reported 9 requirements out of 62 grounded as `Contradictory` where the
body names two different task ids and both slots filled with one constant the body never
mentions:

```
body:     T5.15 SHALL admit an input under a required idempotency key BEFORE T5.16 SHALL ...
reported: (T5.1, T5.1)
```

The captured constant is always the phase's first task. The cause is not alias ordering, as
first suspected, but `str::contains`: `T5.1` is a **prefix of** `T5.15`, so `contains_name`
reports T5.1 as occurring in a text that never names it, twice — once per slot — and the
distinctness rule then certifies a self-ordering constraint. Every affected constant is a
prefix of a longer one (T3.1⊂T3.12, T5.1⊂T5.10/5.11/5.12/5.15/5.16, T7.1⊂T7.11/7.12).

That is D1's mistake one layer down: a grounding weakness reported as a spec bug, sending the
author to fix prose that is correct.

## What Changes

- Constant names match only on identifier boundaries, so `T5.1` no longer matches inside
  `T5.15`. A trailing `.` is a boundary (sentence end) unless a digit follows.
- Slot filling prefers a name match over an alias match, so an alias-only constant can never
  displace a constant the text names literally.
- `Contradictory` requires the repeated constant to be name-matched in **both** slots — the
  author wrote one task where two are required. An alias that wins both slots is a weak
  binding: confidence drops to 0.5 and the result is `Ambiguous`, not `Contradictory`.

## Capabilities

### New Capabilities

<!-- None -->

### Modified Capabilities

- `grounding-outcome`: adds the boundary requirement.
- `argument-assignment`: narrows the repeated-constant rule to name matches and adds the
  name-over-alias preference.

## Non-goals

- No API change. `GroundingStatus`, `GroundedAtom`, `BoundArgument`, `MatchKind` and
  `GroundingResult` keep the shape frozen for the consumer that just adapted to it.
- No change to `PREDICATE_KEYWORDS` or to the positional assignment heuristics.
- No change to the caller's alias list: leading-phrase aliases remain legitimate.

## Impact

- `src/grounders/rule.rs`, new `src/grounders/rule/identifiers.rs`.
- No new dependencies.
