# Steward: Complete Design Blueprint

## Document Purpose

This document is the authoritative specification for Steward. It defines what Steward is, what it is not, how it works, and how to build it. Any implementation that deviates from this document is incorrect.

---

# Part I: Philosophy & Grounding

## 1.1 What is Stewardship?

A steward is not someone who executes the system. A steward is someone who decides what the system is allowed to do, when it must stop, and who is accountable when it acts.

**The shortest definition:**
> A steward designs and maintains the conditions under which automation operates responsibly. They don't push the buttons. They decide which buttons exist at all.

**What stewards do:**
1. **Define intent before execution** — Why should this happen at all?
2. **Set boundaries for autonomy** — What may it do? Where must it stop?
3. **Encode accountability** — Who answers when something goes wrong?
4. **Design evaluation, not just outputs** — Did it work for the right reasons?
5. **Protect human dignity** — Does this disempower people?
6. **Adapt as context changes** — Systems decay; stewards maintain.

**What stewards do not do:**
- Ship the most features
- Write the most code
- Optimize for speed over judgment
- Work downstream after damage is done

---

## 1.2 What is Steward (the software)?

Steward is a **stewardship instrument** that surfaces whether the conditions for responsible automation are being met.

Steward does not judge whether AI output is "good." It answers the questions stewards ask:
- Should this have happened at all?
- Where must this stop?
- Who answers for this?
- Did it work for the right reasons?
- Does this disempower people?

**Steward is:**
- A deterministic evaluation engine
- A contract enforcement mechanism
- An accountability surface
- A human decision support tool

**Steward is not:**
- An LLM-as-a-judge
- A quality scorer
- A recommendation engine
- A replacement for human judgment

---

## 1.3 The Core Distinction

| LLM-as-a-Judge | Steward |
|----------------|---------|
| Asks a model "Is this good?" | Defines what "good" means first, then evaluates against it |
| Model invents criteria | Criteria are human-authored, explicit, stable |
| Single chain of thought | Multiple independent lenses |
| Produces opinion | Produces traceable evidence |
| Overconfident by default | Admits uncertainty as valid signal |
| Downstream | Upstream |

---

# Part II: What Steward Does

## 2.1 Core Function

Steward takes:
1. **A stewardship contract** (human-defined conditions)
2. **An output** (what the AI produced)
3. **Context** (what the AI had access to)

And returns:
1. **A state**: PROCEED, ESCALATE, or BLOCKED
2. **Lens findings**: What each lens observed
3. **Evidence**: Citations from contract and output
4. **Confidence**: How well-supported the finding is

---

## 2.2 The Three States

| State | Meaning | Action Required |
|-------|---------|-----------------|
| **PROCEED** | All conditions met. Automation may continue. | Log and continue. |
| **ESCALATE** | Uncertainty detected. Human judgment required. | Present decision point to human. |
| **BLOCKED** | Boundary violated. Automation must halt. | Stop immediately. Notify accountable human. |

**State resolution rules (strict):**
- If ANY lens returns BLOCKED → final state is BLOCKED
- Else if ANY lens returns ESCALATE → final state is ESCALATE
- Else → final state is PROCEED

These rules are not configurable. They are the policy.

---

## 2.3 The Five Lenses

Each lens asks one stewardship question. Lenses evaluate independently and in parallel.

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

**Contract rules this lens evaluates:**
- `acceptance.dignity_check[]`
- `boundaries.must_escalate_when[]` (dignity-related)

---

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

**Contract rules this lens evaluates:**
- `boundaries.may_do_autonomously[]`
- `boundaries.must_pause_when[]`
- `boundaries.must_escalate_when[]`
- `boundaries.invalidated_by[]`

---

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

**Contract rules this lens evaluates:**
- `boundaries.invalidated_by[]` (privacy-related)
- `intent.never_optimize_away[]` (privacy-related)

---

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

**Contract rules this lens evaluates:**
- `acceptance.fit_criteria[]` (transparency-related)

---

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

