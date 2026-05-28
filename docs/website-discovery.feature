Feature: Website Discovery
  As a user
  I want to discover and process retailer websites
  So that menu and navigation structure can be extracted

  Background:
    Given the system is running
    And the system is subscribed to `website update requested` events

  Scenario: Execute website discovery command
    When I execute the `discover website` command
    Then website discovery process is initiated

  Scenario: Fetch and store homepage
    When a `website update requested` event is received
    Then the retailer homepage is fetched
    And the homepage is stored in the database
    And the homepage is stored on disk

  Scenario: Process website menu into category tree
    When the homepage is processed
    Then the menu structure is parsed
    And a category tree is generated
    And the category tree is stored in the database
