## 1. Groundcontrol: YAML-aware merge

- [ ] 1.1 Add `serde_yaml` dependency to `Cargo.toml`
- [ ] 1.2 Replace `merge_config()` text-append logic with YAML parse/merge/serialize using `serde_yaml`
- [ ] 1.3 Strip tool self-references from `BOOTSTRAP_CONFIG` and `BOOTSTRAP_SUFFIX` context values
- [ ] 1.4 Update tests for `merge_config` to cover: fresh file, existing file with context, existing file with rules, idempotent re-run, cross-tool preservation
- [ ] 1.5 Run `cargo test` and verify all tests pass

## 2. Veriplan: YAML-aware merge

- [ ] 2.1 Add `serde_yaml` dependency to `Cargo.toml`
- [ ] 2.2 Replace `merge_config()` text-append logic with YAML parse/merge/serialize using `serde_yaml`
- [ ] 2.3 Strip tool self-references from `BOOTSTRAP_SUFFIX` context value
- [ ] 2.4 Update tests for `merge_config` to cover: fresh file, existing file with context, existing file with rules, idempotent re-run, cross-tool preservation
- [ ] 2.5 Run `cargo test` and verify all tests pass

## 3. Integration verification

- [ ] 3.1 Run `groundcontrol init` on a project with existing veriplan-initialized config — verify both tools' content is present
- [ ] 3.2 Run `veriplan init` on a project with existing groundcontrol-initialized config — verify both tools' content is present
- [ ] 3.3 Run both `init` commands multiple times — verify no duplicate entries accumulate
- [ ] 3.4 Verify `context:` values contain no tool name references
