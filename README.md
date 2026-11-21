# groundcontrol — Ground Natural Language Into System Signatures

**groundcontrol** checks whether natural-language requirement statements can be grounded to a plan's task vocabulary. It is a companion to [veriplan](https://github.com/autonomous-toaster/veriplan): veriplan verifies plan *structure* (task ordering, temporal constraints), while groundcontrol verifies that the *language* used in requirements actually refers to real tasks.

Based on the paper [*GinSign: Grounding Natural Language Into System Signatures for Temporal Logic Translation*](https://arxiv.org/abs/2512.16770) (arXiv:2512.16770).

---

## The Problem

veriplan reads OpenSpec plans and checks whether temporal constraints hold. But it relies on regex to extract task IDs from requirement text:

```
"T2.1 SHALL complete BEFORE T3.1 SHALL run"  →  regex finds T2.1, T3.1  ✓
"The migration must finish before testing starts"  →  regex finds nothing  ✗
```

When a requirement uses descriptions instead of explicit task IDs, veriplan marks it `PatternUngrounded` and blocks. The LLM (pi) gets a generic error but no guidance on *what* to fix.

groundcontrol solves this by formalizing the plan's vocabulary as a **system signature** and checking whether each NL statement can be grounded to it.

---

## How It Works

### 1. Build a Signature from tasks.md

```
tasks.md:
  ## 1. Setup
  - [ ] 1.1 Create project
  - [ ] 1.2 Add dependencies

  → Signature with 2 constants:
      T1.1  aliases: ["create project", "1.1"]
      T1.2  aliases: ["add dependencies", "1.2"]
```

Each task becomes a constant with aliases derived from its description. Six predicates are always included: `BEFORE`, `AFTER`, `CONCURRENTLY`, `IF_THEN`, `ALWAYS`, `AT_MOST_ONE`.

### 2. Ground NL Statements Against the Signature

```
Input:  "The migration must finish before testing starts"
        ↓
Predicate matching:  "before" → BEFORE
Argument extraction: "migration" → alias match → T2.1
                     "testing"   → alias match → T2.2
Positional:          "X before Y" → earlier=T2.1, later=T2.2
        ↓
Output: BEFORE(T2.1, T2.2) @ 0.90, exact=false   status=grounded
```

Names are matched on identifier boundaries, so `T5.1` does not match inside `T5.15`.
A slot is filled by a name match whenever the text names a task; an alias only fills it
when no name matches. Every argument records which of the two it was.

#### The five statuses

| Status | Meaning | Who has to act |
|---|---|---|
| `grounded` | A candidate with confidence ≥ 0.8 | Nobody |
| `incomplete` | The predicate matched, the arguments did not resolve | Author, but see below |
| `ambiguous` | Candidates exist, all below 0.8, nothing else explains why | Author: narrow the statement |
| `contradictory` | One task named twice in slots that require different tasks | Author: name a second task |
| `ungroundable` | No predicate keyword and no task reference | Author: use a temporal keyword |

**Two distinctions a caller must implement.** They are the reason this is not a single
confidence number:

1. **An absent identifier is not an arity shortfall.** When the text cites a
   task-shaped token the signature does not declare — `T2.6` in a change that only defines
   `T1.1` — the result is `incomplete` *and* `absent_identifiers` names `T2.6`. The statement is
   not under-specified: the author named a task, and this change does not own it. Telling them
   to "add a task ID" is wrong, because they already did. Report that the cited task is
   foreign to the change.
2. **A contradictory slot is not a low-confidence match.** `T1.1 ... BEFORE T1.1` grounds both
   slots on one task. It is `contradictory`, never `grounded`, and never merely "ambiguous".
   The statement is well-formed and meaningless — it asserts a task precedes itself. Sending
   the author to "narrow the statement" wastes their time; they need to name a second task.

For a contradictory verdict both slots must have matched by **name**. If one task wins both
slots through an alias, that is the grounder's weakness rather than a defect in the statement:
it is reported as `ambiguous` at confidence 0.5 instead.

#### The result types

```rust
GroundingResult {
    candidates: Vec<GroundedAtom>,
    status: GroundingStatus,
    absent_identifiers: Vec<String>,  // cited, but not declared here
    unused_identifiers: Vec<String>,  // named in the text, bound by nothing
}

GroundedAtom {
    predicate: String,
    arguments: Vec<BoundArgument>,   // in slot order
    exact: bool,                     // true only if every argument matched by name
    diagnostic: String,              // rendering of the fields above, not a source of truth
    confidence: f64,                 // structural, see below
}

BoundArgument { constant: String, matched: MatchKind }
MatchKind::Name | MatchKind::Alias(String)   // which literal in the text matched it
```

`unused_identifiers` is the mirror of `absent_identifiers`: a task the text names explicitly
that no candidate bound. It means the grounder dropped something the author wrote. It does not
affect `status` — whether an unbound reference is a failure is the caller's policy.

#### What `confidence` means

`confidence` is *structural*, not correctness. It says how the slots were filled, and nothing
about whether the resulting constraint is true:

| Condition | Value |
|---|---|
| every slot filled by name | 0.95 |
| every slot filled, at least one by alias | 0.90 |
| every slot filled, predicate not in the signature | 0.80 |
| one constant bound to two slots | 0.50 |
| argument list short, or longer than the signature | 0.50 |

Read `exact` and the `MatchKind` of each argument for identity, and compare `arguments.len()`
with the predicate's arity for shape. Do not treat `0.95` as "grounded correctly".

---

### 3. Produce Directives

Each non-grounded status yields a directive naming the actual problem, derived from the fields
above rather than from the message text:

```
⚠ specs/ports/spec.md#L3 [incomplete]
   "T2.6 SHALL publish the port BEFORE T1.1 renames the noun"
   → BEFORE(T1.1) @ 0.50
   'T2.6' is not a task of this signature.
   Suggestion: Add 'T2.6' to tasks.md, or reference a task this change declares.

✗ specs/ports/spec.md#L12 [contradictory]
   "T1.1 SHALL write the header BEFORE T1.1 SHALL seal it"
   → BEFORE(T1.1, T1.1) @ 0.50
   BEFORE requires two different tasks, but 'T1.1' fills both 'earlier' and 'later'.
   Suggestion: Name a second task: 'T1.1' cannot be BEFORE itself.
```

An alias match still grounds — it is scored 0.90 rather than 0.95 and marked `exact: false` —
so it appears in the data rather than in the human listing. Read it from `MatchKind`:

```json
{ "predicate": "BEFORE",
  "arguments": [
    { "constant": "T2.1", "matched": { "kind": "alias", "alias": "migration" } },
    { "constant": "T2.2", "matched": { "kind": "alias", "alias": "testing" } } ],
  "exact": false,
  "diagnostic": "matched by alias: T2.1 as 'migration', T2.2 as 'testing'",
  "confidence": 0.9 }
```

Use that to weight a binding that rests on a shared word rather than an explicit task ID. A
caller that treats `exact: false` as a first-class signal gets this for free; one that reads
only `confidence` sees 0.90 and cannot tell it from a name match.

---

## Relationship to veriplan

```
┌──────────────────────────────────────────────────────────────┐
│  pi (LLM coding assistant)                                   │
│                                                               │
│  writes plan → groundcontrol check → veriplan check → loop or done
│                                                               │
│  groundcontrol:  "this spec cites a task this change does not own" │
│  veriplan:   "these temporal constraints are violated"         │
└──────────────────────────────────────────────────────────────┘
```

| | veriplan | groundcontrol |
|---|---|---|
| **What it checks** | Plan structure (task ordering, phases, LTL) | NL phrasing (can specs be grounded to tasks?) |
| **Input** | OpenSpec markdown (tasks.md, specs/) | Same OpenSpec markdown |
| **Output** | Violations + counterexamples | Grounded atoms + structured directives |
| **When it runs** | After specs are well-grounded | Before veriplan, as a pre-check |
| **Exit codes** | 0 = valid, 1 = violations | 0 = every statement grounded, 1 = issues found |

The two tools are independent — groundcontrol has no dependency on veriplan. They share the same input format (OpenSpec) but have separate parsers and output different things.

---

## Usage

```bash
# Check grounding quality (default: human-readable)
groundcontrol check my-change

# JSON output for machine consumption
groundcontrol check my-change --format json

# Emit the signature JSON
groundcontrol signature my-change

# Check by path instead of change name
groundcontrol check ./openspec/changes/my-change
```

### Exit Codes

| Code | Meaning | Action |
|---|---|---|
| 0 | Every statement grounded | Proceed to veriplan check |
| 1 | At least one statement `incomplete`, `ambiguous`, `contradictory` or `ungroundable` | Fix the cited statements, re-run |
| 2 | Error (missing file, parse error) | Check input |

A statement may also be grounded yet leave a named task unbound
(`unused_identifiers`). That is a grounding weakness, not a defect in the statement, so it is
printed as a note and still exits 0.

### Output Format

On success (exit 0): silent, unless a grounded statement left a named task unbound.

On error (exit 1): successes first (quiet one-liners), then the problems with full detail. Each
problem line carries its status, so the failure modes are never confused:

```
✓ specs/ordering/spec.md#L5 → BEFORE(T2.1, T2.2)
✓ specs/safety/spec.md#L3 → ALWAYS(T6.1)
   unbound although named: T1.5

✗ specs/safety/spec.md#L7 [ungroundable]
   "The rollback SHALL always be available"
   Could not ground: no task reference found
   Close matches: 'T6.1' (alias: 'rollback'), 'T1.5' (alias: 'rollback trigger')
   Suggestion: Use 'T6.1 SHALL ALWAYS be available' or add '(T6.1)' after 'rollback'.

Summary: 3 total | 2 grounded | 0 incomplete | 0 ambiguous | 0 contradictory | 1 ungroundable
```

`--format json` emits `GroundingResult` as serialised, including `absent_identifiers` and
`unused_identifiers`, so a caller can branch on the data instead of parsing messages.

---

## How It Differs From the Paper

[GinSign](https://arxiv.org/abs/2512.16770) uses a BERT-based classifier for hierarchical predicate-then-argument grounding. groundcontrol uses a **rule-based grounder** (keyword matching + positional heuristics) with no ML dependency. This is sufficient for structured OpenSpec specs that use explicit task IDs and temporal keywords. A learned grounder can be added later via the `Grounder` trait.

---

## Project Structure

```
src/
├── main.rs              # CLI, check orchestration, human-readable output
├── lib.rs               # library facade: types, parse, grounders
├── types.rs             # Signature, GroundingResult, GroundedAtom, Grounder trait
├── directives.rs        # directives generated from a grounding result
├── specs.rs             # spec.md requirement extraction
├── init.rs              # `groundcontrol init` (openspec/config.yaml)
├── parse/
│   ├── mod.rs           # tasks.md parser (tree-sitter)
│   └── helpers.rs       # Tree-sitter helpers
└── grounders/
    ├── mod.rs
    └── rule.rs          # Rule-based grounder (6 predicate types)
        ├── identifiers.rs  # name shapes, boundaries, absent identifiers
        └── tests.rs        # + diagnostics_tests.rs

openspec/
├── specs/               # published capabilities (the contract)
└── changes/archive/     # completed changes, kept for history
```