**Contract rules this lens evaluates:**
- `accountability.approved_by`
- `accountability.answerable_human`
- `accountability.escalation_path[]`

---

## 2.4 Confidence Calculation

Confidence reflects evidence quality, not certainty in verdict.

**Formula:**
```
confidence = min(lens_confidences)
```

**Lens confidence is reduced when:**
- Evidence is sparse (few citations)
- Rules are ambiguous
- Output is unclear or contradictory
- Metadata is missing

**Confidence thresholds:**
- `>= 0.7`: High confidence
- `0.4 - 0.7`: Moderate confidence
- `< 0.4`: Low confidence (triggers ESCALATE if no BLOCKED)

---

# Part III: What Steward Does Not Do

## 3.1 Explicit Non-Goals

| Steward Does NOT | Reason |
|------------------|--------|
| Generate alternatives | It evaluates, it does not create |
| Recommend actions | It surfaces options, humans decide |
| Learn autonomously | Contract updates require human approval |
| Rate quality | It assesses alignment with conditions |
| Replace judgment | It structures judgment already encoded |
| Make LLM calls in core | Core is deterministic |
| Invent criteria | Criteria come from contracts only |
| Guess when uncertain | It returns ESCALATE instead |

---

## 3.2 Anti-Patterns (What Bad Looks Like)

**Bad: Vague verdicts**
```
"This output seems mostly okay but could be better."
```

**Good: Precise verdicts**
```
State: BLOCKED
Violation: B3 (boundaries.invalidated_by[2])
Evidence: Output contains email address at position 47-72
Accountable: security@company.com
```

**Bad: Invented criteria**
```
"The tone feels too aggressive for a support response."
```

**Good: Contract-cited criteria**
```
Rule: acceptance.dignity_check[1] = "Does not dismiss customer concerns"
Finding: Output uses phrase "That's not how it works" — matches dismissive pattern
Evidence: Line 3, characters 0-24
```

**Bad: High confidence with sparse evidence**
```
Confidence: 0.95
Evidence: []
```

**Good: Confidence reflects evidence**
```
Confidence: 0.42
Evidence: [1 citation]
Note: Limited evidence available; escalation recommended
```

---

# Part IV: Data Specifications

## 4.1 Stewardship Contract Schema

```yaml
# contracts/general.yaml
contract_version: "1.0"
schema_version: "2025-12-20"
policy_pack: ["general"]

name: "Customer Support Response"
description: "Stewardship contract for AI-generated customer support responses"

intent:
  purpose: "Generate helpful responses to customer inquiries"
  optimizing_for:
    - "Resolution of customer issue"
    - "Customer satisfaction"
    - "Accurate information"
  never_optimize_away:
    - id: "N1"
      rule: "Human escalation path"
    - id: "N2"
      rule: "Accurate information over speed"
    - id: "N3"
      rule: "Customer dignity"

boundaries:
  may_do_autonomously:
    - id: "A1"
      rule: "Answer factual questions about products"
    - id: "A2"
      rule: "Provide order status from verified data"
    - id: "A3"
      rule: "Explain policies with citations"

  must_pause_when:
    - id: "P1"
      rule: "Customer expresses frustration or anger"
    - id: "P2"
      rule: "Question involves account security"

  must_escalate_when:
    - id: "E1"
      rule: "Legal or compliance topic detected"
    - id: "E2"
      rule: "Customer explicitly requests human agent"
    - id: "E3"
      rule: "Billing dispute over $100"

  invalidated_by:
    - id: "B1"
      rule: "Customer PII exposed in response"
    - id: "B2"
      rule: "Medical, legal, or financial advice given"
    - id: "B3"
      rule: "System cannot verify accuracy of claim"

accountability:
  approved_by: "Support Team Lead"
  answerable_human: "support-escalation@company.com"
  escalation_path:
    - "Tier 1 Support Agent"
    - "Support Team Lead"
    - "Legal (if compliance-related)"
  review_cadence: "quarterly"

acceptance:
  fit_criteria:
    - id: "F1"
      rule: "Addresses the customer's actual question"
    - id: "F2"
      rule: "Accurate based on provided context"
    - id: "F3"
      rule: "Provides clear next steps"
    - id: "F4"
      rule: "Cites sources when making claims"

  dignity_check:
    - id: "D1"
      rule: "Does not dismiss customer concerns"
    - id: "D2"
      rule: "Does not pressure customer toward automated resolution"
    - id: "D3"
      rule: "Preserves clear path to human help"
```

