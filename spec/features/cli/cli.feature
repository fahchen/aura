@cli
Feature: CLI
  As a developer
  I want a simple command-line interface for Aura
  So that I can start the HUD and integrate with agents

  Rule: Default command starts the HUD daemon

    Scenario: Start daemon
      When the user runs "aura"
      Then the HUD daemon starts
      And the indicator window appears
      And the Unix socket server begins listening
      And the Codex rollout watcher starts

  Rule: Verbosity controls log output

    Scenario Outline: Verbosity flag sets log level
      When the user runs "aura <flag>"
      Then the log level is "<level>"

      Examples:
        | flag | level |
        | -v   | info  |
        | -vv  | debug |
        | -vvv | trace |

    Scenario: Default verbosity is warn
      When the user runs "aura" without flags
      Then the log level is "warn"

  Rule: set-name is a stub that exits successfully

    Scenario: set-name prints and exits
      When the user runs "aura set-name \"fix login bug\""
      Then the command prints a confirmation message
      And exits with code 0

    # Note: The actual name update happens via hook parsing when
    # Claude Code's PreToolUse hook intercepts this Bash command.

  Rule: hook subcommand forwards events to daemon

    Scenario: Hook reads stdin and sends to socket
      When the user runs "aura hook --agent claude-code"
      And provides hook JSON via stdin
      Then the hook CLI parses the JSON into AgentEvents
      And sends them to the daemon Unix socket

    Scenario Outline: Hook supports multiple agent types
      When the user runs "aura hook --agent <agent>"
      Then the hook CLI accepts the agent type

      Examples:
        | agent       |
        | claude-code |

    # Note: gemini-cli and open-code are future features
