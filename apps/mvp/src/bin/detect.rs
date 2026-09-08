//! Classifies the links in every sitemap dump under `data/` by type and writes
//! each result, grouped, to `data/detected/` as pretty JSON. A failure on one
//! file does not stop the others.

use std::fs;
use std::path::{Path, PathBuf};

use ::retailer_sourcing::registry::classify_link;
use bodul_shared::link::LinkKind;
use bodul_shared::retailer::RetailerCode;
use mvp::lib_sitemap::io::SitemapDocument;
use serde::Serialize;

#[derive(Serialize, Default)]
struct Group {
    product: Vec<String>,
    catalog: Vec<String>,
    content: Vec<String>,
    not_interested: Vec<String>,
    unknown: Vec<String>,
}

#[derive(Serialize, Clone, Copy)]
struct Counts {
    product: usize,
    catalog: usize,
    content: usize,
    not_interested: usize,
    unknown: usize,
}

#[derive(Serialize)]
struct Detection {
    source: String,
    counts: Counts,
    links: Group,
}

fn main() {
    dotenvy::dotenv().ok();
    let _guard = mvp::logging::init();
    let sourcedir = PathBuf::from("data/processed-sitemaps");
    let data_dir = PathBuf::from("data");
    let out_dir = data_dir.join("detected");
    if let Err(error) = fs::create_dir_all(&out_dir) {
        tracing::error!("error creating {}: {error}", out_dir.display());
        std::process::exit(1);
    }

    let mut sources = match sitemap_dumps(&sourcedir) {
        Ok(sources) => sources,
        Err(error) => {
            tracing::error!("error reading {}/: {error}", sourcedir.display());
            std::process::exit(1);
        }
    };
    sources.sort();

    if sources.is_empty() {
        tracing::warn!("no sitemap dumps found in {}/", sourcedir.display());
        std::process::exit(1);
    }

    let mut succeeded = 0usize;
    let mut failed = 0usize;

    for source in &sources {
        let Some(code) = retailer_for(source) else {
            tracing::warn!("skip {}: unrecognized retailer slug", source.display());
            continue;
        };
        match detect_file(source, code, &out_dir) {
            Ok((out_path, counts)) => {
                println!(
                    "ok   {} -> {} (product {}, catalog {}, content {}, not_interested {}, unknown {})",
                    source.display(),
                    out_path.display(),
                    counts.product,
                    counts.catalog,
                    counts.content,
                    counts.not_interested,
                    counts.unknown,
                );
                succeeded += 1;
            }
            Err(error) => {
                tracing::error!("fail {}: {error}", source.display());
                failed += 1;
            }
        }
    }

    println!("done: {succeeded} succeeded, {failed} failed");
    if succeeded == 0 {
        std::process::exit(1);
    }
}

/// Removes duplicate entries within a single list, keeping first occurrences in
/// their original order.
fn dedup(list: &mut Vec<String>) {
    let mut seen = std::collections::HashSet::new();
    list.retain(|item| seen.insert(item.clone()));
}

/// Resolves the retailer for a dump from its filename slug (the part before
/// `-sitemap`, e.g. `minisforumeu-sitemap-….json`). `None` if unrecognized.
fn retailer_for(path: &Path) -> Option<RetailerCode> {
    let name = path.file_name()?.to_str()?;
    let slug = name.split("-sitemap").next()?;
    RetailerCode::from_slug(slug)
}

/// Reads one sitemap dump, classifies its links with `code`'s retailer-specific
/// rules, and writes the grouped result.
fn detect_file(source: &Path, code: RetailerCode, out_dir: &Path) -> Result<(PathBuf, Counts), String> {
    let json = fs::read_to_string(source).map_err(|error| error.to_string())?;
    let document: SitemapDocument = serde_json::from_str(&json).map_err(|error| error.to_string())?;

    let mut links = Group::default();
    for url in document.all_urls("main") {
        match classify_link(code, &url.location, &url.source, url.images.len()) {
            LinkKind::Product => links.product.push(url.location.clone()),
            LinkKind::Catalog => links.catalog.push(url.location.clone()),
            LinkKind::Content => links.content.push(url.location.clone()),
            LinkKind::NotInterested => links.not_interested.push(url.location.clone()),
            LinkKind::Unknown => links.unknown.push(url.location.clone()),
        }
    }

    // Deduplicate the items within each list independently (a URL can repeat
    // across child sitemaps); duplicates are not compared across lists.
    dedup(&mut links.product);
    dedup(&mut links.catalog);
    dedup(&mut links.content);
    dedup(&mut links.not_interested);
    dedup(&mut links.unknown);

    let counts = Counts {
        product: links.product.len(),
        catalog: links.catalog.len(),
        content: links.content.len(),
        not_interested: links.not_interested.len(),
        unknown: links.unknown.len(),
    };

    let stem = source.file_stem().and_then(|stem| stem.to_str()).unwrap_or("sitemap");
    let out_path = out_dir.join(format!("{stem}-detected.json"));

    let detection = Detection {
        source: source.display().to_string(),
        counts,
        links,
    };
    let out = serde_json::to_string_pretty(&detection).map_err(|error| error.to_string())?;
    fs::write(&out_path, &out).map_err(|error| format!("writing {}: {error}", out_path.display()))?;

    Ok((out_path, counts))
}

/// All sitemap dump files directly under `dir` (skipping any `-detected` output).
fn sitemap_dumps(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut dumps = Vec::new();
    for entry in fs::read_dir(dir).map_err(|error| error.to_string())?.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.ends_with(".json") && name.contains("-sitemap") && !name.contains("-detected") {
            dumps.push(path);
        }
    }
    Ok(dumps)
}
