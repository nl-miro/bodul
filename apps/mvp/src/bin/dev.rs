//! Developer utilities for inspecting DB data.
//!
//! Dumps a random sample of raw bodies out of Postgres and onto disk so parsers
//! and classifiers can be eyeballed against real retailer content. With no
//! `--retailer` the sample is drawn across all retailers; pass one or more to
//! target them:
//!
//! ```text
//! dev dump-sitemap 5                       # -> data/dumps/sitemaps/{Retailer}/{name}.xml
//! dev dump-offer 5 -r ekupihr -r bazzarhr  # -> data/dumps/offers/{Retailer}/{name}.html
//! ```
//!
//! It also reprocesses dumped offer pages (no DB needed), reusing the retailer
//! architectures. `count` here limits pages **per retailer** (0 = all):
//!
//! ```text
//! dev destructure 5 -r admhr   # -> data/offers-destructed/admhr/*.json  (5 pages)
//! dev valueless -r admhr       # -> data/offers-valueless/admhr/… (+ -segments/, all pages)
//! ```

use bodul_html_extractor::html_extractor::ExtractionResult;
use bodul_html_extractor::{ExtractError, html_extractor};
use bodul_shared::retailer::RetailerCode;
use clap::{Parser, Subcommand};
use development::io::RetailerOutcome;
use diesel::prelude::*;
use diesel::sql_types::Text;
use mvp::database::{DatabaseConfig, connect};
use mvp::html_parser;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(about = "Developer utilities for inspecting DB data")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Dump N random raw sitemap bodies to data/dumps/sitemaps/{retailer}/
    DumpSitemap {
        #[arg(default_value = "10")]
        count: usize,
        /// Retailer slug(s) to target (e.g. ekupihr); repeatable. Omit for all.
        #[arg(short, long = "retailer")]
        retailers: Vec<String>,
    },
    /// Dump N random raw offer bodies per retailer to data/dumps/offers/{retailer}/
    DumpOffer {
        /// Max pages per retailer to dump.
        #[arg(default_value = "10")]
        count: usize,
        /// Retailer slug(s) to target (e.g. ekupihr); repeatable. Omit for all.
        #[arg(short, long = "retailer")]
        retailers: Vec<String>,
    },
    /// Destructure dumped offer pages into data/offers-destructed/{retailer}/{page}.json
    Destructure {
        /// Max pages per retailer to process (0 = all).
        #[arg(default_value = "0")]
        count: usize,
        /// Retailer slug(s) to target (e.g. ankercom); repeatable. Omit for all.
        #[arg(short, long = "retailer")]
        retailers: Vec<String>,
    },
    /// Extract dumped offer pages into data/offers-destructed/{retailer}/{page}.json
    Extract {
        /// Max pages per retailer to process (0 = all).
        #[arg(default_value = "0")]
        count: usize,
        /// Retailer slug(s) to target (e.g. ankercom); repeatable. Omit for all.
        #[arg(short, long = "retailer")]
        retailers: Vec<String>,
    },
    /// Blank dumped offer pages into data/offers-valueless/{retailer}/ (+ -segments/)
    Valueless {
        /// Max pages per retailer to process (0 = all).
        #[arg(default_value = "0")]
        count: usize,
        /// Retailer slug(s) to target (e.g. ankercom); repeatable. Omit for all.
        #[arg(short, long = "retailer")]
        retailers: Vec<String>,
    },
}

/// One dumped record: which retailer it belongs to, its source URL, and the raw
/// (already-decompressed) body to write.
#[derive(QueryableByName)]
struct DumpRow {
    #[diesel(sql_type = Text)]
    retailer_code: String,
    #[diesel(sql_type = Text)]
    url: String,
    #[diesel(sql_type = Text)]
    body: String,
}

