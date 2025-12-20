# Lens Specifications

This document defines the five lenses used by Steward to evaluate outputs against stewardship contracts. Each lens asks one stewardship question and operates independently.

## Core Principles

1. **Independence**: Lenses evaluate in parallel and cannot access other lenses' findings
2. **No Inter-Lens Communication**: Synthesis is policy, not intelligence
3. **Determinism**: Same input always produces same output
4. **Evidence-Based**: Every finding must cite evidence from contract, output, or context
5. **Bounded**: Each lens evaluates only the rules assigned to it

---

## Lens 1: Dignity & Inclusion

### Stewardship Question
> Does this disempower people or exclude them from relevance?

### Contract Rules Evaluated
- `acceptance.dignity_check[]`
- `boundaries.must_escalate_when[]` (dignity-related rules only)
- `intent.never_optimize_away[]` (dignity-related rules only)

### Contract Rules NOT Evaluated
- `boundaries.may_do_autonomously[]`
- `boundaries.invalidated_by[]` (handled by Boundaries lens)
- `accountability.*`

### Evaluation Logic

```
for each rule in dignity_check:
    if output matches violation pattern:
        return BLOCKED(rule_id, evidence)
    if output uncertain on dignity:
        return ESCALATE(reason)

for each rule in never_optimize_away (dignity-related):
    if output appears to optimize away this constraint:
        return BLOCKED(rule_id, evidence)

return PASS
```

### State Conditions

| State | Condition |
|-------|-----------|
| **PASS** | All dignity rules satisfied, no exclusion detected |
| **ESCALATE** | Uncertainty about dignity impact, ambiguous language |
| **BLOCKED** | Output assumes away human agency, creates pressure without recourse, excludes groups by default |

### Evidence Requirements
- BLOCKED must cite specific text location where dignity violation occurs
- Evidence pointer format: `output.content[start:end]` or `context[index][start:end]`

---

## Lens 2: Boundaries & Safety

### Stewardship Question
> Does this respect defined scope and stop conditions?

### Contract Rules Evaluated
- `boundaries.may_do_autonomously[]`
- `boundaries.must_pause_when[]`
- `boundaries.must_escalate_when[]`
- `boundaries.invalidated_by[]`

### Contract Rules NOT Evaluated
- `acceptance.dignity_check[]` (handled by Dignity lens)
- `acceptance.fit_criteria[]` (handled by Transparency lens)
- `accountability.*`

### Evaluation Logic

```
// Check invalidation first (most severe)
for each rule in invalidated_by:
    if output matches rule:
        return BLOCKED(rule_id, evidence)

// Check escalation conditions
for each rule in must_escalate_when:
    if output or context matches rule:
        return ESCALATE(rule_id, options)

// Check pause conditions
for each rule in must_pause_when:
    if output or context matches rule:
        return ESCALATE(rule_id, options)

// Verify within autonomous scope
for each action in output:
    if action not covered by may_do_autonomously:
        return ESCALATE("Action outside autonomous scope")

return PASS
```

### State Conditions

| State | Condition |
|-------|-----------|
| **PASS** | Output within autonomous scope, no stop conditions triggered |
| **ESCALATE** | Pause or escalation condition matched, action outside scope |
| **BLOCKED** | Invalidation condition matched (PII exposure, scope violation, etc.) |

### Evidence Requirements
- BLOCKED must cite both the violated rule and the location in output
- ESCALATE must include the specific condition that triggered it

### Pattern Matching (Phase 1)

For deterministic evaluation without LLM, Boundaries lens uses pattern matching:

| Pattern Type | Implementation |
|--------------|----------------|
| PII Detection | Regex for emails, phone numbers, SSN patterns |
| Keyword Match | Case-insensitive keyword presence |
| Phrase Match | Exact phrase matching |
| Scope Check | Output action classification |

---

## Lens 3: Restraint & Privacy

### Stewardship Question
> Does this expose what should be protected?

### Contract Rules Evaluated
- `boundaries.invalidated_by[]` (privacy-related rules only)
- `intent.never_optimize_away[]` (privacy-related rules only)

### Contract Rules NOT Evaluated
- `boundaries.may_do_autonomously[]`
- `boundaries.must_escalate_when[]`
- `acceptance.*`
- `accountability.*`

