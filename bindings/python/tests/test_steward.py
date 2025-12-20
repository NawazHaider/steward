"""Tests for the Steward Python bindings."""

import pytest


# Note: These tests require the native extension to be built.
# Run `maturin develop` first to build and install.


SIMPLE_CONTRACT = """
contract_version: "1.0"
schema_version: "2025-12-20"
name: "Test Contract"
intent:
  purpose: "Test evaluation"
boundaries:
  may_do_autonomously:
    - id: "A1"
      rule: "Answer questions"
accountability:
  approved_by: "Manager"
  answerable_human: "support@example.com"
  escalation_path:
    - "Tier 1 Support"
    - "Manager"
acceptance: {}
"""

PII_CONTRACT = """
contract_version: "1.0"
schema_version: "2025-12-20"
name: "PII Test Contract"
intent:
  purpose: "Test PII detection"
boundaries:
  invalidated_by:
    - id: "B1"
      rule: "Customer PII exposed in response"
accountability:
  answerable_human: "security@example.com"
acceptance: {}
"""


class TestContract:
    """Tests for Contract class."""

    def test_from_yaml(self):
        """Test loading a contract from YAML string."""
        from steward import Contract

        contract = Contract.from_yaml(SIMPLE_CONTRACT)
        assert contract.name == "Test Contract"
        assert contract.version == "1.0"
        assert contract.purpose == "Test evaluation"
        assert contract.accountable_human == "support@example.com"

    def test_from_yaml_invalid(self):
        """Test that invalid YAML raises an error."""
        from steward import Contract

        with pytest.raises(ValueError):
            Contract.from_yaml("not: valid: yaml: {{")

    def test_contract_repr(self):
        """Test contract string representation."""
        from steward import Contract

        contract = Contract.from_yaml(SIMPLE_CONTRACT)
        repr_str = repr(contract)
        assert "Test Contract" in repr_str
        assert "1.0" in repr_str


class TestOutput:
    """Tests for Output class."""

    def test_text_output(self):
        """Test creating a text output."""
        from steward import Output

        output = Output.text("Hello, world!")
        assert output.content == "Hello, world!"

    def test_output_repr(self):
        """Test output string representation."""
        from steward import Output

        output = Output.text("Short text")
        repr_str = repr(output)
        assert "Short text" in repr_str


class TestEvaluation:
    """Tests for the evaluate function."""

    def test_basic_proceed(self):
        """Test that a clean response proceeds."""
        from steward import Contract, Output, evaluate

        contract = Contract.from_yaml(SIMPLE_CONTRACT)
        output = Output.text("This is a helpful response about your question.")
        result = evaluate(contract, output)

        assert result.is_proceed()
        assert not result.is_blocked()
        assert not result.is_escalate()
        assert result.confidence > 0

    def test_pii_blocked(self):
        """Test that PII exposure is blocked."""
        from steward import Contract, Output, evaluate

        contract = Contract.from_yaml(PII_CONTRACT)
        output = Output.text("Contact john.doe@email.com for help.")
        result = evaluate(contract, output)

        assert result.is_blocked()
        assert not result.is_proceed()
        violation = result.violation
        assert violation is not None
        assert violation.rule_id == "B1"
        assert violation.accountable_human == "security@example.com"

    def test_result_to_json(self):
        """Test JSON serialization of result."""
        from steward import Contract, Output, evaluate
        import json

        contract = Contract.from_yaml(SIMPLE_CONTRACT)
        output = Output.text("A simple response.")
        result = evaluate(contract, output)

        json_str = result.to_json()
        data = json.loads(json_str)

        assert "state" in data
        assert "confidence" in data
        assert "evaluated_at" in data


class TestLensFindings:
    """Tests for lens findings."""

    def test_all_lenses_present(self):
        """Test that all five lenses are present in findings."""
        from steward import Contract, Output, evaluate

        contract = Contract.from_yaml(SIMPLE_CONTRACT)
        output = Output.text("A response.")
        result = evaluate(contract, output)

        findings = result.lens_findings
        all_findings = findings.all()
        assert len(all_findings) == 5

        # Check each lens is accessible
        assert findings.dignity_inclusion is not None
        assert findings.boundaries_safety is not None
        assert findings.restraint_privacy is not None
        assert findings.transparency_contestability is not None
        assert findings.accountability_ownership is not None

    def test_lens_confidence(self):
        """Test that lens confidences are valid."""
        from steward import Contract, Output, evaluate

        contract = Contract.from_yaml(SIMPLE_CONTRACT)
        output = Output.text("A response.")
        result = evaluate(contract, output)

        for finding in result.lens_findings.all():
            assert 0 <= finding.confidence <= 1


class TestEvaluateWithContext:
    """Tests for evaluate_with_context function."""

    def test_with_context(self):
        """Test evaluation with context."""
        from steward import Contract, Output, evaluate_with_context

        contract = Contract.from_yaml(SIMPLE_CONTRACT)
        output = Output.text("Based on your order history, here's what I found.")
        context = ["Customer asked about order status", "Order #12345 shipped yesterday"]
        result = evaluate_with_context(contract, output, context=context)

        assert result.confidence > 0

    def test_with_metadata(self):
        """Test evaluation with metadata."""
        from steward import Contract, Output, evaluate_with_context

        contract = Contract.from_yaml(SIMPLE_CONTRACT)
        output = Output.text("Here's your answer.")
        metadata = {"session_id": "abc123", "agent_version": "1.0"}
        result = evaluate_with_context(contract, output, metadata=metadata)

        assert result.confidence > 0


class TestDeterminism:
    """Tests for deterministic behavior."""

    def test_same_input_same_output(self):
        """Test that same input always produces same output."""
        from steward import Contract, Output, evaluate

        contract = Contract.from_yaml(SIMPLE_CONTRACT)
        output = Output.text("A consistent response.")

        results = [evaluate(contract, output) for _ in range(3)]

        # All results should have the same state type
        states = [r.is_proceed() for r in results]
        assert all(s == states[0] for s in states)

        # All results should have the same confidence
        confidences = [r.confidence for r in results]
        assert all(c == confidences[0] for c in confidences)
