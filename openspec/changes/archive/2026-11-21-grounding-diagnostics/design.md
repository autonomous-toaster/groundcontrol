# Design — grounding-diagnostics

## Context

`RuleGrounder::ground` does keyword → constant-extraction → positional assignment → score.
The score has exactly two inputs: `args.len() == pred_def.arguments.len()` and
`!args.is_empty()`. Every diagnostic the caller needs is already available at that point —
the Signature, the predicate def, and the text — but is discarded.

Two defects follow from that, and one from `appears_in` being a `str::contains` test.

## Decision 1 — Absent identifiers are detected by type shape, not by a task-ID regex

**Problem.** We know the Signature and we know the text. A token like `T2.6` looks like a
constant of a declared type but no `ConstantDef` provides it. Today it vanishes; the
statement looks arity-short.

**Options.**

1. Hard-code a task-ID regex (`T\d+\.\d+`) and diff against constant names. Cheap, but
   bakes the caller's identifier syntax into a domain-agnostic grounder, and breaks when a
   plan uses another naming convention.
2. Ask the caller to pass the identifier grammar in. Correct, but it adds an input the caller
   must supply — real cost for veriplan, and it leaks the problem back to the caller.
3. **Derive the shape from the Signature's own constants.** A type's shape is what its
   constants' names look like: split each name into maximal runs of *same character class*
   (alphabetic / digit / other). `T1.1` → [alpha, digit, other, digit]. `T2.6` matches.
   `SHALL` does not. Split the text the same way and union hypotheses to infer boundaries.

