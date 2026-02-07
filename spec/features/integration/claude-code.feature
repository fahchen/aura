@integration @claude-code
Feature: Claude Code Integration
  As a developer using Claude Code
  I want Aura to monitor my sessions via hooks
  So that the HUD reflects what Claude Code is doing in real time

  Background:
    Given the Aura daemon is running
    And the aura plugin hooks are installed

  Rule: Hook events map to session events

    Scenario: SessionStart creates a new session
      When a "SessionStart" hook fires with session "abc" and cwd "/projects/myapp"
      Then session "abc" is created with agent type "ClaudeCode"

    Scenario: PreToolUse starts a tool
      Given session "abc" exists
      When a "PreToolUse" hook fires with tool "Read" and tool_use_id "t1"
      Then session "abc" has a running tool "Read" with id "t1"

    Scenario: PostToolUse completes a tool
      Given session "abc" has a running tool with id "t1"
      When a "PostToolUse" hook fires with tool_use_id "t1"
      Then session "abc" no longer has a running tool with id "t1"

    Scenario: PostToolUseFailure completes a tool
      Given session "abc" has a running tool with id "t1"
      When a "PostToolUseFailure" hook fires with tool_use_id "t1"
      Then session "abc" no longer has a running tool with id "t1"

    Scenario: Notification with permission_prompt triggers Attention
      When a "Notification" hook fires with notification_type "permission_prompt" and tool_name "Bash"
      Then session state is "Attention"
      And the permission tool is "Bash"

    Scenario: Notification with idle_prompt triggers Waiting
      When a "Notification" hook fires with notification_type "idle_prompt"
      Then session state is "Waiting"

    Scenario: Stop triggers Idle
      When a "Stop" hook fires
      Then session state is "Idle"

    Scenario: PreCompact triggers Compacting
      When a "PreCompact" hook fires
      Then session state is "Compacting"

    Scenario: SessionEnd removes session
      When a "SessionEnd" hook fires for session "abc"
      Then session "abc" is removed

    Scenario: UserPromptSubmit triggers Activity
      When a "UserPromptSubmit" hook fires
      Then session state is "Running"

  Rule: Hook events are received via Unix socket

    Scenario: Hook CLI forwards events to daemon
      When Claude Code invokes "aura hook --agent claude-code"
      And passes hook JSON via stdin
      Then the hook CLI parses the JSON and sends AgentEvents to the Unix socket

  Rule: Session naming is parsed from Bash tool hook events

    Scenario: Extracting session name from aura set-name command
      Given session "abc" exists
      When a "PreToolUse" hook fires with tool "Bash" and command "aura set-name \"fix login bug\""
      Then session "abc" name is updated to "fix login bug"
      And a SessionNameUpdated event is emitted

    Scenario: Non-set-name Bash commands are not parsed for names
      Given session "abc" exists
      When a "PreToolUse" hook fires with tool "Bash" and command "npm test"
      Then session "abc" name is not changed

    Scenario: ToolStarted event is still emitted for set-name commands
      Given session "abc" exists
      When a "PreToolUse" hook fires with tool "Bash" and command "aura set-name \"fix login bug\""
      Then a ToolStarted event is emitted for "Bash"
      And a SessionNameUpdated event is also emitted

  Rule: Tool labels are extracted for all known tools

    Scenario Outline: Tool-specific label extraction
      When a "PreToolUse" hook fires with tool "<tool>" and input containing "<field>" = "<value>"
      Then the tool label is "<expected_label>"

      Examples:
        | tool             | field       | value                              | expected_label         |
        | Bash             | description | Run test suite                     | Run test suite         |
        | Bash             | command     | npm test                           | npm test               |
        | Read             | file_path   | /home/user/project/src/main.rs     | main.rs                |
        | Write            | file_path   | /home/user/project/src/lib.rs      | lib.rs                 |
        | Edit             | file_path   | /home/user/project/src/server.rs   | server.rs              |
        | Glob             | pattern     | **/*.ts                            | **/*.ts                |
        | Grep             | pattern     | TODO.*fix                          | TODO.*fix              |
        | WebFetch         | url         | https://example.com/api            | https://example.com/api |
        | WebSearch        | query       | react hooks best practices         | react hooks best practices |
        | Task             | description | Find API endpoints                 | Find API endpoints     |
        | NotebookEdit     | notebook_path | /home/user/notebook.ipynb        | notebook.ipynb         |
        | AskUserQuestion  | questions   | (any)                              | (tool name fallback)   |
        | EnterPlanMode    | (none)      | (none)                             | (tool name fallback)   |
        | Skill            | skill       | commit                             | commit                 |

  Rule: Subagent events are ignored

    Scenario: SubagentStart is not forwarded
      When a "SubagentStart" hook fires
      Then no AgentEvent is emitted

    Scenario: SubagentStop is not forwarded
      When a "SubagentStop" hook fires
      Then no AgentEvent is emitted
