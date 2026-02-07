@tracking
Feature: Tool Tracking
  As a developer
  I want to see what tools my AI agents are using
  So that I have ambient awareness of agent activity

  Background:
    Given the Aura daemon is running
    And a session "abc" exists in "Running" state

  Rule: Running tools are tracked in real time

    Scenario: Single tool running
      When tool "Read" starts with id "t1" and label "main.rs"
      Then session "abc" shows 1 running tool
      And the tool displays as "main.rs"

    Scenario: Multiple tools running simultaneously
      When tool "Read" starts with id "t1" and label "main.rs"
      And tool "Bash" starts with id "t2" and label "npm test"
      Then session "abc" shows 2 running tools

    Scenario: Tool completion removes from running list
      Given tool "Read" is running with id "t1"
      When tool "t1" completes
      Then session "abc" shows 0 running tools

  Rule: Recent tools persist for minimum display duration

    Scenario: Completed tool stays visible briefly
      Given tool "Read" is running with id "t1" and label "main.rs"
      When tool "t1" completes
      Then "main.rs" remains visible for 1 second
      And then disappears from the display

  Rule: Recent activity maintains a history queue

    Scenario: Activity labels accumulate
      When tool "Read" starts and completes with label "main.rs"
      And tool "Bash" starts and completes with label "npm test"
      And tool "Edit" starts and completes with label "server.rs"
      Then the recent activity queue contains "main.rs", "npm test", "server.rs"

    Scenario: Queue is capped at 6 entries
      When 8 tools start and complete with distinct labels
      Then the recent activity queue contains the last 6 labels

    Scenario: Duplicate consecutive labels are deduplicated
      When tool "Read" completes with label "main.rs"
      And tool "Read" completes with label "main.rs"
      Then the recent activity queue has one entry for "main.rs"

    Scenario: Recent activity cycles when no tools are running
      Given no tools are running
      And the recent activity queue has 3 entries
      Then the display rotates through recent activity labels

  Rule: MCP tools are formatted with server prefix

    Scenario: MCP tool displays as server: function
      When tool "mcp__github__search_repositories" starts with label "react hooks"
      Then the tool displays as "github: react hooks"

    Scenario: MCP tool without label shows server: function name
      When tool "mcp__memory__create_entities" starts without a label
      Then the tool displays as "memory: create_entities"
