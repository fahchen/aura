@session
Feature: Session Lifecycle
  As a developer running AI coding agents
  I want Aura to track session states accurately
  So that I can glance at the HUD and know what each agent is doing

  Background:
    Given the Aura daemon is running

  Rule: Sessions start fresh on daemon launch

    Scenario: No sessions restored on startup
      When the daemon starts
      Then the session registry is empty
      And no sessions are loaded from disk or files

  Rule: Six distinct session states

    Scenario: Running state when agent is actively working
      Given a session "abc" exists
      When the session receives a tool_started event
      Then the session state is "Running"

    Scenario: Idle state when agent stops
      Given a session "abc" is in "Running" state
      When the session receives a Stop event
      Then the session state is "Idle"

    Scenario: Attention state when permission is needed
      Given a session "abc" is in "Running" state
      When the session receives a needs_attention event
      Then the session state is "Attention"

    Scenario: Waiting state when agent awaits user input
      Given a session "abc" is in "Running" state
      When the session receives a waiting_for_input event
      Then the session state is "Waiting"

    Scenario: Compacting state during context compaction
      Given a session "abc" is in "Running" state
      When the session receives a compacting event
      Then the session state is "Compacting"

    Scenario: Stale state after inactivity timeout
      Given a session "abc" is in "Idle" state
      When no events are received for 10 minutes
      Then the session state is "Stale"

  Rule: Activity resumes sessions from non-running states

    Scenario Outline: Activity revives a non-running session
      Given a session "abc" is in "<from_state>" state
      When the session receives an activity or tool_started event
      Then the session state is "Running"

      Examples:
        | from_state |
        | Idle       |
        | Stale      |
        | Attention  |
        | Waiting    |
        | Compacting |

  Rule: Sessions end cleanly

    Scenario: Session removed on session_ended event
      Given a session "abc" exists
      When the session receives a session_ended event
      Then the session is removed from the registry

  Rule: Stale detection uses per-session timers

    Scenario: Timer resets on new events
      Given a session "abc" is in "Idle" state
      And the stale timer is counting
      When the session receives an activity event
      Then the stale timer resets

    Scenario: Timer fires after inactivity
      Given a session "abc" is in "Idle" state
      When no events are received for 10 minutes
      Then the session transitions to "Stale"

    Scenario: Running sessions do not go stale
      Given a session "abc" is in "Running" state
      When no events are received for 10 minutes
      Then the session state remains "Running"

    Scenario: Stale sessions are not auto-removed
      Given a session "abc" is in "Stale" state
      When time passes
      Then the session remains in the registry as "Stale"