---

## 4.2 Evaluation Request Schema

```rust
pub struct EvaluationRequest {
    pub contract: Contract,
    pub output: Output,
    pub context: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, String>>,
}

pub struct Output {
    pub content_type: ContentType,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

pub enum ContentType {
    Text,
    // Future: Image, Audio, Code
}
```

---

## 4.3 Evaluation Result Schema

```rust
pub struct EvaluationResult {
    pub state: State,
    pub lens_findings: LensFindings,
    pub confidence: f64,
    pub evaluated_at: DateTime<Utc>,
}

pub enum State {
    Proceed {
        summary: String,
    },
    Escalate {
        uncertainty: String,
        decision_point: String,
        options: Vec<String>,
    },
    Blocked {
        violation: BoundaryViolation,
    },
}

pub struct BoundaryViolation {
    pub lens: LensType,
    pub rule_id: String,           // e.g., "B1"
    pub rule_text: String,         // Full rule from contract
    pub evidence: Vec<Evidence>,
    pub accountable_human: String,
}

pub struct LensFindings {
    pub dignity_inclusion: LensFinding,
    pub boundaries_safety: LensFinding,
    pub restraint_privacy: LensFinding,
    pub transparency_contestability: LensFinding,
    pub accountability_ownership: LensFinding,
}

pub struct LensFinding {
    pub lens: LensType,
    pub question_asked: String,
    pub state: LensState,
    pub rules_evaluated: Vec<RuleEvaluation>,
    pub confidence: f64,
}

pub enum LensState {
    Pass,
    Escalate { reason: String },
    Blocked { violation: String },
}

pub struct RuleEvaluation {
    pub rule_id: String,
    pub rule_text: String,
    pub result: RuleResult,
    pub evidence: Vec<Evidence>,
    pub rationale: String,
}

pub enum RuleResult {
    Satisfied,
    Violated,
    Uncertain,
    NotApplicable,
}

pub struct Evidence {
    pub claim: String,
    pub source: EvidenceSource,
    pub pointer: String,  // Path to location (e.g., "output.content[47:72]")
}

pub enum EvidenceSource {
    Contract,
    Output,
    Context,
    Metadata,
}
```

---

## 4.4 Example Outputs

### PROCEED Example
```json
{
  "state": {
    "type": "Proceed",
    "summary": "Output addresses customer question, provides accurate order status with source citation, and maintains clear escalation path. All contract conditions satisfied."
  },
  "lens_findings": {
    "dignity_inclusion": {
      "state": "Pass",
      "rules_evaluated": [
        {"rule_id": "D1", "result": "Satisfied", "evidence": []},
        {"rule_id": "D2", "result": "Satisfied", "evidence": []},
        {"rule_id": "D3", "result": "Satisfied", "evidence": []}
      ],
      "confidence": 0.89
    },
    "boundaries_safety": {
      "state": "Pass",
      "rules_evaluated": [],
      "confidence": 0.92
    }
  },
  "confidence": 0.89,
  "evaluated_at": "2025-12-20T14:32:00Z"
}
```

