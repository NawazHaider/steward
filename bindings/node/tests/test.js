/**
 * Tests for the Steward Node.js bindings.
 *
 * Note: These tests require the native extension to be built.
 * Run `npm run build` first to build the native binding.
 */

const assert = require('assert');
const {
  Contract,
  Output,
  evaluate,
  evaluateWithContext,
  isProceed,
  isEscalate,
  isBlocked,
} = require('../index.js');

const SIMPLE_CONTRACT = `
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
`;

const PII_CONTRACT = `
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
`;

describe('Contract', () => {
  it('should create from YAML string', () => {
    const contract = Contract.fromYaml(SIMPLE_CONTRACT);
    assert.strictEqual(contract.name, 'Test Contract');
    assert.strictEqual(contract.version, '1.0');
    assert.strictEqual(contract.purpose, 'Test evaluation');
    assert.strictEqual(contract.accountableHuman, 'support@example.com');
  });

  it('should throw on invalid YAML', () => {
    assert.throws(() => {
      Contract.fromYaml('not: valid: yaml: {{');
    });
  });
});

describe('Output', () => {
  it('should create text output', () => {
    const output = Output.text('Hello, world!');
    assert.strictEqual(output.content, 'Hello, world!');
  });
});

describe('evaluate', () => {
  it('should return Proceed for clean response', () => {
    const contract = Contract.fromYaml(SIMPLE_CONTRACT);
    const output = Output.text('This is a helpful response about your question.');
    const result = evaluate(contract, output);

    assert(isProceed(result.state.stateType));
    assert(!isBlocked(result.state.stateType));
    assert(!isEscalate(result.state.stateType));
    assert(result.confidence > 0);
  });

  it('should return Blocked for PII exposure', () => {
    const contract = Contract.fromYaml(PII_CONTRACT);
    const output = Output.text('Contact john.doe@email.com for help.');
    const result = evaluate(contract, output);

    assert(isBlocked(result.state.stateType));
    assert(!isProceed(result.state.stateType));
    assert(result.state.violation !== null);
    assert.strictEqual(result.state.violation.ruleId, 'B1');
    assert.strictEqual(result.state.violation.accountableHuman, 'security@example.com');
  });
});

describe('LensFindings', () => {
  it('should include all five lenses', () => {
    const contract = Contract.fromYaml(SIMPLE_CONTRACT);
    const output = Output.text('A response.');
    const result = evaluate(contract, output);

    assert(result.lensFindings.dignityInclusion !== undefined);
    assert(result.lensFindings.boundariesSafety !== undefined);
    assert(result.lensFindings.restraintPrivacy !== undefined);
    assert(result.lensFindings.transparencyContestability !== undefined);
    assert(result.lensFindings.accountabilityOwnership !== undefined);
  });

  it('should have valid confidence values', () => {
    const contract = Contract.fromYaml(SIMPLE_CONTRACT);
    const output = Output.text('A response.');
    const result = evaluate(contract, output);

    const findings = [
      result.lensFindings.dignityInclusion,
      result.lensFindings.boundariesSafety,
      result.lensFindings.restraintPrivacy,
      result.lensFindings.transparencyContestability,
      result.lensFindings.accountabilityOwnership,
    ];

    for (const finding of findings) {
      assert(finding.confidence >= 0 && finding.confidence <= 1);
    }
  });
});

describe('evaluateWithContext', () => {
  it('should work with context', () => {
    const contract = Contract.fromYaml(SIMPLE_CONTRACT);
    const output = Output.text("Based on your order history, here's what I found.");
    const context = ['Customer asked about order status', 'Order #12345 shipped yesterday'];
    const result = evaluateWithContext(contract, output, context);

    assert(result.confidence > 0);
  });

  it('should work with metadata', () => {
    const contract = Contract.fromYaml(SIMPLE_CONTRACT);
    const output = Output.text("Here's your answer.");
    const metadata = { session_id: 'abc123', agent_version: '1.0' };
    const result = evaluateWithContext(contract, output, null, metadata);

    assert(result.confidence > 0);
  });
});

describe('Determinism', () => {
  it('should produce same output for same input', () => {
    const contract = Contract.fromYaml(SIMPLE_CONTRACT);
    const output = Output.text('A consistent response.');

    const result1 = evaluate(contract, output);
    const result2 = evaluate(contract, output);
    const result3 = evaluate(contract, output);

    // All results should have the same state type
    assert.strictEqual(isProceed(result1.state.stateType), isProceed(result2.state.stateType));
    assert.strictEqual(isProceed(result2.state.stateType), isProceed(result3.state.stateType));

    // All results should have the same confidence
    assert.strictEqual(result1.confidence, result2.confidence);
    assert.strictEqual(result2.confidence, result3.confidence);
  });
});

// Simple test runner
function describe(name, fn) {
  console.log(`\n${name}`);
  fn();
}

function it(name, fn) {
  try {
    fn();
    console.log(`  ✓ ${name}`);
  } catch (error) {
    console.error(`  ✗ ${name}`);
    console.error(`    ${error.message}`);
    process.exitCode = 1;
  }
}

// Run if this file is executed directly
if (require.main === module) {
  console.log('Running Steward Node.js binding tests...\n');
  console.log('Note: These tests require the native binding to be built.');
  console.log('Run `npm run build` first if tests fail.\n');
}
