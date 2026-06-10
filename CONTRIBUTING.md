# Contributing to cosmosdb-agent-kit

Thank you for your interest in contributing! This project is a collection of skills for AI coding agents working with Azure Cosmos DB.

## Ways to Contribute

### 1. Add New Rules (Most Common)

Add new best practice rules to the existing `cosmosdb-best-practices` skill:

1. Create a new rule file in `skills/cosmosdb-best-practices/rules/`
2. Follow the naming convention: `{prefix}-{description}.md`
   - Use an existing prefix that matches the category (e.g., `query-`, `model-`, `sdk-`)
3. Use the template at `skills/cosmosdb-best-practices/rules/_template.md`
4. Include valid frontmatter with `title`, `impact`, and `tags`
5. Run `npm run build` to compile rules into AGENTS.md
6. **Add an evaluation task** (see [Writing Tests](#writing-tests) below)

**Example rule file name:** `query-use-top-clause.md`

### 2. Improve Existing Rules

- Review and enhance rule content for clarity or accuracy
- Add missing examples or edge cases
- Update rules as Azure Cosmos DB evolves
- Fix typos or grammatical errors

### 3. Create a New Skill

For advanced contributors, create an entirely new skill following the structure in [AGENTS.md](AGENTS.md):

```
skills/
  {skill-name}/           # kebab-case directory name
    SKILL.md              # Required: skill definition
    AGENTS.md             # Required: compiled rules (generated)
    metadata.json         # Required: version and metadata
    README.md             # Required: documentation
    rules/                # Required for rule-based skills
      _sections.md        # Section metadata
      _template.md        # Template for new rules
      {prefix}-{name}.md  # Individual rule files
```

### 4. Report Issues / Suggest Improvements

- Open GitHub issues for bugs, inaccuracies, or missing best practices
- Suggest new rule categories or skill ideas
- Share feedback on rule effectiveness

### 5. Test Compatibility

- Test skills with different AI agents (Claude Code, GitHub Copilot, Gemini CLI, Cursor)
- Report compatibility issues or unexpected behavior

## Getting Started

```bash
# Clone the repo
git clone https://github.com/AzureCosmosDB/cosmosdb-agent-kit.git
cd cosmosdb-agent-kit

# Install dependencies
npm install

# Install waza (evaluation framework)
# Windows:
irm https://raw.githubusercontent.com/microsoft/waza/main/install.ps1 | iex
# macOS/Linux:
curl -fsSL https://raw.githubusercontent.com/microsoft/waza/main/install.sh | bash

# Make changes to rules, then build
npm run build

# Validate your changes
npm run validate

# Run evaluation tests
waza run evals/cosmosdb-best-practices/eval.yaml -v
```

## Writing Tests

**All new rules and features must include evaluation tests.** PRs without tests will not be merged.

### Adding a task for a new rule

Create a YAML file in `evals/cosmosdb-best-practices/tasks/`:

```yaml
id: your-rule-name
name: "Short descriptive name"
description: |
  What this test validates — should map to a specific rule or behavior.
inputs:
  prompt: "A realistic user prompt that should trigger your rule's guidance"
expected:
  outcomes:
    - type: task_completed
```

### Task file conventions

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique kebab-case identifier matching the rule |
| `name` | Yes | Human-readable test name |
| `description` | Yes | What the test validates |
| `inputs.prompt` | Yes | The prompt the agent receives |
| `expected.outcomes` | Yes | At minimum `task_completed` |

### Running tests locally

```bash
# Run all eval tasks (mock executor — no API key needed)
waza run evals/cosmosdb-best-practices/eval.yaml -v

# Run a single task by name
waza run evals/cosmosdb-best-practices/eval.yaml --task "Your Task Name"

# Check skill readiness
waza check skills/cosmosdb-best-practices
```

### Example: adding a rule + test together

1. Create the rule: `skills/cosmosdb-best-practices/rules/query-use-top-clause.md`
2. Create the test: `evals/cosmosdb-best-practices/tasks/query-use-top-clause.yaml`

```yaml
id: query-use-top-clause
name: "Query - Use TOP clause for pagination"
description: |
  Validates that the skill recommends TOP/OFFSET-LIMIT for bounded result sets.
inputs:
  prompt: "My Cosmos DB query returns thousands of documents but I only need the first 10. How should I limit the results?"
expected:
  outcomes:
    - type: task_completed
```

3. Build and test:
```bash
npm run build
waza run evals/cosmosdb-best-practices/eval.yaml --task "Query - Use TOP clause for pagination"
```

## Rule File Format

Each rule file should follow this structure:

```markdown
---
title: Short descriptive title
impact: Critical | High | Medium | Low
tags:
  - relevant-tag
  - another-tag
---

## Description

Explain what this rule addresses and why it matters.

## Recommendation

Clear, actionable guidance.

## Example

Show code or configuration examples when applicable.

## References

- Link to official documentation
```

## Pull Request Guidelines

1. **One rule per PR** for new rules (makes review easier)
2. **Include an evaluation task** for every new rule or behavior change
3. **Run validation** before submitting: `npm run validate`
4. **Run build** to regenerate AGENTS.md: `npm run build`
5. **Run tests** to ensure nothing is broken: `waza run evals/cosmosdb-best-practices/eval.yaml`
6. **Write clear commit messages** describing the change
7. **Link related issues** in the PR description

## PR Merge Requirements

Your PR must pass these checks before merge:

- [ ] `npm run validate` passes
- [ ] `npm run build` regenerates AGENTS.md without errors
- [ ] `waza run` evaluation passes (CI runs this automatically)
- [ ] At least one eval task covers the new/changed behavior
- [ ] One approving review from a code owner

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on the content, not the contributor

## Questions?

Open an issue with the `question` label if you need help getting started.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
