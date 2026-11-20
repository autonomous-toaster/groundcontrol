## 1. Data model (`src/types.rs`)

- [x] 1.1 Add `MatchKind { Name, Alias(String) }` and `BoundArgument { constant, matched }` with serde derives
- [x] 1.2 Change `GroundedAtom::arguments` to `Vec<BoundArgument>`; add `exact: bool`; document `confidence` as structural
- [x] 1.3 Add `GroundingStatus::{Incomplete, Contradictory}` with stable lower-case serde names
- [x] 1.4 Add `GroundingResult::absent_identifiers` and a `GroundedAtom::names()` helper
- [x] 1.5 `CheckSummary` gains `incomplete` / `contradictory` buckets (serde-default for old JSON)
- [x] 1.6 Unit tests: status serde strings, `exact` semantics, result JSON round-trip preserving absent identifiers

## 2. Absent-identifier extraction (`src/grounders/rule.rs`)

- [x] 2.1 `name_shape` — split a constant name into maximal same-character-class runs (alpha / digit / other)
- [x] 2.2 `extract_absent_identifiers` — infer one pattern per constant type from its longest constant name, scan the text, report unknown tokens of matching shape
- [x] 2.3 Reject shapes with a single run, hypotheses longer than 12 chars, or longer than the type's longest constant name; require an alphabetic first run
- [x] 2.4 Unit tests: `T2.6` absent vs `T1.1` present; absent id with no temporal keyword; no inferable shape; `SHALL` not reported; longest-match keeps punctuation out

## 3. Argument provenance (`src/grounders/rule.rs`)

- [x] 3.1 `find_match` returns `Option<BoundArgument>` (name match preferred over alias)
- [x] 3.2 Thread `BoundArgument` through `assign_arguments`
- [x] 3.3 Populate `exact` and the 0.95 / 0.9 / 0.8 confidence ladder
- [x] 3.4 Suppress alias matching when absent identifiers exist — an alias may describe the missing constant, and guessing produces false contradictions

## 4. Slot distinctness (`src/grounders/rule.rs`)

- [x] 4.1 `distinguishes_slots(pred_def)` — more than one arg slot with pairwise distinct names
- [x] 4.2 Mark a candidate contradictory when a distinguished predicate receives a repeated constant
- [x] 4.3 Unit tests: BEFORE / AFTER / IF_THEN with a repeated task; distinct tasks still ground; `ALWAYS` and undeclared predicates unaffected; repeated slot names are reflexive; slack slots do not contradict

## 5. Status classification (`src/grounders/rule.rs`)

- [x] 5.1 Decision order: contradictory → ≥0.8 grounded → incomplete → ambiguous → ungroundable
- [x] 5.2 `Incomplete` when absent identifiers exist, or a candidate is short against a declared arity ≥ 2 predicate
- [x] 5.3 Unit tests: the three handoff reproducers end to end; existing tests migrated to `BoundArgument`

## 6. CLI output (`src/directives.rs`, `src/main.rs`)

- [x] 6.1 Directive for `absent_identifiers`: "'X' is not a task of this signature"
- [x] 6.2 Directive for `Contradictory` naming the repeated task and both slot names
- [x] 6.3 Alias directive reads `MatchKind::Alias` instead of re-deriving from the text
- [x] 6.4 `Incomplete` / `Contradictory` counted, exit non-zero, printed in human and JSON output
- [x] 6.5 Unit tests: absent-id directive suppresses the generic "add task IDs" text; alias directive; contradictory directive

## 7. Structure and closing

- [x] 7.1 Split `src/main.rs` (500-line rule): modules `directives`, `specs`, `init`; tests moved next to their module
- [x] 7.2 Fix pre-existing lint blockers (`desc` unused in `src/parse/mod.rs` tests, dead `ground_batch`) so `just ci` can pass
- [x] 7.3 `just ci` green (check, lint -Dwarnings, test 47+89, fmt, file sizes)
- [x] 7.4 Manual end-to-end run on a hand-built change dir reproducing D1 and D2, human and JSON
- [x] 7.5 Kata issue sryj closed with the verification commands