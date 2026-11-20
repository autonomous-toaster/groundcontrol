# Design — grounding-identifier-boundaries

## The defect, precisely

`appears_in` and `find_match` decided that a constant occurs in the text with
`text.contains(&constant.name)`. For names that are prefixes of others this is wrong:

```
"T5.15 SHALL ... BEFORE T5.16 SHALL ...".contains("T5.1")  ==  true
```

Two consequences, and the second is why the report looked like a spec bug:

1. `T5.1` enters `matched_constants` although the text never names it.
2. It matches by **name**, not alias, in both the before-slice and the after-slice, so both
   slots bind `T5.1` with `MatchKind::Name` and `exact: true` — a confidence-0.95 *name*
   repetition, which is exactly the shape the distinctness rule exists to catch.

So the consumer's proposed remedy ("report Contradictory only when the repeated constant is
name-matched in both slots") would not have suppressed these findings — they already were
name matches. The containment test had to be fixed first. The remedy is still right for the
different class it addresses, and is adopted below.

## Decision 1 — Match on identifier boundaries, not substrings

A name occurrence counts only when the character before it and the character after it do not
continue an identifier (`is_alphanumeric() || '_'`).

The one subtlety is `.`: constant names contain it (`T5.1`), and prose ends sentences with it.
`"T5.1 SHALL run."` must match, `"T5.1.5"` must not. So a following `.` counts as a boundary
only when the character after the dot is not a digit.

Aliases get the same treatment, which also kills the older `serve` ⊂ `server` false positive
independently of the caller pruning its alias list.

## Decision 2 — A name match wins the slot

`bind_slot` runs two passes over the matched constants: a boundary-correct name match first,
then aliases. Previously the winner was whichever constant came first in signature order,
so an alias hit on `T5.1` could beat a literal `T5.15`. Now a constant the author wrote
always wins, and the consumer's request ("an alias-only constant must not displace a
name-matched one") holds structurally rather than by luck of ordering.

## Decision 3 — Contradictory means the author wrote it twice

`name_repeated_pair` finds a pair of slots holding the same constant where **both** matched by
name. Only that is `Contradictory`. An alias-driven repeat is our own weak binding, so
confidence drops to 0.5 and classification falls through to `Ambiguous` — the caller is sent
to "narrow this statement", not to "you named one task twice". The `diagnostic` rendering says
which of the two happened.

This keeps the frozen status set: no sixth variant was needed.

## Considered and not done: the inverse diagnostic

The consumer suggested reporting constants the text names but selection did not use — the
mirror of `absent_identifiers`, and the signal that would have surfaced this class directly.
It is a good idea and it needs one new field on `GroundingResult`, which is a shape change at
the exact moment the caller is mid-adaptation. Measured against the boundary fix, the field
would have been empty in all 9 corpus cases (the named tasks are bound after the fix), so it
is a guard rather than a load-bearing diagnostic. Deferred, not rejected.

## Verification

`src/grounders/rule/diagnostics_tests.rs`: prefix signatures T5.1/T5.10/T5.12/T5.15/T5.16 —
`T5.15 ... BEFORE T5.16` binds both named tasks and is `Grounded`; the same with a trailing
sentence dot still matches; an alias on the prefix constant does not win a slot; an alias-only
double binding is `Ambiguous` with a diagnostic saying so; a name repetition is still
`Contradictory`. End to end on a three-phase corpus fixture: exit 0, no findings.
