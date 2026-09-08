//! Produces "valueless" product pages.
//!
//! For each retailer dir found under `data/dumps/offers/{retailer}/` it processes
//! that retailer's `*.html` pages and writes one HTML file per page into
//! `data/offers-valueless/{retailer}/`. The output is still HTML, with every value
//! targeted by the retailer's architecture replaced by a placeholder.
//!
//! It also writes the valueless HTML of each top-level section (segment) and each
//! lifted collection component into
//! `data/offers-valueless-segments/{retailer}/{name}/...`. Collection items are
//! replaced in the page by `[name_index]` placeholders and written one file per
//! item (`{page}_{index}.html`).

use std::fs;
use std::path::{Path, PathBuf};

use mvp::html_parser;
use bodul_shared::retailer::RetailerCode;
use bodul_shared::retailer::code_for_name;

fn main() {
    dotenvy::dotenv().ok();
    let _guard = mvp::logging::init();
    let input_root = PathBuf::from("data/dumps/offers");
    let output_root = PathBuf::from("data/offers-valueless");
    let sections_root = PathBuf::from("data/offers-valueless-segments");

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
        let sections_dir = sections_root.join(slug);

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

            match valueless_file(&path, &output_dir, &sections_dir, retailer_code) {
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

/// Reads one page, blanks it against the retailer's architecture, and writes the
/// full-page result, each section, and each lifted collection component.
fn valueless_file(
    path: &std::path::Path,
    output_dir: &std::path::Path,
    sections_dir: &std::path::Path,
    retailer: RetailerCode,
) -> Result<PathBuf, String> {
    let html = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("page.html");
    let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("page");

    let blanked = html_parser::valueless(&html, retailer).map_err(|error| error.to_string())?;

    let out_path = output_dir.join(file_name);
    fs::write(&out_path, blanked.page).map_err(|error| format!("writing {}: {error}", out_path.display()))?;

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
fn write_into(dir: &std::path::Path, file_name: &str, contents: &str) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|error| format!("creating {}: {error}", dir.display()))?;
    let path = dir.join(file_name);
    fs::write(&path, contents).map_err(|error| format!("writing {}: {error}", path.display()))
}
