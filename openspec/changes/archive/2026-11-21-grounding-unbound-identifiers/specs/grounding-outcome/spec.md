## ADDED Requirements

### Requirement: Named-but-unbound constants are reported

T7.1 SHALL populate `GroundingResult::unused_identifiers` with every constant whose name
occurs in the statement and that no candidate bound BEFORE T7.2 SHALL render a directive from
it. T7.3 SHALL leave the field empty when no candidate was produced. T7.1 SHALL NOT report a
constant whose name only occurs inside a longer identifier.

#### Scenario: a named task the constraint cannot use

- **WHEN** T7.1 grounds `T5.15 SHALL ALWAYS mention T5.16` against a signature declaring both
  `T5.15` and `T5.16`
- **THEN** `unused_identifiers` SHALL equal `["T5.16"]`
- **THEN** `absent_identifiers` SHALL be empty

#### Scenario: every named task is bound

- **WHEN** T7.1 grounds `T5.15 SHALL run BEFORE T5.16 SHALL stop`
- **THEN** `unused_identifiers` SHALL be empty

#### Scenario: no candidate produced

- **WHEN** T7.3 grounds `T5.15 does something`, which matches no predicate keyword
- **THEN** `unused_identifiers` SHALL be empty
- **THEN** the status SHALL be `Ungroundable`

#### Scenario: a prefix collision is not an unbound reference

- **WHEN** T7.1 grounds `T5.15 SHALL admit an input BEFORE T5.16 SHALL reject it` against a
  signature that also declares `T5.1`
- **THEN** `unused_identifiers` SHALL be empty

#### Scenario: the directive does not blame the author

- **WHEN** T7.2 renders a directive for `T5.16` in `unused_identifiers`
- **THEN** the directive SHALL name `T5.16` and SHALL state that it was named but not bound
- **THEN** the suggested action SHALL NOT ask for additional task ID references

#### Scenario: an unbound reference does not fail the check

- **WHEN** the check runs over only grounded inputs, at least one carrying an unbound
  identifier
- **THEN** the diagnostics SHALL be printed and the process SHALL exit 0
