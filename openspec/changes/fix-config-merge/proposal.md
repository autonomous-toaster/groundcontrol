## Why

`groundcontrol init` (and `veriplan init`) append raw YAML text to `openspec/config.yaml` instead of merging into the existing YAML structure. This creates duplicate top-level keys (`context:`, `rules:`) — the second occurrence silently overwrites the first in any YAML parser. When both tools run on the same project, the first tool's context and rules are discarded. Additionally, the `context` field cites the tool name ("groundcontrol checks that...") rather than providing pure guidance to the coding agent.

## What Changes

- **groundcontrol**: Replace text-append `merge_config()` with YAML-aware merge using `serde_yaml`
- **veriplan**: Same fix — replace text-append `merge_config()` with YAML-aware merge
- **Context text**: Remove tool self-references from `context:` values — only keep the nudges
- **Marker system**: Keep `# groundcontrol init` / `# veriplan init` YAML comments as idempotency markers, but merge the actual data under them

## Capabilities

### New Capabilities

- `yaml-merge`: YAML-aware merging of `context` (append text with separator) and `rules` (deduplicate list items per artifact type) into `openspec/config.yaml`

### Modified Capabilities

*(none — no existing specs)*

## Impact

- **groundcontrol**: `src/main.rs` — `merge_config()` function, `BOOTSTRAP_CONFIG` and `BOOTSTRAP_SUFFIX` constants
- **veriplan**: `src/main.rs` — `merge_config()` function, `BOOTSTRAP_SUFFIX` constant
- **Dependency**: Add `serde_yaml` to both `Cargo.toml` files
- **Config files**: Existing `openspec/config.yaml` files with duplicate keys will be normalized on next `init` run
