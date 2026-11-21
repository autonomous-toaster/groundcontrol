## 1. Data

- [x] 1.1 `GroundingResult.unused_identifiers` with serde-default and skip-if-empty
- [x] 1.2 `GroundedInput.unused_identifiers` threaded through `check_single`

## 2. Computation

- [x] 2.1 `unused_identifiers(sig, nl, candidates)` — named minus bound, empty without candidates
- [x] 2.2 Boundary-aware, so prefix collisions are not reported as unbound

## 3. Output

- [x] 3.1 Directive for each unbound identifier; suggestion does not ask for more task IDs
- [x] 3.2 Human output prints `unbound although named: ...` under a grounded line
- [x] 3.3 The check prints notes but exits 0 unless there is a real finding

## 4. Verification

- [x] 4.1 Shadowed named task, fully-bound statement, no-candidate, prefix collision
- [x] 4.2 JSON round-trip of the field
- [x] 4.3 End to end: grounded corpus with one unbound reference prints and exits 0
- [x] 4.4 `just ci` green
