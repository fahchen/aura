@ui @theme
Feature: Theme System
  As a developer
  I want Aura to match my desktop aesthetic
  So that the HUD blends naturally with my environment

  Rule: Five themes are available

    Scenario Outline: Theme "<theme>" applies its visual style
      When the theme is set to "<theme>"
      Then the HUD renders with "<style>" style

      Examples:
        | theme        | style                              |
        | System       | Auto-detected from OS appearance   |
        | Liquid Dark  | Transparent glass on dark, shadows |
        | Liquid Light | Transparent glass on light, shadows |
        | Solid Dark   | Opaque dark background, shadows    |
        | Solid Light  | Opaque light background, shadows   |

  Rule: Liquid themes use transparent glass without backdrop blur

    Scenario: Liquid theme renders without blur
      When the theme is "Liquid Dark"
      Then the HUD background is transparent with glass gradients
      And no backdrop blur is applied
      And shadows are rendered for depth

  Rule: Solid themes use opaque backgrounds with shadows

    Scenario: Solid theme renders opaque
      When the theme is "Solid Dark"
      Then the HUD background is opaque
      And box shadows are rendered for depth

  Rule: Color system is achromatic

    Scenario: All colors are grayscale
      When any theme is active
      Then all HUD colors use hue 0 and saturation 0
      And colors are defined as lightness and alpha pairs

  Rule: Theme persists across restarts

    Scenario: Selected theme is remembered
      Given the user selects "Solid Dark" theme
      When the daemon restarts
      Then the theme is "Solid Dark"

  Rule: Theme can be switched via multiple methods

    Scenario: Switch via menu
      When the user opens the "Aura" menu and selects a theme from "Theme" submenu
      Then the theme changes

    Scenario: Switch via triple-click
      When the user triple-clicks the indicator
      Then the theme cycles to the next option
