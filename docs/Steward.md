# Steward Design Overview

## What is Stewardship?

A steward is not someone who executes the system. A steward is someone who decides what the system is allowed to do, when it must stop, and who is accountable when it acts.

**The shortest definition:**
> A steward designs and maintains the conditions under which automation operates responsibly. They don't push the buttons. They decide which buttons exist at all.

---

## The Governance Calculus

Steward has a formal foundation that makes it governance infrastructure, not opinion:

- A formal language (contracts + rule IDs + schemas)
- Independent evaluators (lenses)
- A deterministic reduction (synthesizer)
- A partial order over outcomes (BLOCKED dominates ESCALATE dominates PROCEED)
- A conservative confidence operator (min())

### 1. Inputs and Types

Let:
- **C** = contract (human-authored policy)
- **O** = output (automation result)
- **X** = context (what the system had access to)
- **M** = metadata (optional)

Evaluation is a pure function:

```
E(C, O, X, M) → R
```

where R is an EvaluationResult.

### 2. Lens Semantics

There are 5 lenses L₁..L₅. Each lens evaluates only its allowed rule set:

```
Fᵢ = Lᵢ(C, O, X, M)
```

Each Fᵢ returns:
- **LensState** ∈ {Pass, Escalate, Blocked}
- Evidence pointers
- Lens confidence confᵢ ∈ [0,1]

### 3. Synthesizer as a Strict Policy Machine

Final state is computed by a non-configurable reduction:

```
state(R) =
  Blocked   if ∃i: state(Fᵢ) = Blocked
  Escalate  else if ∃i: state(Fᵢ) = Escalate
  Proceed   otherwise
```

This is the **governance dominance law**. It is not configurable. It is the policy.

### 4. Confidence Calculus

Overall confidence is conservative:

```
conf(R) = min(conf₁, conf₂, conf₃, conf₄, conf₅)
```

The honesty rule:
- If conf(R) < 0.4 and no lens is Blocked → force ESCALATE

Uncertainty is a governance signal, not an error.

### 5. Evidence Requirement as an Invariant

For any BLOCKED result, evidence must exist:

```
state(R) = Blocked ⟹ |evidence(R)| ≥ 1
```

That one line is why this is governance and not "vibes."

---

## Steward vs Guardrails

| | Guardrails | Steward |
|---|------------|---------|
| **When** | Around model calls (reactive) | Before automation exists (governance-first) |
| **Question** | "Is this output allowed?" | "Should this automation proceed, and who answers for it?" |
| **Criteria** | Model-based moderation | Human-authored, explicit, stable |
| **Output** | Pass/fail per call | PROCEED / ESCALATE / BLOCKED with evidence trail |
| **Accountability** | Implicit | Explicit `accountable_human` in every contract |
| **Confidence** | Hidden or absent | Surfaced; low confidence triggers ESCALATE |

Steward doesn't replace guardrails. It operates upstream—defining the conditions under which automation should exist at all.

---

## The Five Lenses

Each lens asks one stewardship question. They evaluate **independently**—lenses don't debate or persuade each other. Synthesis is policy, not intelligence.

### Lens 1: Dignity & Inclusion

**Question:** Does this disempower people or exclude them from relevance?

**Examines:**
- Who is made invisible by this output?
- Whose judgment is removed?
- Is there silent coercion or pressure?
- Are escape hatches to human help preserved?

**Boundary violations trigger BLOCKED when:**
- Output assumes away human agency
- Creates pressure without recourse
- Excludes groups by default assumption

**Contract rules evaluated:**
- `acceptance.dignity_check[]`
- `boundaries.must_escalate_when[]` (dignity-related)

### Lens 2: Boundaries & Safety

**Question:** What conditions should invalidate this automation entirely?

**Examines:**
- Does the output respect defined scope?
- Does it fail safely?
- Are failure modes known and handled?
- Are stop conditions honored?

**Boundary violations trigger BLOCKED when:**
- Output operates outside `boundaries.may_do_autonomously[]`
- Matches any `boundaries.invalidated_by[]` condition
- Ignores `boundaries.must_pause_when[]` triggers

**Contract rules evaluated:**
- `boundaries.may_do_autonomously[]`
- `boundaries.must_pause_when[]`
- `boundaries.must_escalate_when[]`
- `boundaries.invalidated_by[]`