### ESCALATE Example
```json
{
  "state": {
    "type": "Escalate",
    "uncertainty": "Customer message contains phrase 'I'm really frustrated' which matches P1 (must_pause_when: customer expresses frustration)",
    "decision_point": "Should automation continue or should a human agent take over?",
    "options": [
      "Continue with automated response — frustration is mild and question is straightforward",
      "Transfer to human agent — honor the pause condition strictly",
      "Respond with empathy acknowledgment, then offer human transfer option"
    ]
  },
  "lens_findings": {
    "boundaries_safety": {
      "state": "Escalate",
      "rules_evaluated": [
        {
          "rule_id": "P1",
          "rule_text": "Customer expresses frustration or anger",
          "result": "Uncertain",
          "evidence": [
            {
              "claim": "Customer frustration detected",
              "source": "Context",
              "pointer": "context[0][0:24]"
            }
          ],
          "rationale": "Phrase 'I'm really frustrated' matches pause condition. Severity unclear."
        }
      ],
      "confidence": 0.61
    }
  },
  "confidence": 0.61,
  "evaluated_at": "2025-12-20T14:35:00Z"
}
```

### BLOCKED Example
```json
{
  "state": {
    "type": "Blocked",
    "violation": {
      "lens": "restraint_privacy",
      "rule_id": "B1",
      "rule_text": "Customer PII exposed in response",
      "evidence": [
        {
          "claim": "Email address exposed",
          "source": "Output",
          "pointer": "output.content[142:168]"
        }
      ],
      "accountable_human": "support-escalation@company.com"
    }
  },
  "lens_findings": {
    "restraint_privacy": {
      "state": "Blocked",
      "rules_evaluated": [
        {
          "rule_id": "B1",
          "result": "Violated",
          "evidence": [],
          "rationale": "Output contains customer email 'john.doe@email.com' in plaintext at position 142-168"
        }
      ],
      "confidence": 0.98
    }
  },
  "confidence": 0.98,
  "evaluated_at": "2025-12-20T14:38:00Z"
}
```

---

# Part V: Architecture

## 5.1 Crate Structure

```
steward/
├── Cargo.toml                    # Workspace root
│
├── crates/
│   ├── steward-core/             # DETERMINISTIC: No LLM calls
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── contract/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── schema.rs     # Validation against spec
│   │   │   │   └── parser.rs     # YAML/JSON parsing
│   │   │   ├── lenses/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── dignity.rs
│   │   │   │   ├── boundaries.rs
│   │   │   │   ├── restraint.rs
│   │   │   │   ├── transparency.rs
│   │   │   │   └── accountability.rs
│   │   │   ├── synthesizer.rs    # Strict policy machine
│   │   │   ├── evidence.rs       # Evidence linking
│   │   │   └── types.rs          # Core types
│   │   └── Cargo.toml
│   │
│   ├── steward-runtime/          # OPTIONAL: LLM orchestration
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── orchestrator.rs   # Fan-out/fan-in with LLMs
│   │   │   └── providers/        # LLM provider adapters
│   │   └── Cargo.toml
│   │
│   └── steward-cli/              # Binary CLI
│       ├── src/main.rs
│       └── Cargo.toml
│
├── bindings/
│   ├── python/                   # PyO3 bindings
│   └── node/                     # napi-rs bindings
│
├── spec/                         # THE CONSTITUTION
│   ├── contract.schema.json
│   ├── result.schema.json
│   └── lenses.md
│
├── contracts/                    # Example contracts
│   ├── general.yaml
│   ├── healthcare.yaml
│   └── finance.yaml
│
└── tests/
    ├── golden/                   # Exact expected outputs
    └── properties/               # Invariant tests
```

---

## 5.2 Crate Responsibilities

| Crate | Responsibility | LLM Calls? |
|-------|----------------|------------|
| `steward-core` | Contract parsing, lens evaluation, synthesis, evidence linking | **NO** |
| `steward-runtime` | Optional LLM-based evaluation when rules need interpretation | Yes (optional) |
| `steward-cli` | Command-line interface, stdin pipe, output formatting | No |

---

## 5.3 Data Flow

