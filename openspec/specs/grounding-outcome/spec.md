# grounding-outcome Specification

## Purpose
TBD - created by archiving change grounding-diagnostics. Update Purpose after archive.

## Requirements

### Requirement: Absent identifiers are named in the result

T2.1 SHALL report a constant-shaped token that is missing from the Signature BEFORE T2.2 SHALL
render a grounding directive from it. T2.1 SHALL populate `GroundingResult::absent_identifiers`
with every token in the statement whose shape matches a constant of a type declared in the
Signature and for which no `ConstantDef` of that type exists.

#### Scenario: referenced task is absent from the signature

- **WHEN** T2.1 retrieves `T2.6 SHALL publish the port BEFORE T1.1 renames the noun` with a
  signature whose only constant is `T1.1`
- **THEN** `absent_identifiers` SHALL equal `["T2.6"]`
- **THEN** `absent_identifiers` SHALL NOT contain `T1.1`

#### Scenario: absent identifier suppresses a false arity diagnosis

- **WHEN** T2.1 retrieves the same statement and signature
- **THEN** the status SHALL be `Incomplete` and SHALL NOT be `Ambiguous`
- **THEN** no directive SHALL suggest adding task ID references

#### Scenario: absent identifier is reported for non-temporal text

- **WHEN** T2.1 retrieves `T2.6 SHALL publish the port` with a signature whose only constant is
  `T1.1`
- **THEN** `absent_identifiers` SHALL equal `["T2.6"]`

#### Scenario: every referenced token is present

- **WHEN** T2.1 retrieves `T1.1 SHALL complete BEFORE T2.1 SHALL run` with a signature
  declaring both `T1.1` and `T2.1`
- **THEN** `absent_identifiers` SHALL be empty

#### Scenario: an absent identifier suppresses alias binding

- **WHEN** T2.1 retrieves `T1.1 SHALL publish the port BEFORE T2.6 SHALL rename the noun`, where
  `rename` is an alias of `T1.1`
- **THEN** the BEFORE candidate SHALL bind one argument, not two
- **THEN** the status SHALL NOT be `Contradictory`

#### Scenario: no shape is inferable

- **WHEN** T2.1 retrieves a statement with a signature whose constants carry no character-class
  shape
- **THEN** `absent_identifiers` SHALL be empty and grounding SHALL proceed as before

### Requirement: The identifier shape comes from the signature

T2.3 SHALL derive the identifier shape from the names of the Signature's own constants BEFORE
T2.1 SHALL extract absent identifiers. T2.3 SHALL NOT hard-code an identifier syntax.
T2.1 SHALL NOT report a token longer than 12 characters, nor a token longer than the longest
constant name of the matched type.

#### Scenario: a constant-shaped word longer than any constant is not reported

- **WHEN** T2.3 inspects a signature whose longest task ID is `T1.1`
- **THEN** the token `SHALL` SHALL NOT be reported as an absent identifier

#### Scenario: no hard-coded syntax

- **WHEN** T2.3 inspects a signature whose constants have no multi-run shape
- **THEN** absent identifiers SHALL be empty and no error SHALL be raised

### Requirement: Each bound argument records how it matched

T3.1 SHALL record the match kind of every bound argument BEFORE T3.3 SHALL return a grounding
result. T3.1 SHALL produce `BoundArgument::matched` as `MatchKind::Name` when the constant's
name occurs literally in the statement, and `MatchKind::Alias` carrying the matched alias text
when an alias occurs instead. T3.2 SHALL set `GroundedAtom::exact` to true only when every
bound argument matched by name.

#### Scenario: name match is distinguishable from alias match

- **WHEN** T3.1 retrieves `T1.1 SHALL complete BEFORE T2.1 SHALL run`
- **THEN** both bound arguments SHALL report `MatchKind::Name`
- **THEN** `exact` SHALL be true

#### Scenario: alias match is visible

- **WHEN** T3.1 retrieves `the migration SHALL finish BEFORE testing starts`, where `migration`
  and `testing` are aliases
- **THEN** both bound arguments SHALL report `MatchKind::Alias` with the alias text
- **THEN** `exact` SHALL be false

#### Scenario: mixed match kinds

- **WHEN** T3.1 retrieves a statement binding one constant by name and another by alias
- **THEN** exactly one bound argument SHALL report `MatchKind::Name`
- **THEN** `exact` SHALL be false

### Requirement: Alias binding requires no absent identifier

