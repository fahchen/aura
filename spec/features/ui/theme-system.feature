@ui @theme
Feature: Theme System
  As a developer
  I want Aura to match my desktop aesthetic
  So that the HUD blends naturally with my environment

  Rule: Three themes are available

    Scenario Outline: Theme "<theme>" applies its visual style
      When the theme is set to "<theme>"
      Then the HUD renders with "<style>" style

      Examples:
        | theme        | style                                |
        | System       | Auto-detected from OS appearance     |
        | Liquid Dark  | Transparent glass on dark background |
        | Liquid Light | Transparent glass on light background |

  Rule: Liquid themes use transparent glass without backdrop blur

    Scenario: Liquid theme renders without blur
      When the theme is "Liquid Dark"
      Then the HUD background is transparent with glass gradients
      And no backdrop blur is applied

  Rule: Color system is achromatic

    Scenario: All colors are grayscale
      When any theme is active
      Then all HUD colors use hue 0 and saturation 0
      And colors are defined as lightness and alpha pairs

  Rule: Theme persists across restarts

    Scenario: Selected theme is remembered
      Given the user selects "Liquid Dark" theme
      When the daemon restarts
      Then the theme is "Liquid Dark"

  Rule: Theme can be switched via multiple methods

    Scenario: Switch via menu
      When the user opens the "Aura" menu and selects a theme from "Theme" submenu
      Then the theme changes

    Scenario: Switch via right-click
      When the user right-clicks the indicator
      Then the theme cycles to the next option
