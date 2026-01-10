# Aura Plugin for Claude Code

Real-time situational awareness HUD for AI code agent sessions.

## Features

- **Session tracking**: Monitor multiple concurrent AI coding sessions
- **Tool visibility**: See which tools are currently running with animated display
- **State indicators**: Running, Idle, Attention, Compacting, Stale states
- **Attention alerts**: Visual indicator when permission is needed

## Prerequisites

1. **Build the hook handler**:
   ```bash
   cd /path/to/aura
   cargo build --release -p aura-claude-code-hook
   ```

2. **Add to PATH**:
   ```bash
   # Add to your shell config (~/.zshrc, ~/.bashrc, etc.)
   export PATH="/path/to/aura/target/release:$PATH"
   ```

3. **Start the Aura daemon**:
   ```bash
   cargo run -p aura-daemon
   ```

## Installation

```bash
# From Claude Code
/plugin install /path/to/aura/plugins/aura
```

Or test during development:
```bash
claude --plugin-dir /path/to/aura/plugins/aura
```

## Hook Events

This plugin responds to all Claude Code lifecycle events:

| Event | Description |
|-------|-------------|
| SessionStart | New session begins |
| UserPromptSubmit | User submits a prompt |
| PreToolUse | Before tool execution |
| PostToolUse | After tool completion |
| Notification | Agent notifications |
| Stop | Agent stops responding |
| PreCompact | Context compaction |
| SubagentStop | Subagent stops |
| SessionEnd | Session ends |

## How It Works

```
Claude Code → Hook Event (stdin JSON)
                    ↓
          aura-claude-code-hook (parses event)
                    ↓
          Unix socket IPC → Aura Daemon
                    ↓
          HUD updates in real-time
```

## Troubleshooting

**HUD not updating?**
- Ensure `aura-claude-code-hook` is in your PATH
- Ensure the Aura daemon is running
- Check daemon output for errors

**Permission errors?**
- The hook gracefully fails if the daemon isn't running
- Claude Code continues normally even if hooks fail
