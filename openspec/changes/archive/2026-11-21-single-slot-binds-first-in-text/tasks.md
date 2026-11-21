## 1. Implementation

- [x] 1.1 `name_index` — boundary-aware first occurrence offset (`contains_name` now delegates)
- [x] 1.2 `bind_first_in_text` — earliest mention wins, signature order as tie-break
- [x] 1.3 ALWAYS binds through `bind_slot`, so `MatchKind` and alias suppression are honoured

## 2. Tests (kata 382j, gw3m)

- [x] 2.1 Both orderings of the same pair bind the text first and name the later mention unused
- [x] 2.2 One named task: unchanged
- [x] 2.3 Alias-only single-slot match reports `Alias`, `exact: false`, 0.9
- [x] 2.4 Updated the undeclared-predicate test, which had been asserting the fabricated 0.8
      name match that `take(1)` produced
- [x] 2.5 `just ci` green

## 3. Measurement

- [x] 3.1 Grounded veriplan's 24 spec files (429 statements) against its active change's tasks
      with the old tag and the new tree, and diffed: 9 statements change
- [x] 3.2 Reported the decomposition to the client: 4 intended binding moves, 2 provenance
      corrections, 3 alias-suppression corrections (kata gw3m)
