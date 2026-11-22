## Why

`v0.2.4` fixed a panic that aborted the process on a spec containing an arrow. The fix was
correct, but nothing stopped the next one: the crate used `.unwrap()`, direct indexing and
string slicing in shipped code, and the only thing that noticed was a consumer's wide-corpus
run.

A panic is not a verdict. It exits 101 with no output, so a caller cannot record a finding, a
warning, or even an unverified result. For a tool whose entire product is the diagnosis, an
aborted process is the worst available failure mode - worse than a wrong answer, because a
wrong answer can at least be argued with.

## What Changes

- `Cargo.toml` declares `[lints.clippy]` denials for `panic`, `unwrap_used`, `expect_used`,
  `indexing_slicing`, `string_slice`, `todo`, `unimplemented`, `unreachable`, plus
  `[lints.rust] unsafe_code = "forbid"`. The build refuses the constructs that panic.
- Every shipped site is rewritten as a total operation: `get()` with an explicit fallback,
  `first()`/`get(i)` instead of `[0]`/`[i]`, `if let [only] = slice` instead of length check plus
  index, and `unwrap_or_default()` instead of `unwrap()` in the config writer.
- `parse::helpers::node_text` returns `Option<&str>` instead of `Result<_, Utf8Error>`: a node
  range that is out of bounds or splits a character is not an encoding error to report, it is a
  range that has no text. Private module, so no API change.
- Test modules opt out with one explicit inner attribute each. A test asserts the opt-out never
  appears outside test code.

## Capabilities

### New Capabilities

- `panic-freedom`: the crate never panics on any input, and the build enforces it.

### Modified Capabilities

<!-- None: no observable behaviour changes -->

## Non-goals

- No change to any status, field, type, or matching behaviour. The fixture in
  `tests/fixtures/grounding_cases.json` is unchanged, which is the evidence.
- No `panic = "abort"` profile setting. That does not prevent panics, it removes unwinding, and
  it would take `catch_unwind` away from consumers who currently have it.
- No blanket `#![allow]` anywhere in non-test code.

## Impact

- `Cargo.toml`, `src/grounders/rule.rs`, `src/grounders/rule/identifiers.rs`, `src/parse/mod.rs`,
  `src/parse/helpers.rs`, `src/specs.rs`, `src/directives.rs`, `src/init.rs`, `src/main.rs`,
  test modules, plus `tests/panic_policy.rs`.
