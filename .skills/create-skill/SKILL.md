---
name: create-skill
description: Use this skill when you want to define a new, specialized persona, workflow, or set of reusable instructions. It scaffolds a new directory under `.skills/` to encapsulate the expertise needed for recurring, complex tasks.
allowed-tools:
  - read_file
  - write_file
  - list_dir
  - exec_command
  - glob_find
---

# Create Skill

## Overview
This skill automates the creation of modular "capabilities." By formalizing a task into a skill, you ensure consistent behavior and high-quality outputs for specialized workflows.

## Workflow

1.  **Requirement Gathering**: Identify the core objective. If the user is vague, propose a name and description before writing.
2.  **Conflict Check**: Use `list_dir` on `.skills/` to ensure the `<skill-name>` doesn't already exist to avoid overwriting existing logic.
3.  **Drafting**: Populate the `SKILL.md` using the template below. 
    *   **Logic Check**: Ensure the `description` in the frontmatter contains "trigger keywords" that help an LLM know when to call this skill.
4.  **Execution**:
    *   Create the directory: `.skills/<kebab-case-name>/`.
    *   Write the file: `.skills/<kebab-case-name>/SKILL.md`.
5.  **Verification**: Confirm the file exists and summarize the new "superpower" to the user.

## SKILL.md Template
```markdown
---
name: <kebab-case-name>
description: <Critical for discovery. Use: "Use this skill when..." followed by specific triggers.>
---

# <Display Name>

## Persona & Context
<Define the specific 'hat' agent wears. Is it a Senior Dev? A Creative Writer? Skeptical Auditor?>

## Core Objectives
* <Objective 1: The primary goal.>
* <Objective 2: What success looks like.>

## Constraints & Guidelines
1. **Constraint 1**: (e.g., "Always use TypeScript," "Never mention X.")
2. **Behavior 2**: (e.g., "Be concise and use Markdown tables for comparisons.")
3. **Step-by-Step**: (e.g., "Always validate the input before processing.")

## Evaluation Criteria
- How should the user or agent know if this skill performed correctly?