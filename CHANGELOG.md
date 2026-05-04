# Changelog

All notable changes to this project will be documented in this file.

## [0.5.0] - 2026-05-04

### Added

- **Ebook Creation Skill**: Introduced the `create-ebook` skill, enabling users to generate PDF and EPUB files directly from a directory of Markdown files. This includes automatic page breaks between chapters.

### Changed

- **Skill Prioritization**: Updated agent instructions to prioritize specialized skills over general tools, improving the accuracy and efficiency of task execution.

## [0.4.0] - 2026-05-03

### Added

- **@ File Context References**: Implemented an interactive file referencing system in the CLI. Users can type `@` followed by a file path to inject file contents directly into the prompt. Includes built-in tab-completion powered by `rustyline` scoped to the `workspace/` sandbox.
- **Parallel Task Tool**: Added a custom `parallel_tasks` orchestrator that enables the agent to trigger multiple sub-agents simultaneously for faster multi-task processing.
- **Wiki Management Tools**: Added `get_backlinks`, `apply_template`, `check_broken_links`, and `rename_wiki_page` to enhance Obsidian-style knowledge management.
- **Daily Notes Template**: Added a default `DailyTemplate.md` for consistent daily journaling.

### Changed

- **Project Structure**: Refactored the codebase directory structure. Moved entry points into a dedicated `src/modes/` directory and relocated tools and utilities to `src/tools/` and `src/utils/` for better separation of concerns.
- **Wiki Search**: Upgraded `search_wiki` and `search_wiki_by_tag` to support regex and YAML frontmatter parsing.
- **CLI & Docs**: Changed CLI version greeting to be dynamically retrieved from `Cargo.toml` (v0.4.0).

### Fixed

- **Compilation Errors**: Resolved module pathing and type inference issues related to `get_workspace_dir` and the `rustyline` upgrade following the directory restructure.

### Removed

- **Wiki Date Search**: Removed `search_wiki_by_date` in favor of more robust tag and content search strategies.

## [0.3.0] - 2026-05-03

### Added

- **Hierarchical Sub-Agents**: Introduced an ecosystem of 7 specialized agents (Codebase Investigator, Generalist, Web Developer, DevOps Engineer, Quality Assurance, Data Specialist, Documentation Architect) to support complex task delegation.
- **Obsidian-Style Wiki**: Implemented new wiki tools including `get_wiki_graph` for knowledge graph visualization, `search_wiki_by_tag` for tag-based search, and `create_daily_note` for daily journaling, enhancing bi-directional linking capabilities.

## [0.2.0] - 2026-05-02

### Added

- **`init` command**: Automates project initialization by creating essential configuration files (`AGENT.md`, `MEMORIES.md`, `USER.md`) and bootstrapping the `sessions.db` database schema.
- **Event Compaction**: Implemented automatic conversation history compaction to manage memory and performance.
- **Configurable Compaction**: Added support for configuring compaction settings in serve mode.

### Refactored

- **Agent Configuration**: Centralized compaction logic within the agent module.
- **AgentRunner**: Improved runner architecture to support event compaction and better task management.

### Fixed

- **CLI/Runner**: Improved robustness of CLI and tool execution; added support for ESC cancellation in interactive mode.
- **Help Text**: Enhanced CLI usability with improved help formatting.

## [0.1.0] - Initial Release

- Initial project setup as a Telegram AI Bot (namiClaw) with support for persistent sessions, Wiki KM, and modular tool architecture.
