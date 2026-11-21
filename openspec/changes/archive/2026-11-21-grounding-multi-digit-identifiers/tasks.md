## 1. Fix

- [x] 1.1 Drop the per-pattern length cap derived from the longest declared constant name
- [x] 1.2 Require `boundary_before` / `boundary_after` for an absent-identifier candidate
- [x] 1.3 Keep the fixed sanity cap (`MAX_IDENTIFIER_LEN`) as the only length bound

## 2. Tests (kata 08p0, cdzv)

- [x] 2.1 Repro A: `T2.10` reported verbatim, not truncated to `T2.1`
- [x] 2.2 Repro B: foreign `T1.10` reported instead of swallowed
- [x] 2.3 Multi-digit on both sides (`T10.123`)
- [x] 2.4 Every reported identifier occurs verbatim in the input
- [x] 2.5 Sentence-final dot excluded
- [x] 2.6 B2: `T1.1` not bound when the text cites only `T1.10`, with both declared
- [x] 2.7 `T1.1` and `T1.10` never confused in either direction
- [x] 2.8 `just ci` green

## 3. Reported, not fixed

- [x] 3.1 Single-slot predicates (`ALWAYS`) bind the first named constant in signature order, not
      text order. Raised with the client as an observation; outside B1/B2 and it would move
      corpus results.
