---
description: Shared operating guidelines for all agent interactions
---

# Agent Operating Guidelines

This document defines shared protocols for agent coordination, conflict resolution,
and safe operation within the project workspace.

## 1. Orchestration Model

### Workflow Phases

Agents operate in a structured workflow with clear phase boundaries:

```
+-----------------------------------------------------------------------------+
| DISCOVERY PHASE                                                     |
| Business Analyst -> (requirements, stakeholders, value proposition) |
| Data Analyst -> (metrics requirements, data sources)                |
+------------------------------------+----------------------------------------+
|-----|
                                     v
+-----------------------------------------------------------------------------+
| DESIGN PHASE                                                          |
| Technical Architect -> (system architecture, data model, integration) |
| System Designer -> (UX architecture, design system, accessibility)    |
| Security Expert -> (threat model, compliance, controls)               |
+------------------------------------+----------------------------------------+
|-----|
                                     v
+-----------------------------------------------------------------------------+
| PLANNING PHASE                                                  |
| Development Lead -> (task breakdown, milestones, risk register) |
| QA Expert -> (test strategy, quality gates)                     |
+------------------------------------+----------------------------------------+
|-----|
                                     v
+-----------------------------------------------------------------------------+
| EXECUTION PHASE                                   |
| (Implementation by Development Lead coordination) |
+-----------------------------------------------------------------------------+
```

### Phase Entry Criteria

| Phase     | Entry Criteria        | Primary Agents                       | Supporting Agents |
|-----------|-----------------------|--------------------------------------|-------------------|
| Discovery | Request initiated     | Business Analyst                     | Data Analyst      |
| Design    | Requirements approved | Technical Architect, System Designer | Security Expert   |
| Planning  | Architecture approved | Development Lead                     | QA Expert         |
| Execution | Tasks defined         | Development Lead                     | All (as needed)   |

### Handoff Protocol

When transitioning between phases:

1. **Producing agent** creates a handoff summary with:
   - Deliverables produced
   - Open questions requiring resolution
   - Assumptions made (see Assumptions Template below)
   - Recommended next agent(s)

2. **Receiving agent** acknowledges with:
   - Confirmation of artifacts received
   - Questions for clarification
   - Identified gaps requiring upstream resolution

### Parallel vs Sequential Work

- **Parallel allowed**: Within the same phase (e.g., Technical Architect + System Designer)
- **Sequential required**: Cross-phase dependencies (e.g., requirements before architecture)
- **Cross-consultation**: Any agent may consult another at any time for validation

---

## 2. RACI Matrix

### Domain Responsibility Matrix

| Domain                     | R (Responsible)     | A (Accountable)     | C (Consulted)       | I (Informed)        |
|----------------------------|---------------------|---------------------|---------------------|---------------------|
| **Data Modeling**          |                     |                     |                     |                     |
| - Entity relationships     | Technical Architect | Technical Architect | Data Analyst        | Business Analyst    |
| - Dimensional models       | Data Analyst        | Data Analyst        | Technical Architect | Business Analyst    |
| - Event schemas            | Data Analyst        | Data Analyst        | Technical Architect | QA Expert           |
| **KPIs/Metrics**           |                     |                     |                     |                     |
| - Business KPIs            | Business Analyst    | Business Analyst    | Data Analyst        | Technical Architect |
| - Technical metrics        | Data Analyst        | Data Analyst        | Business Analyst    | QA Expert           |
| - Measurement methodology  | Data Analyst        | Data Analyst        | Business Analyst    | QA Expert           |
| **Security**               |                     |                     |                     |                     |
| - Threat modeling          | Security Expert     | Security Expert     | Technical Architect | All                 |
| - PII handling             | Security Expert     | Security Expert     | Data Analyst        | All                 |
| - Compliance mapping       | Security Expert     | Security Expert     | Business Analyst    | All                 |
| **Quality**                |                     |                     |                     |                     |
| - Test strategy            | QA Expert           | QA Expert           | Development Lead    | All                 |
| - Quality gates            | QA Expert           | QA Expert           | Development Lead    | Security Expert     |
| - Defect management        | QA Expert           | QA Expert           | Development Lead    | Business Analyst    |
| **Architecture**           |                     |                     |                     |                     |
| - System design            | Technical Architect | Technical Architect | All                 | Development Lead    |
| - Technology selection     | Technical Architect | Technical Architect | Development Lead    | All                 |
| - API design               | Technical Architect | Technical Architect | Security Expert     | Development Lead    |
| **UX/Design**              |                     |                     |                     |                     |
| - Information architecture | System Designer     | System Designer     | Business Analyst    | Technical Architect |
| - Design tokens            | System Designer     | System Designer     | -                   | Development Lead    |
| - Accessibility            | System Designer     | System Designer     | QA Expert           | All                 |

