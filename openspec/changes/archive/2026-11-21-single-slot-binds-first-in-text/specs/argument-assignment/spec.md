## ADDED Requirements

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
