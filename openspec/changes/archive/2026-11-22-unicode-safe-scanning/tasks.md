## 1. Char index vs byte offset (kata 9kvc)

- [x] 1.1 Build a character-index → byte-offset table in `extract_absent_identifiers`
- [x] 1.2 Convert before every `boundary_before` / `boundary_after` call

## 2. Lowercased-copy offsets

- [x] 2.1 `find_ci` — case-insensitive search returning offsets in the original string
- [x] 2.2 `assign_arguments` uses `find_ci` for BEFORE / AFTER / IF_THEN split points
- [x] 2.3 `bind_first_in_text` uses `find_ci` for alias positions, so name and alias offsets
      are comparable
- [x] 2.4 `assign_arguments` no longer takes a `lower` parameter

## 3. Tests

- [x] 3.1 The reported arrow repro, asserting a verdict and `absent_identifiers`
- [x] 3.2 Em dash between tasks, accented prose
- [x] 3.3 Lowercase-expansion case with the keyword at the end (the second path)
- [x] 3.4 Sweep: 6 paddings × 6 statement shapes, asserting no panic
- [x] 3.5 Correctness, not just absence of panic: bindings preserved under Unicode prose
- [x] 3.6 Fixture gains both Unicode cases
- [x] 3.7 `just ci` green

## 4. Evidence

- [x] 4.1 The `v0.2.3` tag panics on the reported statement; this tree returns
      `incomplete` with `absent_identifiers = ["H1.1"]`
