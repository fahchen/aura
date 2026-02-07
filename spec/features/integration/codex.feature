@integration @codex
Feature: Codex Integration
  As a developer using Codex
  I want Aura to monitor my sessions via app-server
  So that the HUD reflects what Codex is doing in real time

  Background:
    Given the Aura daemon is running

  Rule: Codex communicates via JSON-RPC over stdio

    Scenario: Daemon spawns Codex app-server
      When the daemon starts
      Then it spawns a "codex app-server" subprocess
      And communicates via JSON-RPC over stdio

    Scenario: Initialize handshake
      When the app-server subprocess starts
      Then Aura sends an "initialize" request
      And follows with an "initialized" notification

  Rule: Thread events map to session events

    Scenario: Thread started creates a session
      When a "thread/started" notification arrives with thread_id "t1"
      Then session "t1" is created with agent type "Codex"

    Scenario: Turn started marks session running
      Given session "t1" exists
      When a "turn/started" notification arrives
      Then session "t1" state is "Running"

    Scenario: Turn completed marks session idle
      Given session "t1" is in "Running" state
      When a "turn/completed" notification arrives
      Then session "t1" state is "Idle"

    Scenario: Item started tracks a tool
      Given session "t1" is in "Running" state
      When an "item/started" notification arrives with type "commandExecution"
      Then session "t1" has a running tool

    Scenario: Item completed removes a tool
      Given session "t1" has a running tool with id "i1"
      When an "item/completed" notification arrives for item "i1"
      Then session "t1" no longer has that running tool

  Rule: Approval requests trigger Attention state

    Scenario: Command execution approval
      When an "item/commandExecution/requestApproval" request arrives
      Then session state is "Attention"

    Scenario: File change approval
      When an "item/fileChange/requestApproval" request arrives
      Then session state is "Attention"

    Scenario: User input request triggers Waiting
      When a "tool/requestUserInput" request arrives
      Then session state is "Waiting"

  Rule: Session names are derived from turn previews

    Scenario: Name extracted from turn/started preview
      Given session "t1" exists
      When a "turn/started" notification arrives with a preview message
      Then session "t1" name is updated from the preview text

  Rule: Thread discovery finds existing sessions

    Scenario: Discovering existing threads
      When Aura polls "thread/list" periodically
      And a thread exists that Aura is not tracking
      Then Aura resumes the thread and creates a session

  Rule: Reconnection with exponential backoff

    Scenario: App-server disconnects
      When the Codex app-server process exits unexpectedly
      Then Aura attempts reconnection with exponential backoff
      And backoff increases from 1 second up to 60 seconds

  Rule: Stale Codex sessions are not auto-removed

    Scenario: Codex session goes stale
      Given session "t1" is in "Idle" state
      When no events are received for 10 minutes
      Then session "t1" state is "Stale"
      And session "t1" remains in the registry