fn main() {
    dotenvy::dotenv().ok();
    let _guard = mvp::logging::init();
    let cli = Cli::parse();

    match cli.command {
        Command::DumpSitemap { count, retailers } => {
            let mut conn = db_conn();
            let filter = retailer_filter("sr.retailer_code", &retailers);
            let query = format!(
                "SELECT sr.retailer_code, rsd.url, rsd.body \
                 FROM raw_sitemap_documents rsd \
                 JOIN sitemap_retrievals sr ON sr.id = rsd.retrieval_id \
                 {filter} ORDER BY random() LIMIT {count}",
            );
            let rows = diesel::sql_query(query)
                .load::<DumpRow>(&mut conn)
                .expect("raw sitemap sample query");
            dump(&rows, "sitemaps", "xml");
        }
        Command::DumpOffer { count, retailers } => {
            let mut conn = db_conn();
            let filter = retailer_filter("o.retailer_code", &retailers);
            // Sample up to `count` rows per retailer (not `count` total), so an
            // unfiltered dump yields a full sample from every retailer rather than
            // whichever ones happen to win a global random ordering.
            let query = format!(
                "SELECT retailer_code, url, body FROM ( \
                   SELECT o.retailer_code, ro.url, ro.body, \
                          row_number() OVER (PARTITION BY o.retailer_code ORDER BY random()) AS rn \
                   FROM raw_offers ro \
                   JOIN offers o ON o.id = ro.offer_id \
                   {filter} \
                 ) ranked WHERE rn <= {count}",
            );
            let rows = diesel::sql_query(query)
                .load::<DumpRow>(&mut conn)
                .expect("raw offer sample query");
            dump(&rows, "offers", "html");
        }
        Command::Destructure { count, retailers } => process_offers(count, &retailers, Processing::Destructure),
        Command::Extract { count, retailers } => process_offers(count, &retailers, Processing::Extract),
        Command::Valueless { count, retailers } => process_offers(count, &retailers, Processing::Valueless),
    }
}

/// Opens a pooled DB connection from the environment (dump commands only).
fn db_conn() -> diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>> {
    let config = DatabaseConfig::from_env();
    let pool = connect(&config).expect("DB pool");
    pool.get().expect("DB connection")
}

/// Builds a `WHERE {column} IN (…)` clause restricting the sample to the given
/// retailers, or an empty string when none are named (sample across all). Each
/// argument is validated through `RetailerCode` and re-emitted as its canonical
/// slug, so only known, safe values ever reach the SQL. Exits on an unknown slug.
fn retailer_filter(column: &str, retailers: &[String]) -> String {
    if retailers.is_empty() {
        return String::new();
    }

    let mut slugs = Vec::with_capacity(retailers.len());
    for retailer in retailers {
        match RetailerCode::try_from(retailer.to_lowercase().as_str()) {
            Ok(code) => slugs.push(format!("'{}'", code.slug())),
            Err(_) => {
                tracing::error!("unknown retailer slug: {retailer}");
                std::process::exit(1);
            }
        }
    }

    format!("WHERE {column} IN ({})", slugs.join(", "))
}

/// Writes each row's body to `data/dumps/{kind}/{retailer_code}/{name}.{ext}`,
/// grouping per retailer. Logs one line per file and a final summary.
fn dump(rows: &[DumpRow], kind: &str, ext: &str) {
    let base = PathBuf::from("data").join("dumps").join(kind);
    let mut written = 0usize;
    let mut failed = 0usize;

    for (index, row) in rows.iter().enumerate() {
        let dir = base.join(&row.retailer_code);
        if let Err(error) = fs::create_dir_all(&dir) {
            tracing::error!("fail creating {}: {error}", dir.display());
            failed += 1;
            continue;
        }

        let filename = page_filename(&row.url, ext).unwrap_or_else(|| format!("{index}.{ext}"));
        let path = dir.join(filename);
        match fs::write(&path, &row.body) {
            Ok(()) => {
                println!("ok   {}", path.display());
                written += 1;
            }
            Err(error) => {
                tracing::error!("fail writing {}: {error}", path.display());
                failed += 1;
            }
        }
    }

    println!("done: {written} written, {failed} failed");
}

