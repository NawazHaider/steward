<p align="center">
  <img src="assets/logo.png" alt="Steward" width="200">
</p>

# Steward

Governance calculus for AI systems

Steward is a governance calculus: human-authored contracts define criteria, parallel lenses evaluate independently, and a deterministic synthesizer reduces findings into PROCEED / ESCALATE / BLOCKED with conservative confidence and evidence-backed accountability.

Steward is a governance primitive, not an evaluation model.
It provides infrastructure for enforcing human-defined boundaries in agentic AI systems.

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

**The honesty rule:** If overall confidence falls below 0.4 and no lens is BLOCKED, the result is forced to ESCALATE. Uncertainty is a governance signal, not an error to hide.

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

### Julia

```julia
using Steward

result = Steward.evaluate(contract_yaml, "Your order #12345 shipped yesterday.")
if result["state"] == "blocked"
    println("BLOCKED: ", result["violation"]["rule_id"])
end
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
├── 5 independent lenses                  ├── Provider registry + factory
│   ├── Dignity & Inclusion               ├── Parallel orchestration
│   ├── Boundaries & Safety               ├── Circuit breaker + budgets
│   ├── Restraint & Privacy               └── Fallback chain + resilience
│   ├── Transparency & Contestability
│   └── Accountability & Ownership        steward-bindings-core (FFI layer)
├── Synthesizer (strict policy)           ├── IR types for marshalling
└── Evidence linking                      └── Shared test fixtures

                                          Language bindings (thin wrappers)
                                          ├── Python (PyO3)
                                          ├── Node.js (napi-rs)
                                          └── Julia (C ABI)
```

The core is always deterministic. Lenses evaluate independently—they don't debate or persuade each other. Synthesis is policy, not intelligence.

The runtime adds optional LLM-assisted evaluation with circuit breakers, token budgets, and fallback strategies. Bindings are thin FFI wrappers—all semantics live in `steward-core`.

---

## What Steward Is Not

**Not an LLM-as-a-judge** — Steward doesn't ask a model "Is this good?" Criteria are human-authored and explicit. Models that grade themselves hide accountability.

**Not a quality scorer** — Numeric scores hide boundary violations and encourage threshold gaming. Steward returns states, not numbers.

**Not a recommendation engine** — ESCALATE presents options to humans without ranking them. Steward surfaces decisions, it doesn't make them.

**Not a replacement for human judgment** — Steward identifies when human judgment is required. It never substitutes for it.

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
# Rust (from source)
cargo install --path crates/steward-cli

# Python (from source)
cd bindings/python && maturin develop

# Node.js (from source)
cd bindings/node && npm run build

# Julia (from source)
cargo build --release -p steward-julia
# Then load libsteward_julia.dylib via ccall
```

*Package registry publishing (crates.io, PyPI, npm) coming soon.*

---

## Context / Related Work

### Policy and Governance Context

The EU Cyber Resilience Act (CRA) introduces the concept of "open-source software stewards" as legal entities responsible for supporting the cybersecurity of FOSS used in commercial products. This governance model—and its implications for fairness, accountability, and transparency—was examined at [ACM FAccT 2025](https://dl.acm.org/doi/10.1145/3715275.3732032) as a novel but still ambiguous approach to software governance.

### How Steward Differs

Steward is not a policy role, certification body, or institutional steward.

It is a **runtime governance primitive**.

Where policy frameworks define *who* is responsible, Steward defines *how* responsibility is enforced at execution time:

- Human-authored contracts, not implicit norms
- Deterministic evaluation, not probabilistic judgment
- Evidence-backed verdicts, not advisory signals
- Explicit `accountable_human` fields, not diffuse responsibility

Steward is designed to close the accountability gap that informal or institutional stewardship models may leave open—by enforcing governance *inside* agentic AI systems, not around them.

### Relevant Sources

| Source | Description |
|--------|-------------|
| [FAccT'25: Stewardship in FOSS Governance](https://dl.acm.org/doi/10.1145/3715275.3732032) | Tridgell & Singh examine "software stewards" under the EU CRA |
| [EU Cyber Resilience Act](https://digital-strategy.ec.europa.eu/en/policies/cyber-resilience-act) | Regulation introducing cybersecurity requirements for digital products |
| [Responsible AI Pattern Catalogue](https://dl.acm.org/doi/10.1145/3626234) | ACM collection of best practices for AI governance |
| [Closing the AI Accountability Gap](https://dl.acm.org/doi/10.1145/3351095.3372873) | Raji et al. on internal algorithmic auditing frameworks |

---

## License

MIT

---

<p align="center">
  Built by <a href="https://agenisea.ai">Agenisea AI™</a> 🪼
</p>
