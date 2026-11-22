## 1. Policy

- [x] 1.1 `[lints.clippy]` denials for the eight panic-capable lints
- [x] 1.2 `[lints.rust] unsafe_code = "forbid"`
- [x] 1.3 Comment in Cargo.toml explaining deny-vs-forbid and why it lives here

## 2. Shipped call sites (32 found by the lints)

- [x] 2.1 `identifiers.rs` — char-index spans and byte-offset slices via `get`, offsets table
- [x] 2.2 `rule.rs` — keyword splits via `get`, argument lookups via `first`/`get`
- [x] 2.3 `parse/mod.rs` — alias prefixes, task-id scanning, byte reads all total
- [x] 2.4 `parse/helpers.rs` — `node_text` returns `Option`, no fabricated `Utf8Error`
- [x] 2.5 `specs.rs` — line and byte access via `get`
- [x] 2.6 `directives.rs`, `init.rs`, `main.rs` — first/get, `unwrap_or_default`, slice pattern

## 3. Test opt-outs

- [x] 3.1 One inner allow per test module, five file modules and five inline modules

## 4. Guards

- [x] 4.1 `tests/panic_policy.rs` asserts every lint is still denied
- [x] 4.2 Same file asserts the opt-out never appears outside test code
- [x] 4.3 Proven: adding a panic-capable function fails the build
- [x] 4.4 Proven: setting one lint to `allow` fails the policy test

## 5. Verification

- [x] 5.1 `cargo clippy --workspace --all-targets` clean
- [x] 5.2 `just ci` green; 210 tests
- [x] 5.3 Fixture unchanged, so no behaviour moved
- [x] 5.4 Public API unchanged (`parse::helpers` is a private module)
