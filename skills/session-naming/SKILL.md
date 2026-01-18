---
name: Aura Session Naming
description: Set descriptive names for Aura HUD session display
version: 1.0.0
---

# Aura Session Naming

Set descriptive session names in the Aura HUD to help users track what each session is working on.

## Command

```bash
aura set-name "<name>"
```

## Rules

### Session name = Overall goal, optionally with current step

The session name should describe **what we're trying to accomplish**. For larger tasks with distinct phases, use `Task - Step` format.

- Simple task: "Fix Login" (stays the same throughout)
- Multi-phase task: "Add Auth - Write Tests" → "Add Auth - Integration" (task stays, step updates)

Use `Task - Step` format when:
- The task has 3+ distinct phases (design, implement, test, deploy)
- Each phase takes significant time
- User benefits from knowing current progress

### MUST set session name when:
1. User requests a task (e.g., "fix the login bug", "add dark mode")
2. Starting work from a plan file
3. User explicitly asks to name the session

### DO NOT set/change session name when:
- Exploring code or answering questions
- Moving to the next step of the same task
- Running commands as part of the current task
- Name already reflects the current goal

## Naming Rules

| Rule | Good | Bad |
|------|------|-----|
| 2-6 words | "Fix Login Auth Flow" | "Fix the authentication bug in the login system" |
| Start with verb | "Add Export" | "Export Feature" |
| No filler words | "Refactor API" | "Refactor the API Layer" |
| No paths/files | "Update Config" | "Update src/config.ts" |
| No jargon | "Speed Up Search" | "Optimize O(n²)" |

## Examples

| User Request | Session Name |
|--------------|--------------|
| "Fix the bug where login fails" | "Fix Login" |
| "Add a button to export data" | "Add Export" |
| "Refactor the authentication code" | "Refactor Auth" |
| "Add dark mode to the app" | "Add Dark Mode" |
| "Help me understand this code" | (don't set) |
| "Run the tests" | (don't set) |
| Large feature with phases | "Add Auth - Design" → "Add Auth - Implement" → "Add Auth - Test" |

## Patterns

- `Fix [Thing]` → "Fix Login", "Fix Cart"
- `Add [Thing]` → "Add Export", "Add Dark Mode"
- `Refactor [Area]` → "Refactor Auth", "Refactor API"
- `Update [Thing]` → "Update Deps", "Update Config"
- `[Task] - [Step]` → "Add Auth - Design", "Add Auth - Implement", "Add Auth - Test"
