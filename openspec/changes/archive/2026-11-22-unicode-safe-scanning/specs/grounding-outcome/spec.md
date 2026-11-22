## ADDED Requirements

### Requirement: Scanning is total over Unicode text

T8.1 SHALL produce a `GroundingResult` for every statement BEFORE T8.2 SHALL index any
substring. T8.1 SHALL NOT panic, abort, or return an error for a statement containing
characters outside ASCII. T8.2 SHALL convert between character indices and byte offsets
explicitly rather than using one as the other. T8.3 SHALL search case-insensitively in the
string being sliced, so no offset is taken from a lowercased copy.

#### Scenario: a multi-byte character follows an identifier

- **WHEN** T8.1 grounds `T1.1 SHALL forward the upstream status during H2→H1.1 transcoding.`
- **THEN** it SHALL return a result and SHALL NOT panic
- **THEN** `absent_identifiers` SHALL contain `H1.1`

#### Scenario: an em dash separates two tasks

- **WHEN** T8.1 grounds `T1.1 SHALL complete — BEFORE T2.2 SHALL start`
- **THEN** the bound arguments SHALL be `T1.1` and `T2.2`

#### Scenario: accented words surround the identifiers

- **WHEN** T8.1 grounds a statement whose surrounding prose is accented
- **THEN** the named tasks SHALL still bind in order

#### Scenario: a lowercase-expanding character precedes a trailing keyword

- **WHEN** T8.3 grounds a statement padded with `İ` and ending in `before`
- **THEN** it SHALL return a result and SHALL NOT panic

#### Scenario: the sweep

- **WHEN** T8.1 grounds every statement shape in the test sweep with six different multi-byte
  paddings
- **THEN** no case SHALL panic
