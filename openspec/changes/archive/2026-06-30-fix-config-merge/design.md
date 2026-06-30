## Context

`groundcontrol init` and `veriplan init` both write to `openspec/config.yaml` by appending raw YAML text. This creates duplicate top-level keys (`context:`, `rules:`) when both tools run on the same project. YAML parsers silently take the last value, so the first tool's configuration is discarded.

The current approach uses a text-based marker (`# groundcontrol init` / `# veriplan init`) to detect prior runs, but this only prevents re-appending the same tool's block — it doesn't prevent cross-tool key collisions.

## Goals / Non-Goals

**Goals:**

- `groundcontrol init` SHALL merge its context and rules into existing `openspec/config.yaml` without creating duplicate YAML keys
- `veriplan init` SHALL merge its context and rules into existing `openspec/config.yaml` without creating duplicate YAML keys
- The `context` field SHALL contain only coding-agent nudges, not tool self-references
- Existing config content (user-written context, rules from other tools) SHALL be preserved
- Running `init` multiple times SHALL be idempotent

**Non-Goals:**

- No changes to the config.yaml schema or how it's consumed by pi
- No changes to the `check` or `signature` commands
- No changes to veriplan's `update_gitignore` or other side effects

## Decisions

### Decision 1: Use `serde_yaml` for proper YAML merge

**Chosen**: Add `serde_yaml` dependency and parse/merge/serialize the config as structured YAML.

**Alternatives considered:**

- **Text-based with key awareness**: Parse lines to find `context:` and `rules:` positions, then insert content. Rejected because YAML block scalars (`|-`, `>`) have complex indentation rules that make line-based parsing fragile.
- **Separate config files per tool**: Each tool writes its own file (e.g., `config.groundcontrol.yaml`). Rejected because pi reads a single `config.yaml` — splitting would require changes to the pi integration.
- **Single merged constant**: Combine both tools' content into one `BOOTSTRAP_CONFIG`. Rejected because it couples the two tools and breaks if only one is installed.

### Decision 2: Merge strategy

**context**: Append new text to existing value with `\n\n` separator. The `context` field is a YAML block scalar shown to the AI — concatenation preserves all guidance.

**rules**: For each artifact type (specs, tasks, etc.), add new rule strings that aren't already present. Deduplication is by exact string match.

### Decision 3: Marker comments become informational only

The `# groundcontrol init` / `# veriplan init` YAML comments are kept as human-readable markers but are no longer used for idempotency. Idempotency is achieved by the merge logic itself: if a rule string already exists, it won't be added again.

### Decision 4: Context text stripped of tool self-references

The `context:` value SHALL NOT mention the tool name. It SHALL contain only the guidance that helps the coding agent write better artifacts.

## Risks / Trade-offs

- **[New dependency] `serde_yaml`** → Adds ~50KB to binary size. Mitigation: it's a mature, well-maintained crate already used in the Rust ecosystem.
- **[Existing configs with duplicate keys]** → Files created by the old text-append approach will have duplicate keys. On next `init`, the YAML parser will see only the last value. Mitigation: the merge logic normalizes on write — the first `init` after upgrade produces a clean single-key file.
- **[veriplan coupling]** → The fix must be applied to both tools. If only one is fixed, running the other will re-introduce duplicates. Mitigation: both are in the same monorepo workspace and can be fixed together.
