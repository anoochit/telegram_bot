# Changelog

All notable changes to this project will be documented in this file.

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
