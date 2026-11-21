# argument-assignment Specification

## Purpose
TBD - created by archiving change grounding-diagnostics. Update Purpose after archive.

## Requirements

### Requirement: Distinct slots reject a repeated constant

T1.1 SHALL reject a binding that repeats one name-matched constant across distinct slots
BEFORE T1.2 SHALL score the candidate. T1.1 SHALL treat a predicate as distinguishing its
slots when it declares more than one arg slot with pairwise distinct slot names. T1.3 SHALL
mark such a candidate as contradictory BEFORE T1.2 SHALL apply the 0.8 grounded threshold.
T1.6 SHALL NOT mark a candidate contradictory when the repeated constant matched by alias in
either slot; T1.6 SHALL score such a candidate at 0.5.

#### Scenario: before with the same task twice

- **WHEN** T1.1 retrieves `T1.1 SHALL write the header BEFORE T1.1 SHALL seal it`
- **THEN** the BEFORE candidate SHALL be marked contradictory
- **THEN** the result status SHALL NOT be `Grounded`

#### Scenario: after with the same task twice

- **WHEN** T1.1 retrieves `T1.1 SHALL run AFTER T1.1 SHALL complete`
- **THEN** the AFTER candidate SHALL be marked contradictory

#### Scenario: concurrently with the same task twice

- **WHEN** T1.1 retrieves `T1.1 and T1.1 SHALL run concurrently`
- **THEN** the CONCURRENTLY candidate SHALL NOT be `Grounded`

#### Scenario: at most one with the same task twice

- **WHEN** T1.1 retrieves `At most one of T1.1, T1.1 SHALL be active`
- **THEN** the AT_MOST_ONE candidate SHALL NOT be `Grounded`

#### Scenario: if then with the same task twice

- **WHEN** T1.1 retrieves `IF T1.1 fails THEN T1.1 SHALL run`
- **THEN** the IF_THEN candidate SHALL be marked contradictory

#### Scenario: distinct tasks still ground

- **WHEN** T1.1 retrieves `T1.1 SHALL write the header BEFORE T2.1 SHALL seal it`
- **THEN** the BEFORE candidate SHALL NOT be marked contradictory
- **THEN** the result status SHALL be `Grounded`

#### Scenario: single-slot predicate has nothing to distinguish

- **WHEN** T1.1 retrieves `T1.1 SHALL ALWAYS be available`
- **THEN** the ALWAYS candidate SHALL NOT be marked contradictory

#### Scenario: an alias-driven repeat is not a contradiction

- **WHEN** T1.6 binds one constant to both slots of a distinguishing predicate and at least
  one binding matched by alias
- **THEN** the candidate's confidence SHALL be 0.5
- **THEN** the result status SHALL be `Ambiguous` and SHALL NOT be `Contradictory`

#### Scenario: a name repeat is still a contradiction

- **WHEN** T1.6 binds one constant to both slots and both bindings matched by name
- **THEN** the result status SHALL be `Contradictory`

### Requirement: Distinctness is read from the signature's slot names

T1.4 SHALL read slot distinctness from `PredicateDef::arguments` BEFORE T1.1 SHALL reject a
repeated constant. T1.4 SHALL NOT use a hard-coded predicate allow-list. T1.5 SHALL keep
grounding of an undeclared predicate independent of the distinctness rule when no slot names
exist.

#### Scenario: a two-slot predicate with repeated slot names is reflexive

- **WHEN** T1.4 inspects a predicate declaring two args both named `x`
- **THEN** that predicate SHALL be treated as not distinguishing its slots

#### Scenario: a two-slot predicate with distinct names distinguishes

- **WHEN** T1.4 inspects a predicate declaring args named `earlier` and `later`
- **THEN** that predicate SHALL be treated as distinguishing its slots

### Requirement: A contradictory binding is reported, not silently downgraded

T2.1 SHALL expose a contradictory binding in the returned atom BEFORE T2.2 SHALL render a
directive for it. T2.1 SHALL keep the bound arguments and SHALL set confidence to 0.5.
T2.2 SHALL phrase the directive as one task occupying two slots that require different tasks.

#### Scenario: contradictory atom is inspectable

- **WHEN** T2.1 returns the result for `T1.1 SHALL write the header BEFORE T1.1 SHALL seal it`
- **THEN** the BEFORE candidate SHALL retain both bound arguments
- **THEN** the candidate's confidence SHALL be 0.5

#### Scenario: contradictory directive

- **WHEN** T2.2 renders directives for that result
- **THEN** a directive SHALL name the repeated task and the two slot names

#### Scenario: contradictory wins over a later grounded candidate

- **WHEN** T2.1 returns a result with one contradictory candidate and one candidate scoring at
  least 0.8
- **THEN** the result status SHALL be `Contradictory`

### Requirement: A name match wins the slot over an alias

T3.1 SHALL bind a slot to a constant whose name occurs in the slot's text BEFORE T3.2 SHALL
consider any alias. T3.2 SHALL bind by alias only when no name match exists for that slot.

#### Scenario: an alias cannot displace a named constant

- **WHEN** T3.1 fills the earlier slot of `T5.15 SHALL admit an input BEFORE T5.16 SHALL reject it`
  and `T5.1` carries the alias `admit an input`
- **THEN** the slot SHALL bind `T5.15`

#### Scenario: alias binding still fills a slot with no name match

- **WHEN** T3.2 fills a slot whose text contains only an alias of a declared constant
- **THEN** the slot SHALL bind that constant with `MatchKind::Alias`

### Requirement: A single-slot predicate binds the first mention in the text

T4.1 SHALL bind a predicate with one arg slot to the constant whose mention occurs earliest in
the statement BEFORE T4.2 SHALL consider signature order. T4.2 SHALL break ties by signature
order so the choice stays deterministic. T4.1 SHALL bind through the same path as multi-slot
predicates, so `MatchKind` reports the real match and alias binding obeys the absent-identifier
suppression rule.

#### Scenario: the subject is the later constant in signature order

- **WHEN** T4.1 grounds `T1.10 SHALL ALWAYS mention T1.1` against a signature declaring `T1.1`
  and `T1.10`
- **THEN** the bound argument SHALL be `T1.10`
- **THEN** `unused_identifiers` SHALL equal `["T1.1"]`

#### Scenario: the same pair in the other order

- **WHEN** T4.1 grounds `T1.1 SHALL ALWAYS mention T1.10` against the same signature
- **THEN** the bound argument SHALL be `T1.1`
- **THEN** `unused_identifiers` SHALL equal `["T1.10"]`

#### Scenario: one named task is unaffected

- **WHEN** T4.1 grounds a statement naming exactly one task
- **THEN** the binding SHALL be that task and `unused_identifiers` SHALL be empty

#### Scenario: an alias-only single-slot match reports an alias

- **WHEN** T4.1 binds a single-slot predicate to a constant that appears only through an alias
- **THEN** the argument's `MatchKind` SHALL be `Alias`
- **THEN** `exact` SHALL be false and confidence SHALL be 0.9