### Lens 3: Restraint & Privacy

**Question:** What must this system never be allowed to do, even if it could?

**Examines:**
- Does it take only what it needs?
- Does it expose what should be protected?
- Does it respect scope limits?
- Is data minimized?

**Boundary violations trigger BLOCKED when:**
- PII exposure detected
- Secrets or credentials exposed
- Scope creep beyond defined authority
- Data retention violations

**Contract rules evaluated:**
- `boundaries.invalidated_by[]` (privacy-related)
- `intent.never_optimize_away[]` (privacy-related)

### Lens 4: Transparency & Contestability

**Question:** Can the human understand why this happened and contest it?

**Examines:**
- Are assumptions visible?
- Is uncertainty disclosed?
- Can the decision be challenged?
- Is AI involvement indicated?

**Triggers ESCALATE when:**
- Assumptions are unstated
- Uncertainty is hidden
- No path to contest exists

**Contract rules evaluated:**
- `acceptance.fit_criteria[]` (transparency-related)

### Lens 5: Accountability & Ownership

**Question:** If something goes wrong, who approved it, why, and who can stop it?

**Examines:**
- Is ownership clear?
- Is escalation path defined?
- Is there audit trail?
- Can someone stop this?

**Triggers ESCALATE when:**
- Ownership is unclear
- Escalation path is missing
- No way to halt automation

**Contract rules evaluated:**
- `accountability.approved_by`
- `accountability.answerable_human`
- `accountability.escalation_path[]`

---

## Design Principles

1. **Deterministic** — Same contract + same output = same result. Always.
2. **Traceable** — Every BLOCKED cites a rule ID and evidence pointer.
3. **Honest** — Low confidence triggers ESCALATE, not guessing.
4. **Upstream** — Contracts are defined before AI runs, not after.
5. **Human-centered** — BLOCKED identifies the accountable human. ESCALATE presents options, not recommendations.

---

## What Steward Is Not

**Not an LLM-as-a-judge** — Steward doesn't ask a model "Is this good?" Criteria are human-authored and explicit. Models that grade themselves hide accountability.

**Not a quality scorer** — Numeric scores hide boundary violations and encourage threshold gaming. Steward returns states, not numbers.

**Not a recommendation engine** — ESCALATE presents options to humans without ranking them. Steward surfaces decisions, it doesn't make them.

**Not a replacement for human judgment** — Steward identifies when human judgment is required. It never substitutes for it.

---

## Architecture

### Crate Structure

```
steward/
├── crates/
│   ├── steward-core/        # DETERMINISTIC: No LLM calls
│   │   ├── contract/        # Parsing, validation, schema
│   │   ├── lenses/          # Five independent evaluators
│   │   ├── synthesizer.rs   # Strict policy machine
│   │   └── evidence.rs      # Evidence linking
│   │
│   ├── steward-runtime/     # OPTIONAL: LLM orchestration
│   │   ├── providers/       # LLM provider abstraction
│   │   ├── agents/          # Governance agents
│   │   ├── resilience/      # Circuit breaker, budget
│   │   └── evidence/        # Evidence validation
│   │
│   └── steward-cli/         # Binary CLI
│
└── bindings/
    ├── python/              # PyO3 bindings
    └── node/                # napi-rs bindings
```

### Critical Boundary

| Crate | LLM Calls? | Responsibility |
|-------|------------|----------------|
| steward-core | **NO** | Contract parsing, lens evaluation, synthesis, evidence linking |
| steward-runtime | Yes (optional) | LLM-based evaluation when rules need interpretation |
| steward-cli | No | Command-line interface |

**steward-core must never make LLM calls.** This is a hard boundary.

### Terminology

| Layer | Term |
|-------|------|
| System | Steward |
| Runtime | Governance Runtime |
| Agents | Governance Agents |
| Evaluation | Governance Evaluation |
| Outcome | Governance State |

---

## When to Use Steward

Use Steward when:

- **Automation affects people, money, access, or trust** — and you need to define when it must stop
- **You need explicit accountability** — who approved what, why, and who to contact when something goes wrong
- **Low confidence should surface humans, not guesses** — uncertainty is a valid signal, not a problem to hide
- **You want governance as testable data** — contracts that can be versioned, diffed, and validated
- **Compliance requires audit trails** — every BLOCKED cites rule IDs and evidence pointers
