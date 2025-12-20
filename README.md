<p align="center">
  <img src="assets/logo.png" alt="Steward" width="200">
</p>

# Steward

**Stewardship contracts for AI systems.**

Steward is a deterministic evaluation engine that answers the questions stewards ask:
- Should this have happened at all?
- Where must this stop?
- Who answers for this?

```bash
cat response.txt | steward evaluate --contract contract.yaml
# Exit 0: PROCEED | Exit 1: ESCALATE | Exit 2: BLOCKED
```

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

## Core Concepts

### Stewardship Contracts

A contract is a human-authored document that defines intent, boundaries, accountability, and acceptance criteria **before** AI runs:

```yaml
# contract.yaml
contract_version: "1.0"
schema_version: "2025-12-20"

name: "Customer Support Response"
description: "Stewardship contract for AI-generated support responses"

intent:
  purpose: "Generate helpful responses to customer inquiries"
  never_optimize_away:
    - id: "N1"
      rule: "Human escalation path"
    - id: "N2"
      rule: "Customer dignity"

boundaries:
  may_do_autonomously:
    - id: "A1"
      rule: "Answer factual questions about products"
    - id: "A2"
      rule: "Provide order status from verified data"

  must_escalate_when:
    - id: "E1"
      rule: "Customer explicitly requests human agent"
    - id: "E2"
      rule: "Billing dispute over $100"

  invalidated_by:
    - id: "B1"
      rule: "Customer PII exposed in response"
    - id: "B2"
      rule: "Medical, legal, or financial advice given"

accountability:
  approved_by: "Support Team Lead"
  answerable_human: "support-escalation@company.com"
  escalation_path:
    - "Tier 1 Support Agent"
    - "Support Team Lead"
    - "Legal (if compliance-related)"

acceptance:
  fit_criteria:
    - id: "F1"
      rule: "Addresses the customer's actual question"
    - id: "F2"
      rule: "Cites sources when making claims"

  dignity_check:
    - id: "D1"
      rule: "Does not dismiss customer concerns"
    - id: "D2"
      rule: "Preserves clear path to human help"
```

Contracts are **data**, not config, not prompts. This externalizes judgment, makes governance testable, and prevents policy drift.

### The Three States

| State | Meaning | Action |
|-------|---------|--------|
| **PROCEED** | All conditions met | Log and continue |
| **ESCALATE** | Uncertainty detected | Present decision to human |
| **BLOCKED** | Boundary violated | Stop immediately, notify accountable human |

**Resolution rules (not configurable):**
- Any lens returns BLOCKED → final state is **BLOCKED**
- Else any lens returns ESCALATE → final state is **ESCALATE**
- Else → **PROCEED**

Non-configurable policy is a feature. This is governance machinery, not a tuning toy.

### The Five Lenses

Each lens asks one stewardship question. They evaluate **independently**—lenses don't debate or persuade each other. Synthesis is policy, not intelligence.

| Lens | Question |
|------|----------|
| **Dignity & Inclusion** | Does this disempower people? |
| **Boundaries & Safety** | Does this respect defined scope and stop conditions? |
| **Restraint & Privacy** | Does this expose what should be protected? |
| **Transparency & Contestability** | Can the human understand and challenge this? |
| **Accountability & Ownership** | Who approved this, and who can stop it? |

---

## Usage

### CLI

```bash
# Evaluate output against contract
steward evaluate --contract contract.yaml --output response.txt

# Pipe from stdin
cat response.txt | steward evaluate --contract contract.yaml

# JSON output
steward evaluate --contract contract.yaml --output response.txt --format json

# Validate contract schema
steward contract validate contract.yaml
```

Exit codes: `0` PROCEED, `1` ESCALATE, `2` BLOCKED, `3` Error

### Rust

```rust
use steward_core::{Contract, Output, evaluate};

let contract = Contract::from_yaml_file("contract.yaml")?;
let output = Output::text("Your order #12345 shipped yesterday.");
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

### Python

```python
from steward import Contract, Output, evaluate

contract = Contract.from_yaml_file("contract.yaml")
output = Output.text("Your order #12345 shipped yesterday.")
result = evaluate(contract, output)

if result.is_blocked():
    print(f"BLOCKED: {result.violation.rule_id}")
    print(f"Contact: {result.violation.accountable_human}")
elif result.is_escalate():
    print(f"ESCALATE: {result.decision_point}")
else:
    print(f"PROCEED: {result.summary}")
```

### TypeScript

```typescript
import { Contract, Output, evaluate } from '@anthropic/steward';

const contract = Contract.fromYamlFile('contract.yaml');
const output = Output.text('Your order #12345 shipped yesterday.');
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

