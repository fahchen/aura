@ui @indicator
Feature: HUD Indicator
  As a developer
  I want a minimal floating indicator on my screen
  So that I have ambient awareness without losing screen space

  Rule: Indicator is always visible as a 36x36 circle

    Scenario: Indicator appears on daemon start
      When the Aura daemon starts
      Then a 36x36 pixel circular indicator appears
      And it is positioned centered under the menu bar

  Rule: Indicator icon reflects session state

    Scenario: No active sessions shows panda
      Given no sessions exist
      Then the indicator shows the Panda icon
      And the indicator opacity is 0.5

    Scenario: Running sessions cycle through creative icons
      Given at least one session is in "Running" state
      Then the indicator cycles through 11 creative icons
      And icons change every 2.5 seconds with a horizontal slide transition

    Scenario: Attention state shows bell icon with shake
      Given at least one session is in "Attention" state
      Then the indicator shows the BellRing icon
      And the indicator shakes horizontally

    Scenario: Waiting state shows spinning fan
      Given at least one session is in "Waiting" state
      Then the indicator shows the Fan icon
      And it spins counter-clockwise over 2 seconds

    Scenario: Stale state shows ghost with breathe animation
      Given all sessions are in "Stale" state
      Then the indicator shows the Ghost icon
      And the indicator opacity oscillates between 0.5 and 0.3

  Rule: Click toggles the session list

    Scenario: Click opens session list
      Given the session list is collapsed
      And at least one session exists
      When the user clicks the indicator
      Then the session list expands

    Scenario: Click closes session list
      Given the session list is expanded
      When the user clicks the indicator
      Then the session list collapses

    Scenario: Click ignored when no sessions
      Given no sessions exist
      When the user clicks the indicator
      Then nothing happens

  Rule: Right-click cycles the theme

    Scenario: Right-click cycles to next theme
      Given the current theme is "Liquid Dark"
      When the user right-clicks the indicator
      Then the theme changes to the next in the cycle

    Scenario: Right-click does not toggle session list
      Given the session list is collapsed
      When the user right-clicks the indicator
      Then the session list remains collapsed

  Rule: Indicator is draggable

    Scenario: Drag repositions the indicator
      When the user clicks and drags the indicator
      Then the indicator moves to the new position
      And the session list follows the indicator position

    Scenario: Small movements are treated as clicks not drags
      When the user clicks and moves less than 5 pixels
      Then it is treated as a click, not a drag

  Rule: Indicator position persists across restarts

    Scenario: Position saved on drag
      When the user drags the indicator to a new position
      Then the position is saved to disk

    Scenario: Position restored on startup
      Given a saved indicator position exists
      When the daemon starts
      Then the indicator appears at the saved position

  Rule: Hover enhances the indicator

    Scenario: Hover effect
      When the user hovers over the indicator
      Then the indicator scales to 1.08x
      And the shadow is enhanced
