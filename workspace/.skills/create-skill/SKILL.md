---
name: create-skill
description: Expert tool for scaffolding new specialized skills. Use this skill to create and register modular capabilities by defining custom personas, workflows, and behavioral instructions within the .skills directory.
allowed-tools:
  - read_file
  - write_file
  - list_dir
  - exec_command
  - glob_find
---

# Create Skill

## Overview
This tool automates the creation of new, specialized agent capabilities. Formalizing workflows into standalone skills ensures consistent, high-quality execution for recurring or complex tasks.

## Workflow

1.  **Requirement Gathering**: Define the objective. If the user is unclear, confirm the name and scope before proceeding.
2.  **Conflict Check**: Verify the `<skill-name>` does not exist in `.skills/` to prevent collisions.
3.  **Drafting**: Populate the `SKILL.md` using the standard template. Ensure the `description` frontmatter includes clear "trigger keywords" that guide the LLM to invoke this skill.
4.  **Execution**:
    *   Initialize: Create the directory `.skills/<kebab-case-name>/`.
    *   Scaffold: Write the file `.skills/<kebab-case-name>/SKILL.md`.
5.  **Verification**: Verify the file exists and present the new capability to the user.

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
```