## Example Outputs

### PROCEED

```json
{
  "state": {
    "type": "Proceed",
    "summary": "Output addresses customer question with verified order data. All contract conditions satisfied."
  },
  "lens_findings": {
    "dignity_inclusion": {
      "state": "Pass",
      "rules_evaluated": [
        { "rule_id": "D1", "result": "Satisfied", "rationale": "Response acknowledges concern directly" },
        { "rule_id": "D2", "result": "Satisfied", "rationale": "Human contact option preserved in closing" }
      ],
      "confidence": 0.89
    },
    "boundaries_safety": {
      "state": "Pass",
      "rules_evaluated": [
        { "rule_id": "A2", "result": "Satisfied", "rationale": "Order status from verified system data" }
      ],
      "confidence": 0.92
    },
    "restraint_privacy": {
      "state": "Pass",
      "rules_evaluated": [
        { "rule_id": "B1", "result": "Satisfied", "rationale": "No PII in response" }
      ],
      "confidence": 0.95
    },
    "transparency_contestability": {
      "state": "Pass",
      "rules_evaluated": [
        { "rule_id": "F2", "result": "Satisfied", "rationale": "Source cited: order system" }
      ],
      "confidence": 0.88
    },
    "accountability_ownership": {
      "state": "Pass",
      "rules_evaluated": [],
      "confidence": 0.91
    }
  },
  "confidence": 0.89,
  "evaluated_at": "2025-12-20T14:32:00Z"
}
```

### ESCALATE

```json
{
  "state": {
    "type": "Escalate",
    "uncertainty": "Customer message contains 'I'm really frustrated' — matches P1 (must_pause_when: customer expresses frustration)",
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
    },
    "dignity_inclusion": { "state": "Pass", "confidence": 0.85 },
    "restraint_privacy": { "state": "Pass", "confidence": 0.92 },
    "transparency_contestability": { "state": "Pass", "confidence": 0.88 },
    "accountability_ownership": { "state": "Pass", "confidence": 0.90 }
  },
  "confidence": 0.61,
  "evaluated_at": "2025-12-20T14:35:00Z"
}
```

### BLOCKED

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
          "rule_text": "Customer PII exposed in response",
          "result": "Violated",
          "evidence": [
            {
              "claim": "Email address 'john.doe@email.com' found in plaintext",
              "source": "Output",
              "pointer": "output.content[142:168]"
            }
          ],
          "rationale": "Output contains customer email in plaintext at position 142-168"
        }
      ],
      "confidence": 0.98
    },
    "dignity_inclusion": { "state": "Pass", "confidence": 0.87 },
    "boundaries_safety": { "state": "Pass", "confidence": 0.91 },
    "transparency_contestability": { "state": "Pass", "confidence": 0.89 },
    "accountability_ownership": { "state": "Pass", "confidence": 0.90 }
  },
  "confidence": 0.98,
  "evaluated_at": "2025-12-20T14:38:00Z"
}
```

---

## When to Use Steward

Use Steward when:

- **Automation affects people, money, access, or trust** — and you need to define when it must stop
- **You need explicit accountability** — who approved what, why, and who to contact when something goes wrong
- **Low confidence should surface humans, not guesses** — uncertainty is a valid signal, not a problem to hide
- **You want governance as testable data** — contracts that can be versioned, diffed, and validated
- **Compliance requires audit trails** — every BLOCKED cites rule IDs and evidence pointers

---

## What Steward Is Not

**Not an LLM-as-a-judge** — Steward doesn't ask a model "Is this good?" Criteria are human-authored and explicit. Models that grade themselves hide accountability.

**Not a quality scorer** — Numeric scores hide boundary violations and encourage threshold gaming. Steward returns states, not numbers.

**Not a recommendation engine** — ESCALATE presents options to humans without ranking them. Steward surfaces decisions, it doesn't make them.

**Not a replacement for human judgment** — Steward identifies when human judgment is required. It never substitutes for it.

---

## Design Principles

1. **Deterministic** — Same contract + same output = same result. Always.
2. **Traceable** — Every BLOCKED cites a rule ID and evidence pointer.
3. **Honest** — Low confidence triggers ESCALATE, not guessing.
4. **Upstream** — Contracts are defined before AI runs, not after.
5. **Human-centered** — BLOCKED identifies the accountable human. ESCALATE presents options, not recommendations.

---

## Installation

```bash
# Rust (from source)
cargo install steward-cli

# Python
pip install steward

# Node.js
npm install @anthropic/steward
```

---

## License

MIT

---

<p align="center">
  Built by <a href="https://agenisea.ai">Agenisea AI™</a> 🪼
</p>