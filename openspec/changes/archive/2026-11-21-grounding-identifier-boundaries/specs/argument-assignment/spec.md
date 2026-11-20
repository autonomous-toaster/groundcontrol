## MODIFIED Requirements

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

## ADDED Requirements

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
