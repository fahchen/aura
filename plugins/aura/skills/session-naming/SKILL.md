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

### Session name = Overall goal, NOT current step

The session name should describe **what we're trying to accomplish**, not what we're currently doing.

- Good: "Add Dark Mode" (stays the same while reading files, writing code, testing)
- Bad: "Reading Config" → "Writing Theme" → "Running Tests" (changing per step)

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
| 2-4 words max | "Fix Login" | "Fix the authentication bug" |
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

## Patterns

- `Fix [Thing]` → "Fix Login", "Fix Cart"
- `Add [Thing]` → "Add Export", "Add Dark Mode"
- `Refactor [Area]` → "Refactor Auth", "Refactor API"
- `Update [Thing]` → "Update Deps", "Update Config"