/// Which reprocessing a `process_offers` run performs.
#[derive(Clone, Copy)]
enum Processing {
    /// Destructure pages into JSON under `data/offers-destructed/`.
    Destructure,
    Extract,
    /// Blank pages into `data/offers-valueless/` (+ lifted `-segments/`).
    Valueless,
}

/// Reprocesses dumped offer pages from `data/dumps/offers/{retailer}/`, writing
/// one output per page. `count` caps pages **per retailer** (0 = all); an empty
/// `retailers` processes every dumped retailer dir. Logs per-file and a summary.
fn process_offers(count: usize, retailers: &[String], mode: Processing) {
    let input_root = PathBuf::from("data/dumps/offers");
    let dirs = offer_dirs(&input_root, retailers);

    let limit = if count == 0 { usize::MAX } else { count };
    let mut processed = 0usize;
    let mut failed = 0usize;

    for (input_dir, retailer) in dirs {
        let slug = input_dir.file_name().and_then(|name| name.to_str()).unwrap_or_default();

        // Collect the dir's *.html pages, sorted for deterministic `take(limit)`.
        let mut pages: Vec<PathBuf> = match fs::read_dir(&input_dir) {
            Ok(entries) => entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "html"))
                .collect(),
            Err(error) => {
                tracing::error!("error reading {}/: {error}", input_dir.display());
                failed += 1;
                continue;
            }
        };
        pages.sort();

        for path in pages.into_iter().take(limit) {
            let result = match mode {
                Processing::Destructure => destructure_page(&path, slug, retailer),
                Processing::Extract => extract_page(&path, slug, retailer),
                Processing::Valueless => valueless_page(&path, slug, retailer),
            };
            match result {
                Ok(out_path) => {
                    println!("ok   {} -> {}", path.display(), out_path.display());
                    processed += 1;
                }
                Err(error) => {
                    tracing::error!("fail {}: {error}", path.display());
                    failed += 1;
                }
            }
        }
    }

    println!("done: {processed} processed, {failed} failed");
}

/// The `data/dumps/offers/{slug}` dirs to process. With no `retailers` every
/// subdir whose name is a known slug is returned (unknown ones warned & skipped);
/// otherwise only the requested slugs, each validated via [`RetailerCode`] and
/// exiting on an unknown one (as the SQL path's [`retailer_filter`] does).
fn offer_dirs(base: &Path, retailers: &[String]) -> Vec<(PathBuf, RetailerCode)> {
    if retailers.is_empty() {
        let entries = match fs::read_dir(base) {
            Ok(entries) => entries,
            Err(error) => {
                tracing::error!("error reading {}/: {error}", base.display());
                std::process::exit(1);
            }
        };
        let mut dirs = Vec::new();
        for path in entries.flatten().map(|entry| entry.path()).filter(|path| path.is_dir()) {
            let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
            match RetailerCode::try_from(name) {
                Ok(code) => dirs.push((path, code)),
                Err(_) => tracing::warn!("skip {}: unknown retailer slug", path.display()),
            }
        }
        dirs.sort_by(|a, b| a.0.cmp(&b.0));
        return dirs;
    }

    let mut dirs = Vec::with_capacity(retailers.len());
    for retailer in retailers {
        match RetailerCode::try_from(retailer.to_lowercase().as_str()) {
            Ok(code) => dirs.push((base.join(code.slug()), code)),
            Err(_) => {
                tracing::error!("unknown retailer slug: {retailer}");
                std::process::exit(1);
            }
        }
    }
    dirs
}

