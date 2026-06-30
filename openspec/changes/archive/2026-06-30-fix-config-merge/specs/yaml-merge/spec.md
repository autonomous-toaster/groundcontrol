## ADDED Requirements

### Requirement: YAML-aware config merge

`groundcontrol init` SHALL parse `openspec/config.yaml` as structured YAML and merge its context and rules into the existing document, rather than appending raw text. `veriplan init` SHALL do the same.

#### Scenario: Merge context into existing config

- **GIVEN** an existing `openspec/config.yaml` with a `context:` field containing "Use conventional commits"
- **WHEN** `groundcontrol init` runs
- **THEN** the `context:` field SHALL contain both "Use conventional commits" and the groundcontrol guidance, separated by a blank line

#### Scenario: Merge rules into existing config

- **GIVEN** an existing `openspec/config.yaml` with `rules.specs` containing "Every requirement MUST use RFC 2119"
- **WHEN** `veriplan init` runs
- **THEN** `rules.specs` SHALL contain both the existing rule and the new veriplan rules, with no duplicates

#### Scenario: Idempotent re-runs

- **GIVEN** a config that has already been initialized by both tools
- **WHEN** either `init` command runs again
- **THEN** the config SHALL remain unchanged (no duplicate entries, no content loss)

#### Scenario: Fresh project (no config)

- **GIVEN** a project with no `openspec/config.yaml`
- **WHEN** `groundcontrol init` runs
- **THEN** a new `openspec/config.yaml` SHALL be created with the groundcontrol context and rules

#### Scenario: Cross-tool preservation

- **GIVEN** a config initialized by `veriplan init`
- **WHEN** `groundcontrol init` runs
- **THEN** the veriplan context and rules SHALL still be present in the merged config

### Requirement: Context must not cite the tool

The `context:` value written by `groundcontrol init` SHALL NOT contain the string "groundcontrol". It SHALL contain only guidance about how to write artifacts. The `context:` value written by `veriplan init` SHALL NOT contain the string "veriplan".

#### Scenario: Groundcontrol context is tool-agnostic

- **GIVEN** a fresh project
- **WHEN** `groundcontrol init` runs
- **THEN** the `context:` value SHALL NOT contain "groundcontrol"
- **AND** the `context:` value SHALL contain guidance about task ID references and aliases

#### Scenario: Veriplan context is tool-agnostic

- **GIVEN** a fresh project
- **WHEN** `veriplan init` runs
- **THEN** the `context:` value SHALL NOT contain "veriplan"
- **AND** the `context:` value SHALL contain guidance about formal state machine modeling
