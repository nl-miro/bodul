Feature: Sitemap Discovery
  As a user
  I want to discover and process retailer sitemaps
  So that product and category information can be extracted

  Background:
    Given the system is running
    And the system is subscribed to `sitemap update requested` events

  Scenario: Execute sitemap discovery command
    When I execute the `discover sitemap` command
    Then sitemap discovery process is initiated

  Scenario: Fetch and store sitemap files
    When a `sitemap update requested` event is received
    Then sitemap files are fetched from the retailer
    And sitemap files are stored in the database
    And sitemap files are stored on disk

  Scenario: Process sitemap for categories
    When sitemap files are processed
    Then categories are extracted from the sitemap
    And categories are stored in the database

  Scenario: Process sitemap for products
    When sitemap files are processed
    Then products are extracted from the sitemap
    And products are stored in the database

  Scenario: Process sitemap for misc information
    When sitemap files are processed
    Then miscellaneous information is extracted
    And miscellaneous information is stored in the database
