# Compliance Mapping for Steward Domain Packs

This document maps Steward contract rules to specific regulatory requirements, enabling organizations to demonstrate compliance through contract adherence.

---

## Healthcare (`contracts/healthcare.yaml`)

### HIPAA Security Rule (45 CFR Part 164)

| Steward Rule | HIPAA Requirement | Description |
|--------------|-------------------|-------------|
| B1 | §164.502(b) - Minimum Necessary | PHI exposure beyond minimum necessary scope |
| B4 | §164.502(a) - Permitted Uses | PHI disclosed without valid authorization |
| B5 | §164.530(c) - Safeguards | Patient identifiers exposed in logs |
| B6 | §164.508(a)(2) - Psychotherapy Notes | Psychotherapy notes require specific authorization |
| N3 | §164.502(b) - Minimum Necessary | HIPAA minimum necessary standard for PHI access |
| N5 | §164.312(b) - Audit Controls | Audit trail for all PHI access |

### HIPAA Privacy Rule

| Steward Rule | HIPAA Requirement | Description |
|--------------|-------------------|-------------|
| E2 | §164.526 - Amendment | Patient requests amendment to medical record |
| E6 | §164.508 - Authorization | Request for PHI disclosure to third party |
| D3 | §164.522 - Individual Rights | Preserves patient autonomy in healthcare decisions |

### GINA (Genetic Information Nondiscrimination Act)

| Steward Rule | GINA Requirement | Description |
|--------------|------------------|-------------|
| B7 | Title II - Employment | Genetic information disclosed improperly |

### 42 CFR Part 2 (Substance Use Disorder Records)

| Steward Rule | 42 CFR Part 2 Requirement | Description |
|--------------|---------------------------|-------------|
| B8 | §2.31 - Consent Requirements | SUD records accessed without compliance |

---

## Financial Services (`contracts/finance.yaml`)

### SEC Regulation Best Interest (Reg BI)

| Steward Rule | Reg BI Requirement | Description |
|--------------|-------------------|-------------|
| B6 | Disclosure Obligation | Failure to disclose material conflicts |
| N4 | Disclosure Obligation | Conflict of interest identification |
| E1 | Care Obligation | Investment recommendation requires human |
| E3 | Care Obligation | Suitability assessment requires oversight |

### Investment Advisers Act of 1940

| Steward Rule | Advisers Act Section | Description |
|--------------|---------------------|-------------|
| B1 | §206(1)-(2) - Anti-fraud | Investment recommendation without advisor |
| B2 | §206(4) - Tax advice | Tax advice outside authorized scope |
| N1 | §206 - Fiduciary Duty | Human oversight for recommendations |
| F6 | §206 - Fiduciary Duty | Conflicts disclosed |

### SEC Regulation S-P (Privacy)

| Steward Rule | Reg S-P Requirement | Description |
|--------------|---------------------|-------------|
| B3 | §248.30 - Safeguards | Client financial information protected |
| B8 | §248.30 - Safeguards | Account credentials protected |

### FINRA Rules

| Steward Rule | FINRA Rule | Description |
|--------------|-----------|-------------|
| N5 | Rule 4511 | Complete and accurate recordkeeping |
| F5 | Rule 2210 | Communications with the public |
| B4 | Rule 2210(d) | Performance claims with required disclosures |

---

## Legal Services (`contracts/legal.yaml`)

### ABA Model Rules of Professional Conduct

| Steward Rule | ABA Rule | Description |
|--------------|----------|-------------|
| N1 | Rule 1.1 - Competence | Attorney review of all legal work product |
| B2 | Rule 1.6 - Confidentiality | Attorney-client privileged information protected |
| B4 | Rule 1.6 - Confidentiality | Confidential client information protected |
| N2 | Rule 1.4 - Communication | Client right to human attorney |
| B7 | Rule 5.5 - Unauthorized Practice | Advice without attorney constitutes UPL |
| E9 | Rule 1.1 - Competence | High confidence required (90%) |

### ABA Formal Opinion 512 (Generative AI)

| Steward Rule | Opinion 512 Guidance | Description |
|--------------|---------------------|-------------|
| B8 | Verification duty | Citation to non-existent case prohibited |
| F1 | Verification duty | Legal citations verified and current |
| N4 | Disclosure duty | Disclosure that AI not substitute for counsel |

### Work Product Doctrine (FRCP 26(b)(3))

| Steward Rule | Work Product Protection | Description |
|--------------|------------------------|-------------|
| B6 | Attorney work product | Protected material not exposed |

---

## Education (`contracts/education.yaml`)

### FERPA (20 U.S.C. § 1232g)

| Steward Rule | FERPA Requirement | Description |
|--------------|-------------------|-------------|
| B1 | §99.30 - Consent | Student records disclosed only with authorization |
| B6 | §99.37 - Directory information | Opt-out honored for directory info |
| E4 | §99.10 - Right to inspect | Parent/guardian access rights |
| E8 | §99.31 - Transfer | Student records transfer procedures |
| N2 | §99.10 - Access | Student and parent access to records |

