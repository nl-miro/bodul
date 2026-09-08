//! Destructures dumped product pages into structured JSON.
//!
//! For each retailer dir found under `data/dumps/offers/{retailer}/` it processes
//! that retailer's `*.html` pages and writes one JSON file per page into
//! `data/offers-destructed/{retailer}/`.

use std::fs;
use std::path::{Path, PathBuf};

use bodul_shared::retailer::RetailerCode;
use bodul_shared::retailer::code_for_name;
use mvp::html_parser;

fn main() {
    dotenvy::dotenv().ok();
    let _guard = mvp::logging::init();
    let input_root = PathBuf::from("data/dumps/offers");
    let output_root = PathBuf::from("data/offers-destructed");

    let retailers = match retailer_dirs(&input_root) {
        Ok(retailers) => retailers,
        Err(error) => {
            tracing::error!("error reading {}/: {error}", input_root.display());
            std::process::exit(1);
        }
    };

    let mut processed = 0usize;
    let mut failed = 0usize;

    for (input_dir, retailer_code) in retailers {
        let slug = input_dir.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        let output_dir = output_root.join(slug);

        if let Err(error) = fs::create_dir_all(&output_dir) {
            tracing::error!("error creating {}: {error}", output_dir.display());
            std::process::exit(1);
        }

        let entries = match fs::read_dir(&input_dir) {
            Ok(entries) => entries,
            Err(error) => {
                tracing::error!("error reading {}/: {error}", input_dir.display());
                std::process::exit(1);
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() || path.extension().is_none_or(|ext| ext != "html") {
                continue;
            }

            match destructure_file(&path, &output_dir, retailer_code) {
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
    if processed == 0 {
        std::process::exit(1);
    }
}

/// Each `data/dumps/offers/{slug}` subdir paired with its resolved `RetailerCode`.
/// Skips (with a warning) any dir whose name isn't a known retailer slug.
fn retailer_dirs(base: &Path) -> Result<Vec<(PathBuf, RetailerCode)>, String> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(base).map_err(|error| error.to_string())?.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        match code_for_name(name) {
            Some(code) => dirs.push((path, code)),
            None => tracing::warn!("skip {}: unknown retailer slug", path.display()),
        }
    }
    Ok(dirs)
}

/// Reads one page, destructures it, and writes the JSON result.
fn destructure_file(
    path: &std::path::Path,
    output_dir: &std::path::Path,
    retailer: RetailerCode,
) -> Result<PathBuf, String> {
    let html = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let destructured = html_parser::destructure(&html, retailer);
    let json = serde_json::to_string_pretty(&destructured).map_err(|error| error.to_string())?;

    let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("page");
    let out_path = output_dir.join(format!("{stem}.json"));
    fs::write(&out_path, json).map_err(|error| format!("writing {}: {error}", out_path.display()))?;

    Ok(out_path)
}