### Evaluation Logic

```
// Check for PII exposure
if output contains PII patterns:
    if PII rule in invalidated_by:
        return BLOCKED(rule_id, evidence with PII location)

// Check for credential/secret exposure
if output contains credential patterns:
    return BLOCKED("credentials_exposed", evidence)

// Check scope creep
if output accesses data beyond stated scope:
    return ESCALATE("scope_creep")

// Check data minimization
if output includes unnecessary data:
    return ESCALATE("data_minimization")

return PASS
```

### State Conditions

| State | Condition |
|-------|-----------|
| **PASS** | No protected data exposed, scope respected |
| **ESCALATE** | Potential scope creep, data minimization concern |
| **BLOCKED** | PII exposed, credentials exposed, explicit privacy violation |

### Built-in Patterns

| Category | Patterns |
|----------|----------|
| Email | `[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}` |
| Phone | `\+?[1-9]\d{1,14}`, `\(\d{3}\) \d{3}-\d{4}` |
| SSN | `\d{3}-\d{2}-\d{4}` |
| Credit Card | `\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}` |
| API Key | `(api[_-]?key|secret|token)[\s:=]+[a-zA-Z0-9]{20,}` |

---

## Lens 4: Transparency & Contestability

### Stewardship Question
> Can the human understand why this happened and contest it?

### Contract Rules Evaluated
- `acceptance.fit_criteria[]`
- `intent.purpose` (for alignment check)

### Contract Rules NOT Evaluated
- `boundaries.*`
- `accountability.*`
- `acceptance.dignity_check[]`

### Evaluation Logic

```
for each rule in fit_criteria:
    if output fails to meet criterion:
        if criterion is mandatory:
            return BLOCKED(rule_id, evidence)
        else:
            confidence -= penalty

// Check transparency indicators
if output makes claims without sources:
    if citation rule in fit_criteria:
        return ESCALATE("uncited_claims")

// Check for hidden assumptions
if output contains unstated assumptions:
    return ESCALATE("unstated_assumptions")

// Check for contestability path
if no clear way to challenge output:
    return ESCALATE("no_contestability_path")

if confidence < 0.4:
    return ESCALATE("low_confidence")

return PASS(confidence)
```

### State Conditions

| State | Condition |
|-------|-----------|
| **PASS** | Output addresses question, sources cited, assumptions visible |
| **ESCALATE** | Uncited claims, hidden assumptions, no contestability path |
| **BLOCKED** | Mandatory fit criterion violated |

---

## Lens 5: Accountability & Ownership

### Stewardship Question
> Who approved this, who can stop it, and who answers for it?

### Contract Rules Evaluated
- `accountability.approved_by`
- `accountability.answerable_human`
- `accountability.escalation_path[]`

### Contract Rules NOT Evaluated
- `boundaries.*`
- `acceptance.*`
- `intent.*`

### Evaluation Logic

```
// Verify contract has required accountability fields
if answerable_human is missing or empty:
    return BLOCKED("no_accountable_human")

// Check escalation path exists
if escalation_path is empty and any ESCALATE could occur:
    return ESCALATE("no_escalation_path")

// Verify approval
if approved_by is missing for high-risk contract:
    return ESCALATE("no_approval")

// All accountability requirements met
return PASS
```

### State Conditions

| State | Condition |
|-------|-----------|
| **PASS** | Accountable human defined, escalation path exists |
| **ESCALATE** | Missing approval, unclear escalation path |
| **BLOCKED** | No accountable human defined (contract is invalid) |

---

## Confidence Calculation

Each lens calculates confidence based on evidence quality:

```rust
fn calculate_confidence(rules_evaluated: &[RuleEvaluation]) -> f64 {
    if rules_evaluated.is_empty() {
        return 0.5; // Default when no rules apply
    }

    let mut confidence = 1.0;

    for rule in rules_evaluated {
        match rule.result {
            Satisfied => {
                // High evidence = minimal penalty
                let penalty = match rule.evidence.len() {
                    0 => 0.1,      // No evidence
                    1 => 0.05,     // Single evidence
                    _ => 0.02,     // Multiple evidence
                };
                confidence -= penalty;
            }
            Uncertain => {
                // Uncertain reduces confidence significantly
                confidence -= 0.2;
            }
            Violated | NotApplicable => {
                // No confidence impact (BLOCKED handled separately)
            }
        }
    }

    confidence.max(0.0).min(1.0)
}
```

