## 1. Groundcontrol: YAML-aware merge

- [x] 1.1 Add `serde_yaml` dependency to `Cargo.toml`
- [x] 1.2 Replace `merge_config()` text-append logic with YAML parse/merge/serialize using `serde_yaml`
- [x] 1.3 Strip tool self-references from `BOOTSTRAP_CONFIG` and `BOOTSTRAP_SUFFIX` context values
- [x] 1.4 Update tests for `merge_config` to cover: fresh file, existing file with context, existing file with rules, idempotent re-run, cross-tool preservation
- [x] 1.5 Run `cargo test` and verify all tests pass

## 2. Veriplan: YAML-aware merge

- [x] 2.1 Add `serde_yaml` dependency to `Cargo.toml`
- [x] 2.2 Replace `merge_config()` text-append logic with YAML parse/merge/serialize using `serde_yaml`
- [x] 2.3 Strip tool self-references from `BOOTSTRAP_SUFFIX` context value
- [x] 2.4 Update tests for `merge_config` to cover: fresh file, existing file with context, existing file with rules, idempotent re-run, cross-tool preservation
- [x] 2.5 Run `cargo test` and verify all tests pass

## 3. Integration verification

- [x] 3.1 Run `groundcontrol init` on a project with existing veriplan-initialized config — verify both tools' content is present
- [x] 3.2 Run `veriplan init` on a project with existing groundcontrol-initialized config — verify both tools' content is present
- [x] 3.3 Run both `init` commands multiple times — verify no duplicate entries accumulate
- [x] 3.4 Verify `context:` values contain no tool name references
