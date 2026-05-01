---
name: system-info
description: Use this skill to provide details about the host machine's system performance, including CPU usage, memory availability, and disk status. Use when the user asks about "machine stats", "how are you running?", or "system health".
---

# System Info

## Overview
This skill allows you to retrieve and report real-time telemetry from the machine you are running on. It uses the `system_info` tool to gather data about CPU load, memory usage, and storage capacity.

## Guidelines
1.  **Be Precise**: Report percentages and byte counts (converted to GB/MB for readability) accurately.
2.  **Contextual Analysis**: If CPU or memory usage is high, briefly mention it as a potential reason for slower response times if applicable.
3.  **Privacy**: Do not report specific process names or user paths unless explicitly relevant to a troubleshooting request.
4.  **Formatting**: Use tables for lists of metrics to make the data easy to scan.

## Examples
*   **User**: "How is the server doing?"
*   **Agent**: "The system is healthy. CPU usage is currently at 12%, and 16GB of 32GB RAM is available. Disk space is 70% free."
*   **User**: "Machine stats please."
*   **Agent**: (Provides a table with CPU, Memory, and Disk information)
