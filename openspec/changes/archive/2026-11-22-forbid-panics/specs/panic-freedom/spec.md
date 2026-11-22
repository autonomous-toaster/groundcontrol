## ADDED Requirements

### Requirement: Shipped code contains no panic-capable construct

T1.1 SHALL keep `panic`, `unwrap_used`, `expect_used`, `indexing_slicing`, `string_slice`,
`todo`, `unimplemented` and `unreachable` denied in `[lints.clippy]` BEFORE T1.2 SHALL keep the
build green. T1.2 SHALL report zero violations of those lints in non-test code. T1.3 SHALL keep
`unsafe_code` forbidden.

#### Scenario: a panic-capable construct is added

- **WHEN** T1.1 checks a change that introduces `.unwrap()` or `s[..n]` in shipped code
- **THEN** the build SHALL fail and name the construct

#### Scenario: the policy is weakened

- **WHEN** T1.1 finds any of those lints set to `allow` or removed from the table
- **THEN** `tests/panic_policy.rs` SHALL fail and name the lint

### Requirement: The opt-out is confined to test code

T2.1 SHALL allow the panic lints to be overridden only inside test modules BEFORE T2.2 SHALL
accept the build. T2.2 SHALL fail when an override appears in a file or module that is not
test-only.

#### Scenario: an override escapes test code

- **WHEN** T2.2 scans `src/` and finds the inner allow attribute in a module that is not a test
  module
- **THEN** the check SHALL fail and name the file

#### Scenario: a test module opts out

- **WHEN** T2.1 inspects a test module carrying the inner allow attribute
- **THEN** T2.2 SHALL accept it

### Requirement: Fallible string work is total

T3.1 SHALL express every string and slice access as a total operation BEFORE T3.2 SHALL expose
it to a caller. T3.1 SHALL NOT fabricate an error to satisfy a `Result` return type.

#### Scenario: a node range has no text

- **WHEN** T3.1 reads a tree-sitter node whose range is outside the source, or splits a
  character
- **THEN** the helper SHALL return no text rather than panic or invent an encoding error

#### Scenario: behaviour is unchanged

- **WHEN** the total rewrite lands
- **THEN** `tests/fixtures/grounding_cases.json` SHALL be unchanged
