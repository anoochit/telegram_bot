---
name: git-sync
description: Automates Git operations (add, commit, push) for rapid project syncing. Use when you have pending changes that need to be committed and pushed to the remote repository.
allowed-tools:
  - exec_command
---

# Git Sync

This skill automates the standard Git workflow: staging all changes, committing with a clear message, and pushing to the remote repository.

## Workflow

1. **Stage Changes**: Executes `git add .` to include all untracked and modified files.
2. **Commit Changes**: Prompts for a commit message or generates one, then runs `git commit -m "<message>"`.
3. **Push Changes**: Executes `git push` to upload changes to the remote repository.

## Usage

When you need to sync your project, simply ask Gemini CLI to:
"Sync my changes", "git push", or "Commit and push this".

The agent will then:
- Check `git status`.
- Add all files.
- Ask for (or generate) a commit message.
- Commit and push the changes.

