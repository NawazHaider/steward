# @steward/core

Deterministic stewardship contract evaluation for AI outputs.

## Installation

```bash
npm install @steward/core
```

## Quick Start

```typescript
import { Contract, Output, evaluate, isBlocked, isEscalate } from '@steward/core';

// Load a contract
const contract = Contract.fromYamlFile('contract.yaml');

// Create output to evaluate
const output = Output.text('The AI generated this response');

// Evaluate
const result = evaluate(contract, output);

// Check the result
if (isBlocked(result)) {
  console.log(`BLOCKED: ${result.state.violation?.ruleId}`);
  console.log(`Contact: ${result.state.violation?.accountableHuman}`);
} else if (isEscalate(result)) {
  console.log(`ESCALATE: ${result.state.decisionPoint}`);
} else {
  console.log(`PROCEED: ${result.state.summary}`);
}
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
- `evaluateWithContext(contract, output, context?, metadata?)` - Evaluate with optional context

### Core Classes

- `Contract.fromYaml(yaml)` - Create a contract from YAML string
- `Contract.fromYamlFile(path)` - Create a contract from YAML file
- `Output.text(content)` - Create a text output

### Helper Functions

- `isProceed(result)` - Check if result is Proceed
- `isEscalate(result)` - Check if result is Escalate
- `isBlocked(result)` - Check if result is Blocked

### Enums

- `LensType` - The five evaluation lenses
- `RuleResult` - Rule evaluation results (Satisfied, Violated, Uncertain, NotApplicable)
- `EvidenceSource` - Source of evidence (Contract, Output, Context, Metadata)

## Development

### Building from Source

```bash
# Install dependencies
npm install

# Build the native binding
npm run build

# Run tests
npm test
```

## License

MIT License - see LICENSE file for details.
