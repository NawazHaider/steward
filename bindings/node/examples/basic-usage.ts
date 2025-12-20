/**
 * Basic usage example for Steward Node.js bindings.
 *
 * This example demonstrates how to:
 * 1. Load a stewardship contract from YAML
 * 2. Create an AI output to evaluate
 * 3. Run the evaluation
 * 4. Interpret the results
 */

import {
  Contract,
  Output,
  evaluate,
  isProceed,
  isEscalate,
  isBlocked,
  EvaluationResult,
} from '@steward/core';

// Define a simple contract inline
const CONTRACT_YAML = `
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
`;

function getStateLabel(result: EvaluationResult): string {
  if (isProceed(result.state.stateType)) return 'PROCEED';
  if (isEscalate(result.state.stateType)) return 'ESCALATE';
  if (isBlocked(result.state.stateType)) return 'BLOCKED';
  return 'UNKNOWN';
}

function main(): void {
  // Load the contract
  const contract = Contract.fromYaml(CONTRACT_YAML);
  console.log(`Loaded contract: ${contract.name}`);
  console.log(`Purpose: ${contract.purpose}`);
  console.log(`Accountable human: ${contract.accountableHuman}`);
  console.log();

  // Example 1: A good response that should PROCEED
  console.log('='.repeat(60));
  console.log('Example 1: Good response');
  console.log('='.repeat(60));
  const goodOutput = Output.text(
    'Regarding your question about the order: According to our records, ' +
    'your order shipped yesterday. Here\'s what you can do: track your package ' +
    'at the link in your email. If you need further assistance, please contact us ' +
    'or speak to a human agent.'
  );
  let result = evaluate(contract, goodOutput);
  console.log(`State: ${getStateLabel(result)}`);
  console.log(`Confidence: ${result.confidence.toFixed(2)}`);
  if (isProceed(result.state.stateType) && result.state.summary) {
    console.log(`Summary: ${result.state.summary}`);
  }
  console.log();

  // Example 2: Response with PII that should be BLOCKED
  console.log('='.repeat(60));
  console.log('Example 2: PII exposure (should be BLOCKED)');
  console.log('='.repeat(60));
  const piiOutput = Output.text(
    'Your order was shipped to john.doe@email.com. ' +
    'Contact us if you have questions.'
  );
  result = evaluate(contract, piiOutput);
  console.log(`State: ${getStateLabel(result)}`);
  console.log(`Confidence: ${result.confidence.toFixed(2)}`);
  if (isBlocked(result.state.stateType) && result.state.violation) {
    const violation = result.state.violation;
    console.log(`Violation: ${violation.ruleId} - ${violation.ruleText}`);
    console.log(`Contact: ${violation.accountableHuman}`);
  }
  console.log();

  // Example 3: Response with uncited claims that should ESCALATE
  console.log('='.repeat(60));
  console.log('Example 3: Uncited claims (should ESCALATE)');
  console.log('='.repeat(60));
  const uncitedOutput = Output.text(
    'Studies show that our product is the best on the market. ' +
    'Research proves that 95% of customers see improvement.'
  );
  result = evaluate(contract, uncitedOutput);
  console.log(`State: ${getStateLabel(result)}`);
  console.log(`Confidence: ${result.confidence.toFixed(2)}`);
  if (isEscalate(result.state.stateType) && result.state.decisionPoint) {
    console.log(`Decision point: ${result.state.decisionPoint}`);
  }
  console.log();

  // Example 4: Inspect lens findings
  console.log('='.repeat(60));
  console.log('Example 4: Lens findings inspection');
  console.log('='.repeat(60));
  result = evaluate(contract, goodOutput);
  console.log('Lens findings:');
  const findings = [
    { name: 'Dignity & Inclusion', finding: result.lensFindings.dignityInclusion },
    { name: 'Boundaries & Safety', finding: result.lensFindings.boundariesSafety },
    { name: 'Restraint & Privacy', finding: result.lensFindings.restraintPrivacy },
    { name: 'Transparency', finding: result.lensFindings.transparencyContestability },
    { name: 'Accountability', finding: result.lensFindings.accountabilityOwnership },
  ];
  for (const { name, finding } of findings) {
    const state = finding.state.stateType;
    console.log(`  - ${name}: ${state} (confidence: ${finding.confidence.toFixed(2)})`);
  }
}

main();
