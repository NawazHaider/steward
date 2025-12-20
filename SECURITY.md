# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Steward, please report it responsibly:

1. **Do not** open a public issue
2. Email the maintainers directly or use GitHub's private vulnerability reporting feature
3. Include a detailed description of the vulnerability
4. Provide steps to reproduce if possible

We will respond within 48 hours and work with you to understand and address the issue.

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| Latest  | Yes                |

## Security Considerations

### Deterministic Evaluation

Steward's core evaluation engine (`steward-core`) is designed to be deterministic and isolated:

- **No LLM calls** in core evaluation — all judgment is based on human-authored contracts
- **No network calls** during evaluation
- **No file system access** except through explicit parameters
- **No random number generation** — same input always produces same output

### Contract Security

Steward contracts define governance rules but do not execute code:

- Contracts are **data** (YAML/JSON), not executable code
- Contract parsing validates against a strict JSON Schema
- Unknown fields are rejected by default — no silent field injection
- Rule IDs must match expected patterns

### Input Validation

All inputs are validated before processing:

- Contract schema validation against `spec/contract.schema.json`
- Output content length limits (configurable)
- Evidence pointers validated for format and bounds
- Metadata keys validated for allowed characters

### Optional LLM Runtime

If using `steward-runtime` for LLM-assisted evaluation:

- LLM calls are **opt-in** and clearly separated from core
- Provider credentials are never logged
- LLM responses are validated before use
- Fallback to deterministic evaluation on LLM failure

## Best Practices for Deployment

1. **Validate contracts** before deployment with `steward contract validate`
2. **Review accountable_human** fields to ensure correct escalation paths
3. **Version control contracts** — they are governance documents
4. **Audit trail** — log all BLOCKED and ESCALATE results with timestamps
5. **Restrict contract authoring** — only authorized stewards should modify contracts

## Data Handling

- Steward does **not** log output content by default
- Evaluation results contain evidence pointers, not full content copies
- No telemetry collected
- Full offline operation supported

## Threat Model

Steward assumes:

- **Trusted contracts**: Contracts are authored by authorized humans
- **Untrusted outputs**: AI outputs are evaluated, not trusted
- **Trusted evaluator**: The Steward binary itself is trusted

Steward does **not** protect against:

- Malicious contract authors (contracts are governance — author access should be controlled)
- Side-channel attacks on the evaluation process
- Denial of service via extremely large inputs (use resource limits)