```
                                 ┌─────────────────────────┐
                                 │   Stewardship Contract  │
                                 │   (human-authored)      │
                                 └───────────┬─────────────┘
                                             │
                                             ▼
┌─────────────┐    ┌─────────────────────────────────────────────┐
│   Output    │───▶│              steward-core                   │
│   (AI)      │    │                                             │
└─────────────┘    │  ┌─────────┐ ┌─────────┐ ┌─────────┐       │
                   │  │ Dignity │ │Boundary │ │Restraint│  ...  │
                   │  │  Lens   │ │  Lens   │ │  Lens   │       │
                   │  └────┬────┘ └────┬────┘ └────┬────┘       │
                   │       │           │           │             │
                   │       └───────────┼───────────┘             │
                   │                   ▼                         │
                   │           ┌───────────────┐                 │
                   │           │  Synthesizer  │                 │
                   │           │ (policy rules)│                 │
                   │           └───────┬───────┘                 │
                   │                   │                         │
                   └───────────────────┼─────────────────────────┘
                                       │
                                       ▼
                           ┌───────────────────────┐
                           │  Evaluation Result    │
                           │  PROCEED|ESCALATE|BLOCKED  │
                           └───────────────────────┘
```

---

# Part VI: Interfaces

## 6.1 CLI Interface

```bash
# Evaluate with contract file
steward evaluate --contract contract.yaml --output output.json

# Evaluate with stdin (most common real-world usage)
cat response.txt | steward evaluate --contract contract.yaml

# Output formats
steward evaluate --contract contract.yaml --output output.json --format json
steward evaluate --contract contract.yaml --output output.json --format text

# Explain mode (human-readable summary)
steward evaluate --contract contract.yaml --output output.json --explain

# Contract management
steward contract validate contract.yaml
steward contract show contract.yaml
steward contract list ./contracts/
```

**Exit codes:**
- `0`: PROCEED
- `1`: ESCALATE
- `2`: BLOCKED
- `3`: Error (invalid contract, parse failure, etc.)

---

## 6.2 Library Interface (Rust)

```rust
use steward_core::{Contract, Output, evaluate};

let contract = Contract::from_yaml_file("contract.yaml")?;
let output = Output::text("The AI generated this response");
let result = evaluate(&contract, &output)?;

match result.state {
    State::Proceed { summary } => println!("OK: {}", summary),
    State::Escalate { decision_point, options, .. } => {
        println!("ESCALATE: {}", decision_point);
        for opt in options {
            println!("  - {}", opt);
        }
    }
    State::Blocked { violation } => {
        println!("BLOCKED: {} ({})", violation.rule_id, violation.rule_text);
        println!("Contact: {}", violation.accountable_human);
    }
}
```

---

## 6.3 Library Interface (Python)

```python
from steward import Contract, Output, evaluate

contract = Contract.from_yaml_file("contract.yaml")
output = Output.text("The AI generated this response")
result = evaluate(contract, output)

if result.is_blocked():
    print(f"BLOCKED: {result.violation.rule_id}")
    print(f"Contact: {result.violation.accountable_human}")
elif result.is_escalate():
    print(f"ESCALATE: {result.decision_point}")
else:
    print(f"PROCEED: {result.summary}")
```

---

## 6.4 Library Interface (TypeScript)

```typescript
import { Contract, Output, evaluate } from '@steward/core';

const contract = Contract.fromYamlFile('contract.yaml');
const output = Output.text('The AI generated this response');
const result = evaluate(contract, output);

if (result.isBlocked()) {
  console.log(`BLOCKED: ${result.violation.ruleId}`);
  console.log(`Contact: ${result.violation.accountableHuman}`);
} else if (result.isEscalate()) {
  console.log(`ESCALATE: ${result.decisionPoint}`);
} else {
  console.log(`PROCEED: ${result.summary}`);
}
```

---

# Part VII: Testing Strategy

## 7.1 Golden Tests

Every contract + output pair has an exact expected JSON result. No variability.

```
tests/golden/
├── support-response-proceed.yaml     # Input
├── support-response-proceed.json     # Expected output (exact match)
├── support-response-blocked-pii.yaml
├── support-response-blocked-pii.json
├── support-response-escalate-frustration.yaml
├── support-response-escalate-frustration.json
```