### Conflict Resolution Protocol

When agents produce conflicting recommendations:

1. **Identify the conflict domain** using the RACI matrix above
2. **Accountable agent decides** - The "A" role for that domain makes the final call
3. **Document the decision** with:
   - Options considered
   - Decision rationale
   - Trade-offs accepted
   - Dissenting perspectives (for the record)

### Escalation Path

If the RACI matrix does not clearly assign accountability:

1. Technical questions -> Technical Architect
2. Business questions -> Business Analyst
3. Risk/compliance questions -> Security Expert
4. Cross-cutting questions -> Development Lead (as coordinator)

---

## 3. Shared Glossary

### Terms and Definitions

| Term                    | Definition                                                                                                                        | Authoritative Agent                                      |
|-------------------------|-----------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------|
| **ADR**                 | Architecture Decision Record - a document capturing a significant architectural decision, its context, and consequences           | Technical Architect                                      |
| **Acceptance Criteria** | Conditions that must be satisfied for a user story to be considered complete                                                      | Business Analyst                                         |
| **Data Model**          | Logical or physical representation of data entities and their relationships                                                       | Technical Architect (entity), Data Analyst (dimensional) |
| **Design Token**        | Named design values (colors, spacing, typography) that form the foundation of a design system                                     | System Designer                                          |
| **E2E Test**            | End-to-end test that validates complete user journeys through the system                                                          | QA Expert                                                |
| **Epic**                | Large body of work that can be broken down into smaller stories                                                                   | Development Lead                                         |
| **Guard Rail Metric**   | A metric that should not degrade even when optimizing other metrics                                                               | Data Analyst                                             |
| **KPI**                 | Key Performance Indicator - a measurable value demonstrating effectiveness                                                        | Business Analyst (business), Data Analyst (technical)    |
| **North Star Metric**   | The single metric that best captures the core value delivered to customers                                                        | Business Analyst                                         |
| **PII**                 | Personally Identifiable Information - data that could identify an individual                                                      | Security Expert                                          |
| **Quality Gate**        | A checkpoint in the CI/CD pipeline that must pass before proceeding                                                               | QA Expert                                                |
| **RACI**                | Responsible, Accountable, Consulted, Informed - a responsibility assignment matrix                                                | Development Lead                                         |
| **SLA**                 | Service Level Agreement - a commitment to measurable service quality                                                              | Technical Architect                                      |
| **SLO**                 | Service Level Objective - a target value for a service level indicator                                                            | Technical Architect                                      |
| **STRIDE**              | Spoofing, Tampering, Repudiation, Information disclosure, Denial of service, Elevation of privilege - a threat modeling framework | Security Expert                                          |
| **Technical Debt**      | Implied cost of future rework caused by choosing an expedient solution                                                            | Development Lead                                         |
| **Threat Model**        | Structured representation of security threats, assets, and mitigations                                                            | Security Expert                                          |
| **User Story**          | A description of a feature from the perspective of the end user                                                                   | Business Analyst                                         |

### Acronyms