/// Destructures one page into `data/offers-destructed/{slug}/{stem}.json`.
fn destructure_page(path: &Path, slug: &str, retailer: RetailerCode) -> Result<PathBuf, String> {
    let html = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let destructured = html_parser::destructure(&html, retailer);
    let json = serde_json::to_string_pretty(&destructured).map_err(|error| error.to_string())?;

    let output_dir = PathBuf::from("data/offers-destructed").join(slug);
    fs::create_dir_all(&output_dir).map_err(|error| format!("creating {}: {error}", output_dir.display()))?;
    let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("page");
    let out_path = output_dir.join(format!("{stem}.json"));
    fs::write(&out_path, json).map_err(|error| format!("writing {}: {error}", out_path.display()))?;
    Ok(out_path)
}

/// Extract one page into `data/offers-extracted/{slug}/{stem}.json`.
fn extract_page(path: &Path, slug: &str, retailer: RetailerCode) -> Result<PathBuf, String> {
    let html = fs::read_to_string(path).map_err(|error| error.to_string())?;

    let architecture = match ::bodul_html_extractor::architecture_for(retailer) {
        Ok(architecture) => architecture,
        Err(error) => return Err(error.to_string()),
    };

    let extractor = html_extractor::Backend::Kuchiki;

    let extracted: ExtractionResult = extractor
        .extract(&html, &architecture)
        .map_err(|error| error.to_string())?;

    let json = serde_json::to_string_pretty(&extracted.json).map_err(|error| error.to_string())?;

    let output_dir = PathBuf::from("data/offers-extracted").join(slug);
    fs::create_dir_all(&output_dir).map_err(|error| format!("creating {}: {error}", output_dir.display()))?;
    let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("page");
    let out_path = output_dir.join(format!("{stem}.json"));
    fs::write(&out_path, json).map_err(|error| format!("writing {}: {error}", out_path.display()))?;
    Ok(out_path)
}

/// Blanks one page into `data/offers-valueless/{slug}/`, and writes each lifted
/// section/component into `data/offers-valueless-segments/{slug}/{name}/…`.
fn valueless_page(path: &Path, slug: &str, retailer: RetailerCode) -> Result<PathBuf, String> {
    let html = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("page.html");
    let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("page");

    let blanked = html_parser::valueless(&html, retailer).map_err(|error| error.to_string())?;

    let output_dir = PathBuf::from("data/offers-valueless").join(slug);
    fs::create_dir_all(&output_dir).map_err(|error| format!("creating {}: {error}", output_dir.display()))?;
    let out_path = output_dir.join(file_name);
    fs::write(&out_path, blanked.page).map_err(|error| format!("writing {}: {error}", out_path.display()))?;

    let sections_dir = PathBuf::from("data/offers-valueless-segments").join(slug);
    for (section, section_html) in blanked.sections {
        write_into(&sections_dir.join(&section), file_name, &section_html)?;
    }
    for (name, index, component_html) in blanked.components {
        write_into(
            &sections_dir.join(&name),
            &format!("{stem}_{index}.html"),
            &component_html,
        )?;
    }
    Ok(out_path)
}

/// Writes `contents` to `dir/file_name`, creating `dir` if needed.
fn write_into(dir: &Path, file_name: &str, contents: &str) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|error| format!("creating {}: {error}", dir.display()))?;
    let path = dir.join(file_name);
    fs::write(&path, contents).map_err(|error| format!("writing {}: {error}", path.display()))
}

/// Builds a filename from a URL's path: strip scheme/host and any query, trim the
/// leading/trailing `/`, replace remaining `/` with `-`, and ensure a `.{ext}`
/// suffix (not doubled when the path already ends in it, e.g. `…/product.xml`).
/// Returns `None` for a URL with an empty path.
fn page_filename(url: &str, ext: &str) -> Option<String> {
    let after_scheme = url.split("://").nth(1)?;
    let path = match after_scheme.find('/') {
        Some(index) => &after_scheme[index..],
        None => return None,
    };
    let path = path.split(['?', '#']).next().unwrap_or(path);
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    let name = trimmed.replace('/', "-");
    if name.ends_with(&format!(".{ext}")) {
        Some(name)
    } else {
        Some(format!("{name}.{ext}"))
    }
}
