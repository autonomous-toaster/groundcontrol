# Design — forbid-panics

## Why lints rather than an attribute

The obvious enforcement is `#![deny(clippy::unwrap_used)]` at the crate root. It sits in the
same file as the code it governs, so the first person who needs an exception deletes the
attribute and the policy dies quietly in a diff nobody reads.

Putting the policy in `Cargo.toml` separates it from the source and makes weakening it a
change to build configuration - a file reviewers already look at for dependency changes. It also
scopes naturally: the library and the binary share the table, and a future workspace member
inherits it.

`[lints.rust] unsafe_code = "forbid"` is `forbid` because nothing in this crate needs `unsafe`
and no test ever should. The clippy lints are `deny` rather than `forbid` because test modules
need to unwind: a test that asserts a result is easier to read as `unwrap()` than as a chain of
`let ... else { panic!() }`, and `assert!` itself panics. `forbid` would make the test code
worse to buy nothing, since the opt-out is itself tested (below).

## The trade-off this accepts

A `deny` lint can be overridden by an `#[allow]` anywhere in the crate, including in shipped
code. Two things make that acceptable:

1. Every opt-out in the tree is a single inner attribute in a test module, and
   `tests/panic_policy.rs` walks `src/` and fails if one appears in a file or module that is not
   test-only.
2. The same test asserts each lint is still declared `deny`, so silencing the policy fails CI
   instead of passing silently.

Without those two tests the lint set would be an aspiration. With them it is a rule.

## Rewriting the call sites

The lints found 32 shipped sites. Each became total rather than suppressed:

| Pattern | Replacement | Why it is total |
|---|---|---|
| `slice[i]` | `slice.get(i)` | `None` on out-of-range |
| `slice[..n]` / `slice[n..]` | `slice.get(..n)` / `slice.get(n..)` | `None` at a non-char boundary |
| `v[0]` | `v.first()` | `None` when empty |
| `len() == 1` then `v[0]` | `if let [only] = v` | pattern and check in one step |
| `s.find(x)` then `s[..pos]` | `find` then `get(..pos)` | the split cannot disagree with the find |
| `.unwrap()` | `unwrap_or_default()` / `?` | no panic path |

`unwrap_or` is not `unwrap`: the lint flags the panic-capable call, which is the one that can
abort. Defaulting is a decision the code states; panicking is one it hides.

## node_text: Result to Option

`node_text` returned `Result<&str, Utf8Error>` but its out-of-bounds case had no `Utf8Error` to
produce, which is how it ended up calling `unwrap_err()` on a valid slice to fabricate one -
caught by the new `unwrap_err` denial, satisfyingly. The honest signature is `Option<&str>`:
there is no text there. `helpers` is a private module, so this is not an API change; every
caller already wrote `if let Ok(text)` and now writes `if let Some(text)`.

## Verification

- `cargo clippy --workspace --all-targets` reports zero diagnostics in `src/` and `tests/`.
- The lints were proven to bite: adding a function with `nl[..1].chars().next().unwrap()` to
  `identifiers.rs` fails the build with both `unwrap_used` and `string_slice`.
- The policy test was proven to bite: setting one lint to `allow` fails it with the exact key.
- `tests/fixtures/grounding_cases.json` is byte-identical, so no behaviour moved.
