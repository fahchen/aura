@ui @animation
Feature: Animation System
  As a developer
  I want smooth animations in the HUD
  So that state changes are visually clear without being distracting

  Rule: Tool cycling uses vertical slide animation

    Scenario: Multiple tools cycle with vertical slide
      Given a session has 3 running tools
      Then the tool display cycles through them
      And each transition uses a vertical slide (new slides up from below, old slides up and out)
      And the cycle interval is randomized between 1500ms and 2000ms

  Rule: Indicator icons cycle with horizontal slide

    Scenario: Running state cycles creative icons
      Given at least one session is in "Running" state
      Then the indicator cycles through 11 creative icons
      And each transition uses a horizontal slide
      And the cycle interval is 2500ms

  Rule: Attention state triggers shake animation

    Scenario: Shake on attention
      Given at least one session is in "Attention" state
      Then the indicator oscillates horizontally by 1.5 pixels
      And the shake period is 150ms

  Rule: Stale state triggers breathe animation

    Scenario: Breathe on stale
      Given all sessions are in "Stale" state
      Then the indicator opacity oscillates between 0.5 and 0.3
      And the breathe cycle is 4000ms using a sine wave

  Rule: Waiting state triggers spin animation

    Scenario: Spin on waiting
      Given at least one session is in "Waiting" state
      Then the indicator Fan icon spins counter-clockwise
      And the rotation period is 2 seconds

  Rule: Session rows animate with slide transitions

    Scenario: Row appears with slide-in
      When a new session row appears
      Then it slides in from the left starting at -12px offset
      And fades in over 350ms

    Scenario: Row disappears with slide-out
      When a session row is removed
      Then it slides out to the right by +12px offset
      And fades out over 300ms

  Rule: Icon swap animates on hover

    Scenario: State icon swaps to bomb on hover
      When the user hovers over a session row
      Then the state icon transitions to the Bomb icon
      And the transition uses a 300ms slide and fade
