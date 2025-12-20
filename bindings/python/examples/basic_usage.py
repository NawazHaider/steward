#!/usr/bin/env python3
"""
Basic usage example for Steward Python bindings.

This example demonstrates how to:
1. Load a stewardship contract from YAML
2. Create an AI output to evaluate
3. Run the evaluation
4. Interpret the results
"""

from steward import Contract, Output, evaluate

# Define a simple contract inline
CONTRACT_YAML = """
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Customer Support Assistant"
intent:
  purpose: "Provide helpful, accurate customer support responses"
  never_optimize_away:
    - id: "N1"
      rule: "Human escalation path must always be available"
boundaries:
  may_do_autonomously:
    - id: "A1"
      rule: "Answer factual questions about products"
    - id: "A2"
      rule: "Provide order status from verified data"
  must_escalate_when:
    - id: "E1"
      rule: "Customer requests human agent"
  invalidated_by:
    - id: "B1"
      rule: "Customer PII exposed in response"
    - id: "B5"
      rule: "Internal system credentials or secrets exposed"
accountability:
  approved_by: "Support Manager"
  answerable_human: "support@example.com"
  escalation_path:
    - "Tier 1 Support"
    - "Tier 2 Support"
    - "Support Manager"
acceptance:
  dignity_check:
    - id: "D1"
      rule: "Does not dismiss or minimize customer concerns"
    - id: "D3"
      rule: "Preserves clear path to human help"
  fit_criteria:
    - id: "F1"
      rule: "Addresses the customer's actual question"
    - id: "F4"
      rule: "Cites sources when making factual claims"
"""


def main():
    # Load the contract
    contract = Contract.from_yaml(CONTRACT_YAML)
    print(f"Loaded contract: {contract.name}")
    print(f"Purpose: {contract.purpose}")
    print(f"Accountable human: {contract.accountable_human}")
    print()

    # Example 1: A good response that should PROCEED
    print("=" * 60)
    print("Example 1: Good response")
    print("=" * 60)
    good_output = Output.text(
        "Regarding your question about the order: According to our records, "
        "your order shipped yesterday. Here's what you can do: track your package "
        "at the link in your email. If you need further assistance, please contact us "
        "or speak to a human agent."
    )
    result = evaluate(contract, good_output)
    print(f"State: {'PROCEED' if result.is_proceed() else 'ESCALATE' if result.is_escalate() else 'BLOCKED'}")
    print(f"Confidence: {result.confidence:.2f}")
    if result.is_proceed():
        print(f"Summary: {result.summary}")
    print()

    # Example 2: Response with PII that should be BLOCKED
    print("=" * 60)
    print("Example 2: PII exposure (should be BLOCKED)")
    print("=" * 60)
    pii_output = Output.text(
        "Your order was shipped to john.doe@email.com. "
        "Contact us if you have questions."
    )
    result = evaluate(contract, pii_output)
    print(f"State: {'PROCEED' if result.is_proceed() else 'ESCALATE' if result.is_escalate() else 'BLOCKED'}")
    print(f"Confidence: {result.confidence:.2f}")
    if result.is_blocked():
        violation = result.violation
        print(f"Violation: {violation.rule_id} - {violation.rule_text}")
        print(f"Contact: {violation.accountable_human}")
    print()

    # Example 3: Response with uncited claims that should ESCALATE
    print("=" * 60)
    print("Example 3: Uncited claims (should ESCALATE)")
    print("=" * 60)
    uncited_output = Output.text(
        "Studies show that our product is the best on the market. "
        "Research proves that 95% of customers see improvement."
    )
    result = evaluate(contract, uncited_output)
    print(f"State: {'PROCEED' if result.is_proceed() else 'ESCALATE' if result.is_escalate() else 'BLOCKED'}")
    print(f"Confidence: {result.confidence:.2f}")
    if result.is_escalate():
        print(f"Decision point: {result.decision_point}")
    print()

    # Example 4: Inspect lens findings
    print("=" * 60)
    print("Example 4: Lens findings inspection")
    print("=" * 60)
    result = evaluate(contract, good_output)
    print("Lens findings:")
    for finding in result.lens_findings.all():
        lens_name = finding.lens.name if finding.lens else "Unknown"
        state = "PASS" if finding.state.is_pass() else "ESCALATE" if finding.state.is_escalate() else "BLOCKED"
        print(f"  - {lens_name}: {state} (confidence: {finding.confidence:.2f})")


if __name__ == "__main__":
    main()