**Test assertion:**
```rust
#[test]
fn test_support_response_proceed() {
    let input = load_yaml("tests/golden/support-response-proceed.yaml");
    let expected = load_json("tests/golden/support-response-proceed.json");
    let result = evaluate(&input.contract, &input.output);
    assert_eq!(result, expected); // Exact match
}
```

---

## 7.2 Property Tests

Invariants that must hold regardless of input:

| Property | Description |
|----------|-------------|
| **Determinism** | Same input always produces same output |
| **Metadata independence** | Adding irrelevant metadata does not flip verdict |
| **Lens independence** | Lens A cannot see Lens B's findings |
| **BLOCKED dominance** | Any BLOCKED lens → final BLOCKED |
| **Confidence bounds** | 0.0 <= confidence <= 1.0 |
| **Evidence requirement** | Every BLOCKED has at least one evidence citation |

```rust
#[test]
fn property_metadata_does_not_flip_verdict() {
    proptest!(|(contract: Contract, output: Output, extra_metadata: HashMap<String, String>)| {
        let result1 = evaluate(&contract, &output);

        let mut output_with_metadata = output.clone();
        output_with_metadata.metadata.extend(extra_metadata);
        let result2 = evaluate(&contract, &output_with_metadata);

        assert_eq!(result1.state.variant(), result2.state.variant());
    });
}
```

---

## 7.3 Schema Validation Tests

Every contract must validate against `spec/contract.schema.json`:

```rust
#[test]
fn test_all_example_contracts_valid() {
    let schema = load_schema("spec/contract.schema.json");
    for contract_file in glob("contracts/*.yaml") {
        let contract = load_yaml(&contract_file);
        assert!(schema.validate(&contract).is_ok(),
            "Contract {} failed validation", contract_file);
    }
}
```

---

# Part VIII: Roadmap

## Phase 1: Vertical Slice (Week 1-2)

**Goal:** Prove the architecture end-to-end with one lens.

**Deliverables:**
- [ ] `spec/contract.schema.json` — Contract JSON Schema
- [ ] `spec/result.schema.json` — Result JSON Schema
- [ ] `spec/lenses.md` — Lens specifications
- [ ] `contracts/general.yaml` — First complete contract
- [ ] `steward-core` crate with:
  - [ ] Contract parsing + validation
  - [ ] Boundaries lens (fully implemented)
  - [ ] Synthesizer (strict policy machine)
  - [ ] Evidence linking
- [ ] `steward-cli` with:
  - [ ] `steward evaluate --contract --output`
  - [ ] `--format json|text`
  - [ ] stdin pipe support
- [ ] Golden tests for Boundaries lens
- [ ] Property tests for invariants

**Success criteria:** `cat output.txt | steward evaluate --contract contracts/general.yaml` returns correct JSON.

---

## Phase 2: Complete Lenses (Week 3-4)

**Goal:** All five lenses implemented and tested.

**Deliverables:**
- [ ] Dignity & Inclusion lens
- [ ] Restraint & Privacy lens
- [ ] Transparency & Contestability lens
- [ ] Accountability & Ownership lens
- [ ] Golden tests for each lens
- [ ] Cross-lens integration tests
- [ ] Confidence calculation implementation

**Success criteria:** All example contracts evaluate correctly through all lenses.

---

## Phase 3: Python Bindings (Week 5)

**Goal:** `pip install steward` works.

**Deliverables:**
- [ ] PyO3 bindings in `bindings/python/`
- [ ] `maturin` build configuration
- [ ] Python package structure
- [ ] Python-specific tests
- [ ] PyPI publishing workflow

**Success criteria:** Python example in README works.

---

## Phase 4: TypeScript Bindings (Week 5-6)

**Goal:** `npm install @steward/core` works.

**Deliverables:**
- [ ] napi-rs bindings in `bindings/node/`
- [ ] TypeScript type definitions
- [ ] npm publishing workflow
- [ ] Node.js-specific tests

**Success criteria:** TypeScript example in README works.

---

## Phase 5: Runtime (Optional LLM) (Week 7+)

**Goal:** Enable LLM-assisted evaluation for complex rules.

