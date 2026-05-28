Feature: Daily Retailer Updates
  As a user
  I want retailers to be updated daily
  So that the sourcing system has the latest data

  Background:
    Given the system is running

  Scenario: Update retailers daily via command
    When I execute the `update retailers daily` command
    Then retailers are updated with the latest data

  Scenario: Emit sitemap update requested event
    When a retailer sitemap update is triggered
    Then a `sitemap update requested` event is emitted

  Scenario: Emit website update requested event
    When a retailer website update is triggered
    Then a `website update requested` event is emitted
