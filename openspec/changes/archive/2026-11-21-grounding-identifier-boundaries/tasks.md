## 1. Identifier boundaries

- [x] 1.1 `contains_name` — accept a name only where neither neighbour continues an identifier
- [x] 1.2 `boundary_before` / `boundary_after`; a `.` is a boundary unless a digit follows
- [x] 1.3 Apply the boundary rule to aliases in `appears_in`
- [x] 1.4 Split identifier handling into `src/grounders/rule/identifiers.rs` (500-line rule)

## 2. Slot filling

- [x] 2.1 `bind_slot` — two passes over the matched constants, name first, then alias
- [x] 2.2 Remove the per-constant `find_match` so ordering can no longer pick the winner

## 3. Contradiction scope

- [x] 3.1 `name_repeated_pair` — both slots must match by name
- [x] 3.2 `classify` uses the name pair; alias repeats fall through to `Ambiguous` at 0.5
- [x] 3.3 `diagnostic_for` states which of the two repeats happened

## 4. Verification

- [x] 4.1 Regression tests for the reported corpus shape (T5.1 / T5.10 / T5.12 / T5.15 / T5.16)
- [x] 4.2 Trailing sentence dot, dotted continuation, alias boundary, name-beats-alias
- [x] 4.3 Alias double binding is `Ambiguous`; name double binding stays `Contradictory`
- [x] 4.4 End to end on a three-phase fixture: exit 0, no findings
- [x] 4.5 `just ci` green

## 5. Deliberately deferred

- [ ] 5.1 Inverse diagnostic (constants named by the text but not bound) — needs a new
      `GroundingResult` field, deferred to avoid a second shape change while the consumer
      adapts. See design.md.