| Acronym | Expansion                                           |
|---------|-----------------------------------------------------|
| ABAC    | Attribute-Based Access Control                      |
| API     | Application Programming Interface                   |
| CI/CD   | Continuous Integration / Continuous Deployment      |
| GDPR    | General Data Protection Regulation                  |
| HIPAA   | Health Insurance Portability and Accountability Act |
| MFA     | Multi-Factor Authentication                         |
| OWASP   | Open Web Application Security Project               |
| PCI-DSS | Payment Card Industry Data Security Standard        |
| RBAC    | Role-Based Access Control                           |
| RTO     | Recovery Time Objective                             |
| RPO     | Recovery Point Objective                            |
| SOC2    | System and Organization Controls 2                  |
| SSO     | Single Sign-On                                      |
| TLS     | Transport Layer Security                            |
| WCAG    | Web Content Accessibility Guidelines                |

---

## 4. Tool Safety Guidelines

### Universal Safe-Usage Rules

All agents must follow these rules when using allowed tools:

#### Bash Tool Safety

| Pattern                                                 | Risk Level | Safe Usage Rules                                                                        |
|---------------------------------------------------------|------------|-----------------------------------------------------------------------------------------|
| `git log:*`, `git show:*`, `git diff:*`, `git branch:*` | Low        | Read-only, safe to use freely                                                           |
| `git:*` (other)                                         | Medium     | Never use `--force`, `--hard`, or destructive operations without explicit user approval |
| `go:*`, `cargo:*`, `npm:*`                              | Medium     | Build/test only; never publish or deploy                                                |
| `psql:*`, `mysql:*`                                     | High       | SELECT queries only; never INSERT/UPDATE/DELETE without explicit approval               |
| Test runners (`go test`, `pytest`, etc.)                | Low        | Safe to run; be aware of test fixtures that may modify state                            |
| `make:*`                                                | Medium     | Review Makefile targets before running; avoid deploy/publish targets                    |

#### Prohibited Actions (All Agents)

- **No secrets in output**: Never include API keys, passwords, tokens, or credentials
- **No production changes**: Never run commands that modify production systems
- **No force operations**: Never use `--force`, `--hard`, `-f` flags without explicit approval
- **No history rewriting**: Never use `git rebase`, `git commit --amend` on shared branches
- **No publishing**: Never run `npm publish`, `cargo publish`, `go release`, etc.

#### mcp__linear__* Tool Documentation

These tools interact with Linear project management:

| Tool                               | Purpose                  | Safe Usage                                       |
|------------------------------------|--------------------------|--------------------------------------------------|
| `mcp__linear__get_issue`           | Retrieve issue details   | Read-only, safe                                  |
| `mcp__linear__list_issues`         | List issues with filters | Read-only, safe                                  |
| `mcp__linear__create_issue`        | Create new issues        | Creates data; confirm with user before creating  |
| `mcp__linear__update_issue`        | Modify existing issues   | Modifies data; confirm with user before updating |
| `mcp__linear__create_comment`      | Add comments to issues   | Creates data; confirm with user before posting   |
| `mcp__linear__list_comments`       | Read issue comments      | Read-only, safe                                  |
| `mcp__linear__list_teams`          | List workspace teams     | Read-only, safe                                  |
| `mcp__linear__get_team`            | Get team details         | Read-only, safe                                  |
| `mcp__linear__list_projects`       | List projects            | Read-only, safe                                  |
| `mcp__linear__get_project`         | Get project details      | Read-only, safe                                  |
| `mcp__linear__list_users`          | List workspace users     | Read-only, safe                                  |
| `mcp__linear__get_user`            | Get user details         | Read-only, safe                                  |
| `mcp__linear__list_cycles`         | List sprint cycles       | Read-only, safe                                  |
| `mcp__linear__list_issue_labels`   | List available labels    | Read-only, safe                                  |
| `mcp__linear__list_issue_statuses` | List workflow states     | Read-only, safe                                  |
| `mcp__linear__list_documents`      | List project documents   | Read-only, safe                                  |
| `mcp__linear__get_document`        | Get document content     | Read-only, safe                                  |
| `mcp__linear__create_document`     | Create new document      | Creates data; confirm with user                  |
| `mcp__linear__update_document`     | Modify document          | Modifies data; confirm with user                 |

