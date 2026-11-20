## ADDED Requirements

### Requirement: Constant names match on identifier boundaries only

T6.1 SHALL match a constant name only where neither adjacent character continues an
identifier BEFORE T2.1 SHALL extract absent identifiers. T6.1 SHALL treat a following `.` as a
boundary only when no digit follows it. T6.2 SHALL apply the same boundary rule to aliases
BEFORE T3.1 SHALL record the match kind.

#### Scenario: a prefix name does not match a longer identifier

- **WHEN** T6.1 retrieves `T5.15 SHALL admit an input BEFORE T5.16 SHALL reject it` against a
  signature declaring `T5.1`, `T5.10`, `T5.15` and `T5.16`
- **THEN** `T5.1` SHALL NOT be considered present in the text
- **THEN** the bound arguments SHALL be `T5.15` and `T5.16`

#### Scenario: a trailing sentence dot does not break the match

- **WHEN** T6.1 retrieves `T5.15 SHALL run BEFORE T5.16 SHALL stop.`
- **THEN** the bound arguments SHALL be `T5.15` and `T5.16`

#### Scenario: a dotted continuation is not a boundary

- **WHEN** T6.1 retrieves text containing `T5.1.5` against a signature declaring `T5.1`
- **THEN** `T5.1` SHALL NOT be considered present in the text

#### Scenario: aliases respect boundaries too

- **WHEN** T6.2 retrieves text containing `server` against a constant whose alias is `serve`
- **THEN** that alias SHALL NOT be considered present in the text

#### Scenario: a prefix collision no longer produces a contradiction

- **WHEN** T6.1 retrieves `T3.12 SHALL serve the document BEFORE T3.13 SHALL serve the schema`
  against a signature declaring `T3.1`, `T3.12` and `T3.13`
- **THEN** the status SHALL be `Grounded` and SHALL NOT be `Contradictory`