**Deliverables:**
- [ ] `steward-runtime` crate
- [ ] Provider adapters (Claude, OpenAI, local)
- [ ] Hybrid mode: deterministic first, LLM only when needed
- [ ] Cost tracking and limits
- [ ] Fallback to deterministic on LLM failure

**Success criteria:** Complex rules that require interpretation work with configurable LLM backend.

---

## Phase 6: Domain Packs (Ongoing)

**Goal:** Ready-to-use contracts for common domains.

**Deliverables:**
- [ ] `contracts/healthcare.yaml`
- [ ] `contracts/finance.yaml`
- [ ] `contracts/legal.yaml`
- [ ] `contracts/education.yaml`
- [ ] `contracts/hr.yaml`
- [ ] Domain-specific lens extensions
- [ ] Compliance mapping documentation

---

# Part IX: Governance

## 9.1 Contract Versioning

Every contract must include:
- `contract_version`: Version of this specific contract (semver)
- `schema_version`: Version of the contract schema (date-based)
- `policy_pack`: Array of policy packs this contract extends

**Unknown fields fail by default.** Contracts are governance — drift kills them.

---

## 9.2 Schema Evolution

Breaking changes to `spec/contract.schema.json` require:
1. New `schema_version` date
2. Migration guide
3. Deprecation period for old version
4. Explicit opt-in to new schema in existing contracts

---

## 9.3 Lens Boundaries

Each lens document in `spec/lenses.md` specifies:
1. The stewardship question it answers
2. Which contract rules it may evaluate
3. Which contract rules it must NOT evaluate
4. Conditions for PASS, ESCALATE, BLOCKED

Lenses may not exceed their boundaries. A lens that evaluates rules outside its specification is a bug.

---

# Part X: Success Criteria

## What Good Looks Like

1. **Deterministic:** Same contract + same output = same result. Always.
2. **Traceable:** Every BLOCKED cites a rule_id and evidence pointer.
3. **Honest:** Low confidence triggers ESCALATE, not guessing.
4. **Upstream:** Contracts are defined before AI runs, not after.
5. **Human-centered:** BLOCKED identifies accountable human. ESCALATE presents options, not recommendations.
6. **Testable:** Golden tests prove exact behavior. Property tests prove invariants.
7. **Extensible:** New lenses and domain packs can be added without modifying core.
8. **Portable:** Works as CLI, Python library, and TypeScript library.

---

## What Failure Looks Like

1. Results vary between runs for same input.
2. BLOCKED without evidence or rule citation.
3. High confidence when evidence is sparse.
4. Contracts written after seeing AI output.
5. System makes recommendations instead of presenting options.
6. Tests that check "contains substring" instead of exact match.
7. Core crate makes LLM calls.
8. Works only in one language/platform.

---

# Appendix A: Glossary

| Term | Definition |
|------|------------|
| **Steward** | A human who defines conditions for responsible automation |
| **Stewardship Contract** | Human-authored document specifying intent, boundaries, accountability, and acceptance criteria |
| **Lens** | Independent evaluator that asks one stewardship question |
| **Synthesizer** | Component that aggregates lens findings into final state |
| **PROCEED** | All conditions met; automation may continue |
| **ESCALATE** | Uncertainty detected; human judgment required |
| **BLOCKED** | Boundary violated; automation must halt |
| **Rule ID** | Unique identifier for a rule in a contract (e.g., "B1", "D2") |
| **Evidence Pointer** | Path to location in output/context (e.g., "output.content[47:72]") |
| **Golden Test** | Test that asserts exact JSON output for given input |
| **Property Test** | Test that asserts invariants hold for any input |

---

# Appendix B: References

- Microsoft Responsible AI Principles: Fairness, Reliability & Safety, Privacy & Security, Inclusiveness, Transparency, Accountability
- Govern, Map, Measure, Manage framework
- PyO3 documentation: https://pyo3.rs
- napi-rs documentation: https://napi.rs

---

*End of Blueprint*

---

This is the complete specification. Every implementation decision should trace back to this document. If something isn't in here, ask before building.
