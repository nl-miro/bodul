//! Downloads every product page listed in `data/detected/*.json` and stores each
//! under `data/pages/{RetailerCode}/`, naming the file after the URL path with
//! `/` replaced by `-` (e.g. `/de/products/ms01` -> `de-products-ms01.html`).
//!
//! Already-downloaded files are skipped, and the same URL appearing in multiple
//! detection files is fetched once. An optional first argument caps the number of
//! downloads (handy for a quick test run): `fetch_products 20`.

use mvp::retailer_data_ingestion::Client;
use rand::prelude::IteratorRandom;
use serde::Deserialize;
use bodul_shared::retailer::RetailerCode;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct Detection {
    links: Links,
}

#[derive(Deserialize)]
struct Links {
    product: Vec<String>,
}

fn main() {
    dotenvy::dotenv().ok();
    let _guard = mvp::logging::init();
    let detected_dir = PathBuf::from("data").join("detected");
    let pages_dir = PathBuf::from("data").join("pages");

    // Optional first arg: cap on total downloads (for a quick test run).
    let limit: Option<usize> = std::env::args().nth(1).and_then(|arg| arg.parse().ok());

    let mut sources = match detection_files(&detected_dir) {
        Ok(sources) => sources,
        Err(error) => {
            tracing::error!("error reading {}/: {error}", detected_dir.display());
            std::process::exit(1);
        }
    };
    sources.sort();

    if sources.is_empty() {
        tracing::warn!("no detection files in {}/", detected_dir.display());
        std::process::exit(1);
    }

    let mut seen: HashSet<String> = HashSet::new();
    let mut downloaded = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    'sources: for source in &sources {
        let retailer = retailer_dir(source);

        if retailer != "Zoocityhr" {
            continue;
        }

        let detection: Detection = match fs::read_to_string(source)
            .map_err(|error| error.to_string())
            .and_then(|json| serde_json::from_str(&json).map_err(|error| error.to_string()))
        {
            Ok(detection) => detection,
            Err(error) => {
                tracing::error!("fail {}: {error}", source.display());
                continue;
            }
        };

        let dir = pages_dir.join(&retailer);
        if let Err(error) = fs::create_dir_all(&dir) {
            tracing::error!("fail creating {}: {error}", dir.display());
            continue;
        }

        println!("{retailer}: {} product links", detection.links.product.len());

        let sample = detection.links.product;

        let mut rng = rand::thread_rng();
        let sample: Vec<_> = sample.iter().choose_multiple(&mut rng, 20);

        for url in sample {
            if !seen.insert(url.clone()) {
                continue; // same URL already handled from another detection file
            }

            let Some(filename) = page_filename(url) else {
                tracing::warn!("  skip (no path): {url}");
                continue;
            };
            let path = dir.join(filename);
            if path.exists() {
                skipped += 1;
                continue;
            }

            if let Some(max) = limit
                && downloaded >= max
            {
                println!("reached download limit ({max})");
                break 'sources;
            }

            match Client::get(url) {
                Ok(body) => match fs::write(&path, body) {
                    Ok(()) => {
                        downloaded += 1;
                        if downloaded.is_multiple_of(25) {
                            println!("  {downloaded} downloaded...");
                        }
                    }
                    Err(error) => {
                        tracing::error!("  fail writing {}: {error}", path.display());
                        failed += 1;
                    }
                },
                Err(error) => {
                    tracing::error!("  fail {url}: {error}");
                    failed += 1;
                }
            }
        }
    }

    println!("done: {downloaded} downloaded, {skipped} skipped (existing), {failed} failed");
    if downloaded == 0 && failed > 0 {
        std::process::exit(1);
    }
}

/// All detection JSON files directly under `dir`.
fn detection_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).map_err(|error| error.to_string())?.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
            files.push(path);
        }
    }
    Ok(files)
}

/// The retailer folder for a detection file: the `{slug}` prefix before
/// `-sitemap`, mapped back to its `RetailerCode` name (e.g. `MinisForumEu`),
/// falling back to the raw slug if it matches no known code.
fn retailer_dir(source: &Path) -> String {
    let name = source.file_name().and_then(|name| name.to_str()).unwrap_or("");
    let slug = name.split("-sitemap").next().unwrap_or("");
    RetailerCode::ALL
        .iter()
        .find(|code| format!("{code:?}").to_lowercase() == slug)
        .map(|code| format!("{code:?}"))
        .unwrap_or_else(|| slug.to_string())
}

/// Builds a filename from a URL's path: strip scheme/host and any query, trim the
/// leading/trailing `/`, replace remaining `/` with `-`, and add `.html`.
/// Returns `None` for a URL with an empty path.
fn page_filename(url: &str) -> Option<String> {
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
    Some(format!("{}.html", trimmed.replace('/', "-")))
}
