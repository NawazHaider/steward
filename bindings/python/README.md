# Steward Python Bindings

Deterministic stewardship contract evaluation for AI outputs.

## Installation

```bash
pip install steward
```

## Quick Start

```python
from steward import Contract, Output, evaluate

# Load a contract
contract = Contract.from_yaml_file("contract.yaml")

# Create output to evaluate
output = Output.text("The AI generated this response")

# Evaluate
result = evaluate(contract, output)

# Check the result
if result.is_blocked():
    print(f"BLOCKED: {result.violation.rule_id}")
    print(f"Contact: {result.violation.accountable_human}")
elif result.is_escalate():
    print(f"ESCALATE: {result.decision_point}")
else:
    print(f"PROCEED: {result.summary}")
```

## Contract Example

```yaml
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Customer Support"
intent:
  purpose: "Provide helpful customer support"
boundaries:
  may_do_autonomously:
    - id: "A1"
      rule: "Answer product questions"
  invalidated_by:
    - id: "B1"
      rule: "Customer PII exposed in response"
accountability:
  answerable_human: "support@example.com"
  escalation_path:
    - "Tier 1 Support"
    - "Manager"
```

## API Reference

### Core Functions

- `evaluate(contract, output)` - Evaluate an output against a contract
- `evaluate_with_context(contract, output, context=None, metadata=None)` - Evaluate with optional context

### Core Types

- `Contract` - A stewardship contract loaded from YAML
- `Output` - AI-generated output to evaluate
- `EvaluationResult` - The result of evaluation

### Result States

- `result.is_proceed()` - All conditions met, automation may continue
- `result.is_escalate()` - Uncertainty detected, human judgment required
- `result.is_blocked()` - Boundary violated, automation must halt

## Development

### Building from Source

```bash
# Install maturin
pip install maturin

# Build and install in development mode
cd bindings/python
maturin develop

# Run tests
pytest tests/
```

## License

MIT License - see LICENSE file for details.
