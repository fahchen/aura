@integration @codex
Feature: Codex Integration
  As a developer using Codex
  I want Aura to monitor my sessions via session rollout files
  So that the HUD reflects what Codex is doing in real time

  Background:
    Given the Aura daemon is running

  Rule: Aura watches Codex rollouts on disk

    Scenario: Daemon spawns Codex rollout watcher
      When the daemon starts
      Then it starts watching "~/.codex/sessions/**.jsonl"
      And it publishes best-effort AgentEvents for "Codex"

    Scenario: CODEX_HOME overrides the default path
      Given the environment variable "CODEX_HOME" is set
      When the daemon starts
      Then it watches "$CODEX_HOME/sessions/**.jsonl"

  Rule: Rollout bootstrap is bounded

    Scenario: Recent rollout creates a session and replays a small tail
      Given a rollout file was modified within 10 minutes
      When Aura discovers the rollout
      Then session "sess_1" is created with agent type "Codex"
      And at most 4 recent AgentEvents are replayed to seed the HUD

    Scenario: Stale rollouts are hidden until they change
      Given a rollout file was last modified more than 10 minutes ago
      When Aura discovers the rollout
      Then no session is created for that rollout
      But the rollout remains watched for future changes

  Rule: Rollout lines map to AgentEvents

    Scenario: session_meta starts a session
      When a "session_meta" line arrives with id "sess_1"
      Then session "sess_1" is created with agent type "Codex"

    Scenario: function_call starts a tool
      Given session "sess_1" exists
      When a response item "function_call" arrives with call_id "call_1"
      Then session "sess_1" has a running tool "call_1"

    Scenario: function_call_output completes a tool
      Given session "sess_1" has a running tool "call_1"
      When a response item "function_call_output" arrives for call_id "call_1"
      Then session "sess_1" no longer has that running tool

    Scenario: task_complete marks the session idle
      Given session "sess_1" is in "Running" state
      When an event_msg line arrives with type "task_complete"
      Then session "sess_1" state is "Idle"

    Scenario: request_user_input marks the session waiting
      Given session "sess_1" exists
      When an event_msg line arrives with type "request_user_input"
      Then session "sess_1" state is "Waiting"

    Scenario: context_compacted marks the session compacting
      Given session "sess_1" exists
      When an event_msg line arrives with type "context_compacted"
      Then session "sess_1" state is "Compacting"

  Rule: Session names are derived from aura set-name

    Scenario: aura set-name updates the session name
      Given session "sess_1" exists
      When an exec_command tool call runs "aura set-name \"fix login bug\""
      Then session "sess_1" name is "fix login bug"

  Rule: Rollout watching is best-effort

    Scenario: Missed filesystem events do not crash the daemon
      When filesystem notifications are dropped
      Then Aura triggers a rescan of the sessions directory
      And continues tailing rollouts best-effort

    Scenario: Codex session goes stale
      Given session "t1" is in "Idle" state
      When no events are received for 10 minutes
      Then session "t1" state is "Stale"
      And session "t1" remains in the registry