### Confidence Thresholds

| Range | Interpretation | Effect |
|-------|----------------|--------|
| >= 0.7 | High confidence | No automatic escalation |
| 0.4 - 0.7 | Moderate confidence | Noted in findings |
| < 0.4 | Low confidence | Triggers ESCALATE if no BLOCKED |

---

## Synthesizer Rules

The Synthesizer aggregates lens findings into final state:

```rust
fn synthesize(findings: &LensFindings) -> State {
    let all_lenses = [
        &findings.dignity_inclusion,
        &findings.boundaries_safety,
        &findings.restraint_privacy,
        &findings.transparency_contestability,
        &findings.accountability_ownership,
    ];

    // Rule 1: Any BLOCKED -> BLOCKED
    for lens in all_lenses {
        if let LensState::Blocked { violation } = &lens.state {
            return State::Blocked {
                violation: build_violation(lens, violation),
            };
        }
    }

    // Rule 2: Any ESCALATE -> ESCALATE
    for lens in all_lenses {
        if let LensState::Escalate { reason } = &lens.state {
            return State::Escalate {
                uncertainty: reason.clone(),
                decision_point: build_decision_point(lens),
                options: build_options(lens),
            };
        }
    }

    // Rule 3: Otherwise -> PROCEED
    State::Proceed {
        summary: build_summary(findings),
    }
}

fn calculate_overall_confidence(findings: &LensFindings) -> f64 {
    // Minimum of all lens confidences
    [
        findings.dignity_inclusion.confidence,
        findings.boundaries_safety.confidence,
        findings.restraint_privacy.confidence,
        findings.transparency_contestability.confidence,
        findings.accountability_ownership.confidence,
    ]
    .iter()
    .cloned()
    .fold(f64::INFINITY, f64::min)
}
```

---

## Lens Execution Model

Based on Amy's fan-out/fan-in architecture:

```
                    ┌──────────────────────────────┐
                    │     CONTRACT + OUTPUT        │
                    └──────────────┬───────────────┘
                                   │
                    ═══════════════╪═══════════════
                         FAN-OUT (parallel)
                    ═══════════════╪═══════════════
            ┌──────────┬───────────┼───────────┬──────────┐
            ▼          ▼           ▼           ▼          ▼
       ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
       │Dignity │ │Boundary│ │Restraint│ │Transp. │ │Account.│
       │  Lens  │ │  Lens  │ │  Lens  │ │  Lens  │ │  Lens  │
       └────┬───┘ └────┬───┘ └────┬───┘ └────┬───┘ └────┬───┘
            │          │          │          │          │
            ▼          ▼          ▼          ▼          ▼
       ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
       │Finding │ │Finding │ │Finding │ │Finding │ │Finding │
       └────┬───┘ └────┬───┘ └────┬───┘ └────┬───┘ └────┬───┘
            │          │          │          │          │
                    ═══════════════╪═══════════════
                         FAN-IN (collect)
                    ═══════════════╪═══════════════
                                   │
                                   ▼
                    ┌──────────────────────────────┐
                    │         SYNTHESIZER          │
                    │  (strict policy, no debate)  │
                    └──────────────┬───────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────────┐
                    │     EVALUATION RESULT        │
                    └──────────────────────────────┘
```

### Key Properties

1. **Thread-Safe Collection**: Lens findings collected with mutex protection
2. **No Dependencies**: Lenses have no dependencies on each other
3. **Deterministic Ordering**: BTreeMap used for deterministic iteration
4. **Fail-Fast**: First BLOCKED encountered is returned (but all lenses still run)

---

## Implementation Notes

### Phase 1 (Boundaries Lens Only)

For the initial implementation, only the Boundaries lens is fully implemented:
- Pattern matching for PII detection
- Keyword matching for escalation conditions
- Scope verification against `may_do_autonomously`

Other lenses return default PASS with documented TODO markers.

### Future Phases

- Phase 2: All five lenses implemented
- Phase 5: Optional LLM-assisted evaluation in `steward-runtime` for ambiguous rules

---

*This document is the authoritative specification for lens behavior. Any implementation that deviates from this document is a bug.*
