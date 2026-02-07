| Term | Definition |
|------|------------|
| Agent | An AI coding tool (Claude Code, Codex, Gemini CLI, OpenCode) that Aura monitors |
| AgentEvent | A typed message (10 variants) that flows through the IPC protocol to update session state |
| Attention | Session state indicating the agent needs user permission to proceed |
| Compacting | Session state indicating the agent is compacting its context window |
| Daemon | The long-running Aura process that manages the HUD, IPC server, and Codex client |
| Hook | A Claude Code plugin mechanism that invokes `aura hook --agent claude-code` on lifecycle events |
| HUD | Heads-Up Display — Aura's floating overlay windows (indicator + session list) |
| Idle | Session state indicating the agent has finished its turn and is waiting for user input |
| Indicator | The 36x36 pixel circular floating window that is always visible |
| IPC | Inter-Process Communication via Unix socket — how hook events reach the daemon |
| Liquid theme | A transparent glass theme style without backdrop blur, with shadows |
| Running | Session state indicating the agent is actively processing or using tools |
| Session | A tracked instance of an AI agent's activity, identified by session_id |
| Session list | The expandable window showing detailed session rows below the indicator |
| Session row | A two-line display element: state icon + name on line 1, tool icon + label on line 2 |
| Solid theme | An opaque theme style with box shadows |
| Stale | Session state indicating no activity for 10 minutes |
| Tool label | A human-readable description extracted from tool input (e.g., filename, command description) |
| Waiting | Session state indicating the agent has explicitly asked for user input (idle_prompt) |
