## MODIFIED Requirements

### Requirement: The identifier shape comes from the signature

T2.3 SHALL derive the identifier shape from the names of the Signature's own constants BEFORE
T2.1 SHALL extract absent identifiers. T2.3 SHALL NOT hard-code an identifier syntax, and SHALL
NOT bound a candidate's length by the length of any declared constant name: a signature that
declares only short identifiers SHALL still recognise a longer one. T2.1 SHALL report a token only
where the characters on either side of it do not continue an identifier, so the reported string is
byte-identical to the input text. T2.1 SHALL NOT report a token longer than the fixed sanity cap.

#### Scenario: a constant-shaped word longer than any constant is not reported

- **WHEN** T2.3 inspects a signature whose longest task ID is `T1.1`
- **THEN** the token `SHALL` SHALL NOT be reported as an absent identifier

#### Scenario: no hard-coded syntax

- **WHEN** T2.3 inspects a signature whose constants have no multi-run shape
- **THEN** absent identifiers SHALL be empty and no error SHALL be raised

#### Scenario: a multi-digit identifier exceeds the longest declared name

- **WHEN** T2.1 extracts from `T2.10 SHALL complete BEFORE T1.1 SHALL start` with a signature
  declaring only `T1.1` and `T1.2`
- **THEN** `absent_identifiers` SHALL equal `["T2.10"]`
- **THEN** it SHALL NOT contain `T2.1`

#### Scenario: a foreign multi-digit identifier is not swallowed

- **WHEN** T2.1 extracts from `T1.10 SHALL complete BEFORE T1.1 SHALL start` with a signature
  declaring only `T1.1`
- **THEN** `absent_identifiers` SHALL equal `["T1.10"]`

#### Scenario: reported identifiers are byte-identical

- **WHEN** T2.1 reports any absent identifier
- **THEN** that exact string SHALL occur in the input text

#### Scenario: a sentence-final dot is not part of the identifier

- **WHEN** T2.1 extracts from `T2.10 SHALL run BEFORE T1.1 SHALL stop.`
- **THEN** `absent_identifiers` SHALL equal `["T2.10"]`
- **THEN** the trailing dot SHALL NOT be included

#### Scenario: a prefix identifier is never bound

- **WHEN** T2.1 grounds `T1.10 SHALL complete BEFORE T1.2 SHALL start` against a signature
  declaring `T1.1`, `T1.2` and `T1.10`
- **THEN** the bound arguments SHALL be `T1.10` and `T1.2`
- **THEN** `T1.1` SHALL NOT be bound
