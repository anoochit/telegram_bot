# LLM Token Optimization Plan - namiclaw

This plan outlines the strategies for reducing token consumption and improving context efficiency in the namiclaw project.

## Phase 1: System Prompt Pruning (`src/agent/agent.rs`)
**Goal:** Reduce fixed token overhead in every turn.
- **Condensation:** Rewrite system instructions into a "token-dense" format using concise bullet points.
- **Metadata Reliance:** Remove detailed explanations of tools (Wiki, FileSystem, etc.) that are already provided by tool definitions.
- **Constraint Consolidation:** Merge "Output Standards" (Language, Formatting, Plain Text) into a single block.

## Phase 2: Intelligent "Lazy" File Context (`src/modes/cli.rs`)
**Goal:** Prevent large file injections from exhausting the context window.
- **Size Thresholding:** Implement a 4KB threshold in `process_file_references`.
    - **Small Files (< 4KB):** Direct injection.
    - **Large Files (> 4KB):** Inject a "Reference Pointer" only.
- **Lazy Loading Strategy:** Explicitly instruct the agent to use `read_file` with line ranges for large referenced files.

## Phase 3: Compaction Tuning (`src/agent/agent.rs`)
**Goal:** Optimize long-term memory summarization.
- **Interval Adjustment:** Increase `compaction_interval` from 5 to 8 or 10.
- **Overlap Reduction:** Decrease `overlap_size` from 2 to 1 to minimize duplication between summaries and raw events.

## Phase 4: Sub-Agent Delegation Protocol
**Goal:** Prevent verbose sub-agent outputs from bloating the main conversation.
- **Summary Mandate:** Update sub-agent instructions to prioritize returning "Summary + Conclusion" over verbose logs.
- **Output Pruning:** Implement logic to strip non-essential metadata from sub-agent responses before they enter the main context.

---
*Created on: Monday, May 4, 2026*