### COPPA (15 U.S.C. § 6501–6506)

| Steward Rule | COPPA Requirement | Description |
|--------------|-------------------|-------------|
| B5 | §312.3 - Verifiable consent | Collection from under-13 requires consent |
| E7 | §312.3 - Notice | Special handling for under-13 |

### IDEA (Individuals with Disabilities Education Act)

| Steward Rule | IDEA Requirement | Description |
|--------------|------------------|-------------|
| B7 | Confidentiality | IEP/disability information protected |
| N5 | Child Find/FAPE | Accommodations for students with disabilities |

### Academic Integrity

| Steward Rule | Academic Standard | Description |
|--------------|-------------------|-------------|
| B3 | Institutional policy | Complete solutions for graded work prohibited |
| N3 | Institutional policy | Academic integrity and original work |
| E6 | Institutional policy | Academic integrity violations escalated |

---

## Human Resources (`contracts/hr.yaml`)

### Title VII of the Civil Rights Act

| Steward Rule | Title VII Requirement | Description |
|--------------|----------------------|-------------|
| B5 | §703(a) - Discrimination | Protected class not considered in decisions |
| B8 | Disparate Impact | Bias audit required for automated tools |
| F6 | Disparate Impact | Adverse impact analysis conducted |

### Americans with Disabilities Act (ADA)

| Steward Rule | ADA Requirement | Description |
|--------------|-----------------|-------------|
| B3 | §102(d) - Medical exams | Disability status confidentiality |
| B6 | §102(d) - Medical records | Medical information access restricted |
| E5 | §102(b)(5) - Reasonable accommodation | Accommodation determination to human |
| P2 | §102(b)(5) | Accommodation requests paused |

### Age Discrimination in Employment Act (ADEA)

| Steward Rule | ADEA Requirement | Description |
|--------------|------------------|-------------|
| B5 | §4(a) - Age discrimination | Age not considered in decisions |

### NYC Local Law 144 (AEDT Law)

| Steward Rule | LL144 Requirement | Description |
|--------------|-------------------|-------------|
| B8 | §20-870 - Bias audit | Audit required before deployment |
| F4 | §20-871 - Notice | AI involvement disclosed |
| F5 | §20-871 - Summary | Bias audit results available |

### EEOC AI Guidance (2023)

| Steward Rule | EEOC Guidance | Description |
|--------------|---------------|-------------|
| B1 | Human oversight | Employment decision requires human review |
| N1 | Human oversight | Hiring and termination decisions reviewed |
| E1, E2, E3, E7 | Human oversight | Key decisions escalated to humans |

---

## Cross-Domain Compliance Summary

| Regulation | Healthcare | Finance | Legal | Education | HR |
|------------|-----------|---------|-------|-----------|-----|
| Human oversight required | ✅ | ✅ | ✅ | ✅ | ✅ |
| Data minimization | ✅ | ✅ | ✅ | ✅ | ✅ |
| Audit trail | ✅ | ✅ | ✅ | ✅ | ✅ |
| Bias monitoring | — | — | — | ✅ | ✅ |
| AI disclosure | ✅ | ✅ | ✅ | ✅ | ✅ |
| Escalation paths | ✅ | ✅ | ✅ | ✅ | ✅ |
| Dignity preservation | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Using This Mapping

### For Compliance Officers

1. **Identify applicable regulations** for your organization
2. **Map contract rules** to regulatory requirements using tables above
3. **Configure Steward** with appropriate domain contract
4. **Monitor BLOCKED/ESCALATE** events as compliance signals
5. **Use audit trails** to demonstrate compliance during examinations

### For Auditors

1. **Request Steward evaluation logs** for AI systems
2. **Verify rule coverage** against regulatory requirements
3. **Check BLOCKED events** for potential violations
4. **Review ESCALATE patterns** for oversight adequacy
5. **Confirm human review** of escalated decisions

### For Developers

1. **Select appropriate domain contract** for your use case
2. **Extend with custom rules** for organization-specific requirements
3. **Test against golden cases** before deployment
4. **Integrate Steward** into CI/CD pipelines
5. **Monitor confidence levels** and adjust thresholds

---

## Regulatory Update Schedule

| Domain | Primary Regulation | Last Reviewed | Review Cadence |
|--------|-------------------|---------------|----------------|
| Healthcare | HIPAA | 2025-12 | Quarterly |
| Finance | SEC/FINRA | 2025-12 | Quarterly |
| Legal | ABA Rules | 2025-12 | Annually |
| Education | FERPA/COPPA | 2025-12 | Annually |
| HR | Title VII/ADA | 2025-12 | Quarterly |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-12-20 | Initial release with 5 domain packs |

*Last Updated: 2025-12-20*
