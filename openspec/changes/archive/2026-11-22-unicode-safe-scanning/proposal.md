## Why

A spec containing one non-ASCII character aborts the process:

```
T1.1 SHALL forward the upstream status during H2→H1.1 transcoding.
thread 'main' panicked at src/grounders/rule/identifiers.rs:169:15:
end byte index 49 is not a char boundary; it is inside '→' (bytes 48..51)
```

A panic is not a verdict. The consumer cannot turn it into a finding or an unverified result:
the run dies with exit 101 and no output. It is a regression against v0.1.0, which reads the same
file and reports ordinary findings.

Two paths mixed index spaces with the string they sliced:

1. `extract_absent_identifiers` walks in **character** indices (it builds a `Vec<char>`) and
   passed them to `boundary_before`/`boundary_after`, which slice by **byte** offset. Any
   multi-byte character before the position makes the two diverge.
2. `assign_arguments` searched a **lowercased copy** for the keyword and used that offset to
   slice the original. Lowercasing can change byte length (U+0130 İ, 2 bytes, lowercases to
   i̇, 3 bytes), so the offset overruns the original.

Both are the same mistake: an index from string A used to slice string B.

## What Changes

- The scanner keeps a character-index → byte-offset table and converts before every boundary
  test.
- Keyword search is case-insensitive **in place** (`find_ci`), so every returned offset belongs
  to the string being sliced. No lowercased copy is used for positions anywhere.
- Tests: the reported repro, em dash, accented text, the lowercase-expansion case, and a sweep
  across six multi-byte paddings and six statement shapes.
- The published fixture gains both Unicode cases, so a consumer diffing it sees the coverage.

## Capabilities

### New Capabilities

<!-- None -->

### Modified Capabilities

- `grounding-outcome`: states that scanning is Unicode-safe and total.

## Non-goals

- No change to any status, field, type, or to which constant is bound in an ASCII statement.
- No attempt to make non-ASCII text *match* better: a non-ASCII alias still matches only where
  ASCII case-insensitivity applies. The requirement is a verdict, not a panic.

## Impact

- `src/grounders/rule.rs`, `src/grounders/rule/identifiers.rs`, tests, fixture.
