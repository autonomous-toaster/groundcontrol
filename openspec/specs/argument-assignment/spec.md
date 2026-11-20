# argument-assignment Specification

## Purpose
TBD - created by archiving change grounding-diagnostics. Update Purpose after archive.

## Requirements

### Requirement: Distinct slots reject a repeated constant

T1.1 SHALL reject a binding that repeats one constant across distinct slots BEFORE T1.2 SHALL
score the candidate. T1.1 SHALL treat a predicate as distinguishing its slots when it declares
more than one arg slot with pairwise distinct slot names. T1.3 SHALL mark such a candidate as
contradictory BEFORE T1.2 SHALL apply the 0.8 grounded threshold.

#### Scenario: before with the same task twice

- **WHEN** T1.1 retrieves `T1.1 SHALL write the header BEFORE T1.1 SHALL seal it`
- **THEN** the BEFORE candidate SHALL be marked contradictory
- **THEN** the result status SHALL NOT be `Grounded`

#### Scenario: after with the same task twice

- **WHEN** T1.1 retrieves `T1.1 SHALL run AFTER T1.1 SHALL complete`
- **THEN** the AFTER candidate SHALL be marked contradictory

#### Scenario: concurrently with the same task twice

- **WHEN** T1.1 retrieves `T1.1 and T1.1 SHALL run concurrently`
- **THEN** the CONCURRENTLY candidate SHALL be marked contradictory

#### Scenario: at most one with the same task twice

- **WHEN** T1.1 retrieves `At most one of T1.1, T1.1 SHALL be active`
- **THEN** the AT_MOST_ONE candidate SHALL be marked contradictory

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
