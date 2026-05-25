mod common;

use dotenvy::dotenv;
use retailer_sourcing::io::{
    write_entry_error_trace_report,
    write_entry_error_trace_report_with_suffix, //
};
use std::env::var;

const DEFAULT_TEST_DATABASE_URL: &str =
    "postgres://postgres:postgres@localhost:26001/retailer_sourcing_test";

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();

    let main_pool = common::database_pool()?;
    let main_report = write_entry_error_trace_report(&main_pool)?;

    let test_database_url = var("TEST_DATABASE_URL")
        .or_else(|_| var("DATABASE_URL_TEST"))
        .unwrap_or_else(|_| DEFAULT_TEST_DATABASE_URL.to_string());
    let test_pool = common::database_pool_from_url(&test_database_url)?;
    let test_report = write_entry_error_trace_report_with_suffix(&test_pool, Some("test"))?;

    println!(
        "Entry error trace report written to {}",
        main_report.display()
    );
    println!(
        "Entry error trace test report written to {}",
        test_report.display()
    );

    Ok(())
}