**Chosen: 3.** No caller input, no hard-coded syntax, degrades safely: a Signature whose
constants have no usable shape reports no absent identifiers (today's behaviour). Guarded
against nonsense: hypotheses longer than 12 characters, or longer than the type's own longest
constant name, are not reported.

## Decision 2 — Provenance moves inside the atom, not beside it

**Problem.** `arguments: Vec<String>` cannot express "matched by name" versus "matched by
the incidental word `header` from T2.6's description". A parallel `Vec` in the result would
let the two fall out of sync.

**Chosen.** `arguments: Vec<BoundArgument>` where `BoundArgument { constant, matched }` and
`MatchKind { Name, Alias(String) }`. Position in the vector stays the slot position, so
callers that only want names map over it. `exact` on the atom is the conjunction of all
arguments matching by name — the boolean that replaces reading `confidence == 0.95` as
"grounded correctly".

## Decision 3 — Distinctness is derived from slot names, not from a predicate list

**Problem.** BEFORE / AFTER / CONCURRENTLY / AT_MOST_ONE / IF_THEN all require two *different*
things. A hard-coded predicate list would satisfy the handoff but would not survive a new
predicate in the Signature.

**Chosen.** A predicate *distinguishes its slots* when it has (a) more than one arg slot and
(b) distinct slot **names**. The Signature already declares `earlier`/`later`, `a`/`b`,
`trigger`/`consequent` — that is the author's own statement that the slots mean different
things. Consequence: `ALWAYS(target)` with one slot has nothing to distinguish, and a
hypothetical `P(x, x)` is legitimately reflexive and stays Grounded. No list to maintain.

A violation is a **contradiction**, not an ambiguity: the statement is well-formed (every
slot filled) and meaningless. Reporting it as Ambiguous would send the caller down the
"narrow my statement" path; it needs the "you named one task twice" path.

## Decision 3a — Alias matching is suppressed when a referenced constant is absent

Found while running the D1 reproducer end to end. With `T2.6` absent, the text
`T1.1 SHALL publish the port BEFORE T2.6 SHALL renames the noun` bound `T1.1` twice:
`T1.1` for `earlier`, and `T1.1` again for `later` via the alias `renames` — and the
distinctness rule then reported a **contradiction** for a statement that is merely
incomplete. A false contradiction is worse than the arity fallback it replaced.

**Rule.** When `absent_identifiers` is non-empty, only name matches bind arguments.
An alias hit is a weak signal, and the text has already named a constant-shaped token
this signature does not know — the alias plausibly describes *that* token. Names are
unambiguous and still bind, so a genuine contradiction sharing a statement with an
absent identifier is still caught (covered by a test).

## Decision 4 — Five statuses; the pre-existing threshold is preserved

`Grounded` keeps its public meaning: some candidate scores at least 0.8. That threshold is
load-bearing for existing callers, so it is not redefined. Decision order:

1. Any candidate with a distinctness violation → `Contradictory`.
2. Otherwise any candidate scoring at least 0.8 → `Grounded`.
3. Otherwise absent identifiers present, or a candidate that is short against a declared
   predicate — `Incomplete`.
4. Otherwise any candidate → `Ambiguous`.
5. Otherwise → `Ungroundable`.

Rule 3 is the D1 fix: a short argument list is only Ambiguous when nothing better explains
it. Absent identifiers, and a short formal declaration, both explain it better.

Scoring (documented on `GroundedAtom::confidence`):

| condition | confidence |
|---|---|
| every slot filled by name | 0.95 |
| every slot filled, at least one by alias | 0.9 |
| every slot filled, predicate undeclared in Signature | 0.8 |
| all slots filled by the same constant (contradiction) | 0.5 |
| argument list short, or extra arguments | 0.5 |
| no arguments | 0.0 |

`exact` and the `BoundArgument` provenance carry the identity/arity distinction the float used
to hide; `confidence` is explicitly *structural* confidence.

## Trade-offs accepted

- An alias hit scores 0.9 and still grounds. Alias matching is what makes descriptive task
  references work; we make it visible rather than disable it.
- The character-class splitter can report a token that is a coincidence of shape. Accepted:
  the field lists *candidates* for the caller to render, and it is strictly more information
  than today's silence.
- `Contradictory` is a new status for callers that exhaustively match `GroundingStatus`.
  Deliberate: a caller that does not know about it cannot silently treat a self-ordering
  constraint as sound.

## Module structure

`just ci` enforces a 500-line-per-source-file rule (550 hard limit). `src/main.rs` was already
1147 lines before this change, so the work was split into modules rather than growing it:

| file | contents |
|---|---|
| `src/main.rs` | CLI, `check_single`, output merging, human-readable printing |
| `src/directives.rs` | directive generation, close matches, string similarity (+ `directives/tests.rs`) |
| `src/specs.rs` | spec.md parsing (`parse_specs`, `extract_requirements`, `extract_shall`) |
| `src/init.rs` | `init` subcommand and config merging (+ `init/tests.rs`) |
| `src/grounders/rule.rs` | grounder (+ `rule/tests.rs`, `rule/diagnostics_tests.rs`) |

Two pre-existing `-Dwarnings` blockers (`let (id, desc)` with `desc` unused in
`src/parse/mod.rs` tests, and dead `Grounder::ground_batch`) were fixed; without them
`just ci` could not pass at all, before or after this change.

## Verification

`veriplan check` on this repo cannot be the acceptance signal: veriplan currently renders any
requirement containing a task id plus "before"/"after" as `[] ( true )`, so it is green
regardless. Acceptance is unit tests with hand-built signatures covering each row of the table
above plus the three handoff cases, and one manual end-to-end run:

```
⚠ cap/spec.md#L3 [incomplete]
   "T1.1 SHALL publish the port BEFORE T2.6 SHALL renames the noun."
   → BEFORE(T1.1) @ 0.50
   'T2.6' is not a task of this signature.
   Suggestion: Add 'T2.6' to tasks.md, or reference a task this change declares.
✗ cap/spec.md#L12 [contradictory]
   "T1.1 SHALL write the header BEFORE T1.1 SHALL seal it."
   → BEFORE(T1.1, T1.1) @ 0.50
   BEFORE requires two different tasks, but 'T1.1' fills both 'earlier' and 'later'.
```