#### WebSearch and WebFetch Safety

- **WebSearch**: Safe for research; cite sources in output
- **WebFetch**: Safe for public URLs; never use with authenticated endpoints or internal URLs

### Agent-Specific Tool Constraints

Refer to each agent's `allowed-tools` in their YAML frontmatter for the authoritative list.

---

## 5. Assumptions and Open Questions Protocol

### Assumptions Template

Every agent output should include an Assumptions section when:
- Making decisions based on incomplete information
- Inferring requirements not explicitly stated
- Choosing between multiple valid approaches

Format:

```markdown
## Assumptions Made

| ID  | Assumption             | Impact if Wrong | Confidence   | Needs Validation By |
|-----|------------------------|-----------------|--------------|---------------------|
| A1  | [Assumption statement] | [What breaks]   | High/Med/Low | [Agent/Role]        |
| A2  | [Assumption statement] | [What breaks]   | High/Med/Low | [Agent/Role]        |
```

### Open Questions Template

```markdown
## Open Questions

| ID  | Question   | Blocking? | Default if Unanswered | Owner   |
|-----|------------|-----------|-----------------------|---------|
| Q1  | [Question] | Yes/No    | [Default decision]    | [Agent] |
| Q2  | [Question] | Yes/No    | [Default decision]    | [Agent] |
```

### Resolution Tracking

When assumptions are validated or questions answered:

```markdown
## Resolved Items

| ID  | Original              | Resolution                    | Resolved By | Date   |
|-----|-----------------------|-------------------------------|-------------|--------|
| A1  | [Original assumption] | [Confirmed/Modified/Rejected] | [Agent]     | [Date] |
| Q1  | [Original question]   | [Answer]                      | [Agent]     | [Date] |
```

---

## 6. PII and Sensitive Data Guidelines

### Data Classification

| Classification   | Definition         | Examples                    | Handling Requirements      |
|------------------|--------------------|-----------------------------|----------------------------|
| **Public**       | No restrictions    | Marketing copy, public docs | None                       |
| **Internal**     | Internal use only  | Architecture docs, code     | No external sharing        |
| **Confidential** | Restricted access  | Business metrics, roadmaps  | Access controls required   |
| **Restricted**   | Highest protection | PII, PHI, credentials       | Encryption + audit logging |

### PII Identification Checklist

All agents should flag the following as PII when encountered:

- [ ] Names (full, partial, usernames)
- [ ] Email addresses
- [ ] Phone numbers
- [ ] Physical addresses
- [ ] Social Security / National ID numbers
- [ ] Financial account numbers
- [ ] Health information (PHI)
- [ ] Biometric data
- [ ] IP addresses (in some contexts)
- [ ] Device identifiers
- [ ] Location data
- [ ] Authentication credentials

### Agent-Specific PII Responsibilities

| Agent                   | PII Responsibility                                                                 |
|-------------------------|------------------------------------------------------------------------------------|
| **Business Analyst**    | Flag PII in requirements; ensure consent/privacy requirements captured             |
| **Data Analyst**        | Design aggregation/anonymization for analytics; never expose raw PII in dashboards |
| **Technical Architect** | Design data flows with PII encryption and access controls                          |
| **Security Expert**     | Define PII handling procedures; map to compliance requirements                     |
| **System Designer**     | Ensure UX supports privacy (consent flows, data visibility controls)               |
| **QA Expert**           | Use synthetic data for testing; never use production PII in test environments      |
| **Development Lead**    | Ensure PII handling tasks are explicitly tracked and reviewed                      |

### Never Include in Agent Output

- Actual credentials, API keys, or tokens
- Real user data (names, emails, etc.)
- Production database connection strings
- Internal IP addresses or hostnames
- Unredacted log entries containing PII
