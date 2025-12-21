<p align="center">
  <img src="assets/logo.png" alt="Steward" width="200">
</p>

# Steward

**Governance calculus for AI systems.**

Steward is governance calculus: contracts define criteria, lenses evaluate independently, and a deterministic synthesizer reduces findings into PROCEED/ESCALATE/BLOCKED with conservative confidence and evidence-backed accountability.

```bash
cat response.txt | steward evaluate --contract contract.yaml
# Exit 0: PROCEED | Exit 1: ESCALATE | Exit 2: BLOCKED
```

```
Contract + Output → [5 Lenses in parallel] → Synthesizer → PROCEED | ESCALATE | BLOCKED
                                                              │
                                    confidence = min(all lenses), evidence required
```

---

## The Three States

| State | Meaning | Action |
|-------|---------|--------|
| **PROCEED** | All conditions met | Log and continue |
| **ESCALATE** | Uncertainty detected | Present decision to human |
| **BLOCKED** | Boundary violated | Stop immediately, notify accountable human |

---

## Quick Start

### CLI

```bash
# Evaluate output against contract
steward evaluate --contract contract.yaml --output response.txt

# JSON output
steward evaluate --contract contract.yaml --output response.txt --format json
```

Exit codes: `0` PROCEED, `1` ESCALATE, `2` BLOCKED, `3` Error

### Rust

```rust
use steward_core::{Contract, Output, evaluate};

let contract = Contract::from_yaml_file("contract.yaml")?;
let output = Output::text("Your order #12345 shipped yesterday.");
let result = evaluate(&contract, &output)?;

match result.state {
    State::Proceed { .. } => println!("OK"),
    State::Escalate { decision_point, .. } => println!("ESCALATE: {}", decision_point),
    State::Blocked { violation } => println!("BLOCKED: {}", violation.rule_id),
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
```

### TypeScript

```typescript
import { Contract, Output, evaluate, isBlocked } from '@steward/core';

const contract = Contract.fromYamlFile('contract.yaml');
const output = Output.text('Your order #12345 shipped yesterday.');
const result = evaluate(contract, output);

if (isBlocked(result.state.stateType)) {
  console.log(`BLOCKED: ${result.state.violation.ruleId}`);
}
```

---

## Example Contract

```yaml
name: "Customer Support Response"

intent:
  purpose: "Generate helpful responses to customer inquiries"
  never_optimize_away:
    - id: "N1"
      rule: "Human escalation path"

boundaries:
  may_do_autonomously:
    - id: "A1"
      rule: "Answer factual questions about products"
  must_escalate_when:
    - id: "E1"
      rule: "Customer explicitly requests human agent"
  invalidated_by:
    - id: "B1"
      rule: "Customer PII exposed in response"

accountability:
  answerable_human: "support-escalation@company.com"
```

---

## Domain Packs

Ready-to-use contracts for regulated industries with compliance mapping:

| Domain | Contract | Regulations |
|--------|----------|-------------|
| **Healthcare** | [healthcare.yaml](contracts/healthcare.yaml) | HIPAA, GINA, 42 CFR Part 2 |
| **Finance** | [finance.yaml](contracts/finance.yaml) | SEC Reg BI, Advisers Act, FINRA |
| **Legal** | [legal.yaml](contracts/legal.yaml) | ABA Model Rules, Privilege |
| **Education** | [education.yaml](contracts/education.yaml) | FERPA, COPPA, IDEA |
| **HR** | [hr.yaml](contracts/hr.yaml) | Title VII, ADA, EEOC AI Guidance |

See [Compliance Mapping](docs/compliance-mapping.md) for detailed regulatory coverage.

---

## Architecture

```
steward-core (deterministic, NO LLM)      steward-runtime (optional LLM)
├── 5 independent lenses                  ├── ProviderFactory/Registry
├── Synthesizer (strict policy)           ├── Parallel orchestration
└── Evidence linking                      └── Fallback chain + resilience
```

The core is always deterministic. The runtime adds optional LLM-assisted evaluation with circuit breakers, token budgets, and fallback strategies.

---

## Documentation

| Document | Description |
|----------|-------------|
| [Steward Design](docs/Steward.md) | Architecture, governance calculus, and lens specifications |
| [Blueprint Specs](docs/steward-blueprint-specs.md) | Authoritative specification |
| [Compliance Mapping](docs/compliance-mapping.md) | Regulatory requirements by domain |
| [Contract Schema](spec/contract.schema.json) | JSON Schema for contracts |

---

## Installation

```bash
# Rust
cargo install steward-cli

# Python
pip install steward

# Node.js
npm install @steward/core
```

---

## License

MIT

---

<p align="center">
  Built by <a href="https://agenisea.ai">Agenisea AI™</a> 🪼
</p>
