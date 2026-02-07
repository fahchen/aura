@ui @session-list
Feature: Session List
  As a developer
  I want to see details of all active AI sessions
  So that I can understand what each agent is doing at a glance

  Background:
    Given the Aura daemon is running

  Rule: Session list expands and collapses on demand

    Scenario: User opens session list via indicator click
      Given at least one session exists
      When the user clicks the indicator
      Then the session list window appears below the indicator

    Scenario: User closes session list via indicator click
      Given the session list is expanded
      When the user clicks the indicator
      Then the session list window hides

    Scenario: Auto-collapse when last session ends
      Given the session list is expanded with 1 session
      When the last session ends
      Then the session list collapses automatically

    Scenario: No auto-expand when first session appears
      Given no sessions exist
      And the session list is collapsed
      When a new session starts
      Then the session list remains collapsed

  Rule: Session rows display two-line layout

    Scenario: Session row shows state and name
      Given a session exists with name "fix login bug" in "Running" state
      Then the session row shows the Running state icon and "fix login bug"

    Scenario: Session row shows tool activity
      Given a session has a running tool "Read" with label "main.rs"
      Then the session row second line shows the Read tool icon and "main.rs"

    Scenario: Session without custom name shows directory name
      Given a session exists with cwd "/home/user/my-project" and no custom name
      Then the session row shows "my-project" as the name

  Rule: State placeholders appear when no tools are running

    Scenario: Running session with no tools shows thinking placeholder
      Given a session is in "Running" state with no running tools
      Then the second line shows a random placeholder from "thinking...", "drafting...", "building...", etc.
      And the placeholder is stable per session_id

    Scenario: Idle session shows waiting timestamp
      Given a session is in "Idle" state since "Jan 17, 14:30"
      Then the second line shows "waiting since Jan 17, 14:30"

    Scenario: Stale session shows inactive timestamp
      Given a session is in "Stale" state since "Jan 17, 10:00"
      Then the second line shows "inactive since Jan 17, 10:00"

    Scenario: Attention session shows permission tool
      Given a session is in "Attention" state with permission tool "Bash"
      Then the second line shows "Bash needs permission"

    Scenario: Waiting session shows input prompt
      Given a session is in "Waiting" state
      Then the second line shows "waiting for input"

    Scenario: Compacting session shows compacting message
      Given a session is in "Compacting" state
      Then the second line shows "compacting context..."

  Rule: Sessions can be manually removed

    Scenario: Hover reveals remove icon
      When the user hovers over a session row
      Then the state icon swaps to a Bomb icon with a 300ms slide transition

    Scenario: Click bomb removes session
      Given the user is hovering over a session row
      When the user clicks the Bomb icon
      Then the session is removed from the HUD

  Rule: Session list has size constraints

    Scenario: Maximum 5 sessions visible without scrolling
      Given 7 sessions exist
      Then the session list shows 5 sessions
      And the remaining sessions are accessible by scrolling

    Scenario: Session list height adapts to session count
      Given 3 sessions exist
      Then the session list height fits exactly 3 session rows

  Rule: Session rows animate on appearance and removal

    Scenario: New session slides in from left
      When a new session appears
      Then the session row slides in from the left over 350ms

    Scenario: Removed session slides out to right
      When a session is removed
      Then the session row slides out to the right over 300ms