T3.6 SHALL bind arguments by alias only when `absent_identifiers` is empty BEFORE T3.1 SHALL
record the match kind. T3.6 SHALL always bind by name.

#### Scenario: alias suppressed by an unknown identifier

- **WHEN** T3.6 grounds a statement that names a constant absent from the signature and also
  contains words matching an alias of a declared constant
- **THEN** the declared constant SHALL be bound only where its name occurs
- **THEN** an alias match SHALL NOT introduce a second binding of the same constant

#### Scenario: alias still applies when every reference is known

- **WHEN** T3.6 grounds `the migration SHALL finish BEFORE testing starts` with all references
  resolvable
- **THEN** both arguments SHALL bind by alias

### Requirement: Confidence means structure, not identity

T3.4 SHALL document `GroundedAtom::confidence` as structural confidence BEFORE T3.5 SHALL
expose it to callers. T3.4 SHALL score every slot filled by name at 0.95, every slot filled
with at least one alias at 0.9, every slot filled against an undeclared predicate at 0.8, a
short or over-long argument list at 0.5, and no arguments at 0.0. T3.5 SHALL keep the
`Grounded` threshold at 0.8.

#### Scenario: alias-grounded statement is not marked exact

- **WHEN** T3.4 scores a statement that fills every slot via aliases
- **THEN** the atom's confidence SHALL be 0.9 and `exact` SHALL be false

#### Scenario: name-grounded statement

- **WHEN** T3.4 scores a statement that fills every slot by name
- **THEN** the atom's confidence SHALL be 0.95 and `exact` SHALL be true

#### Scenario: no arguments scores zero

- **WHEN** T3.4 scores a candidate with an empty argument list
- **THEN** the atom SHALL NOT be part of the result's candidates

### Requirement: Status separates incomplete grounding from ambiguity

T4.1 SHALL introduce `GroundingStatus::Incomplete` and `GroundingStatus::Contradictory` BEFORE
T4.2 SHALL classify results into them. T4.2 SHALL order classification: contradiction first,
then the 0.8 grounded threshold, then `Incomplete` when absent identifiers exist or a
candidate is short against a declared predicate, then `Ambiguous`, then `Ungroundable`.
T4.3 SHALL serialize the new statuses to stable lower-case strings BEFORE T4.4 SHALL emit
JSON check output.

#### Scenario: incomplete before ambiguous

- **WHEN** T4.2 classifies a result whose only candidate is short against a declared predicate
- **THEN** the status SHALL be `Incomplete`

#### Scenario: ambiguous still reachable

- **WHEN** T4.2 classifies a result with a partial candidate, no absent identifiers, and no
  declared predicate for that candidate
- **THEN** the status SHALL be `Ambiguous`

#### Scenario: ungroundable unchanged

- **WHEN** T4.2 classifies a statement with no predicate keyword and no matching constant
- **THEN** the status SHALL be `Ungroundable`

#### Scenario: status strings are stable

- **WHEN** T4.3 serializes each status
- **THEN** the output SHALL contain `"incomplete"` and `"contradictory"` for the new variants

#### Scenario: absent identifiers survive JSON

- **WHEN** T4.4 serializes a check output whose result carries absent identifiers
- **THEN** the JSON SHALL contain the absent identifier and SHALL round-trip through
  `serde_json` without loss

### Requirement: Directive generation uses the grounder's diagnostics

T5.1 SHALL render an absent-identifier directive from `absent_identifiers` BEFORE T5.2 SHALL
render an alias provenance directive from `BoundArgument::matched`. T5.1 SHALL phrase the
directive as the identifier not being a constant of this signature. T5.2 SHALL NOT emit the
generic "add explicit task ID references" directive when a diagnostic explains the outcome.

#### Scenario: absent identifier directive

- **WHEN** T5.1 renders directives for a result containing `T2.6` in `absent_identifiers`
- **THEN** a directive SHALL name `T2.6` and SHALL state that it is not a task of this
  signature
- **THEN** no directive SHALL suggest adding task ID references

#### Scenario: alias directive comes from provenance

- **WHEN** T5.2 renders directives for a result whose bound argument matched via alias
  `migration`
- **THEN** a directive SHALL name the alias `migration` and the constant it bound

#### Scenario: human output shows the diagnosis

- **WHEN** T5.1 prints the human-readable check output for an `Incomplete` input
- **THEN** the printed text SHALL name the absent identifier

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
