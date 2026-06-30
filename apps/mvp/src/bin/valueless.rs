//! Produces "valueless" product pages.
//!
//! For each supported retailer it processes `data/pages/{Retailer}/*.html` and
//! writes one HTML file per page into `data/pages-valueless/{Retailer}/`. The
//! output is still HTML, with every value targeted by the retailer's architecture
//! replaced by a placeholder.
//!
//! It also writes the valueless HTML of each top-level section (segment) and each
//! lifted collection component into
//! `data/pages-valueless-segments/{Retailer}/{name}/...`. Collection items are
//! replaced in the page by `[name_index]` placeholders and written one file per
//! item (`{page}_{index}.html`).

use std::fs;
use std::path::PathBuf;

use mvp::html_parser;
use shared::retailer::RetailerCode;

/// The retailers processed, as `(directory name, code)`.
const RETAILERS: &[(&str, RetailerCode)] = &[
    ("MinisForumAu", RetailerCode::MinisForumAu),
    ("MinisForumCa", RetailerCode::MinisForumCa),
    ("MinisForumEu", RetailerCode::MinisForumEu),
    ("MinisForumUs", RetailerCode::MinisForumUs),
    ("MinisForumUk", RetailerCode::MinisForumUk),
    ("MinisForumFr", RetailerCode::MinisForumFr),
    ("MinisForumKr", RetailerCode::MinisForumKr),
    ("MinisForumJp", RetailerCode::MinisForumJp),
    ("MinisForumRu", RetailerCode::MinisForumRu),
    ("MinisForumHk", RetailerCode::MinisForumHk),
    ("MiCom", RetailerCode::MiCom),
    ("UgreenCom", RetailerCode::UgreenCom),
    ("UgreenUs", RetailerCode::UgreenUs),
    ("UgreenCa", RetailerCode::UgreenCa),
    ("UgreenEu", RetailerCode::UgreenEu),
    ("UgreenDe", RetailerCode::UgreenDe),
    ("UgreenUk", RetailerCode::UgreenUk),
    ("UgreenFr", RetailerCode::UgreenFr),
    ("UgreenNl", RetailerCode::UgreenNl),
    ("UgreenJp", RetailerCode::UgreenJp),
    ("UgreenKr", RetailerCode::UgreenKr),
    ("UgreenIn", RetailerCode::UgreenIn),
    ("UgreenNas", RetailerCode::UgreenNas),
    ("UgreenNasCa", RetailerCode::UgreenNasCa),
    ("UgreenNasEu", RetailerCode::UgreenNasEu),
    ("UgreenNasDe", RetailerCode::UgreenNasDe),
    ("UgreenNasUk", RetailerCode::UgreenNasUk),
    ("UgreenNasFr", RetailerCode::UgreenNasFr),
    ("UgreenNasEs", RetailerCode::UgreenNasEs),
    ("UgreenNasIt", RetailerCode::UgreenNasIt),
    ("UgreenNasAu", RetailerCode::UgreenNasAu),
    ("UgreenNasJp", RetailerCode::UgreenNasJp),
    ("AnkerCom", RetailerCode::AnkerCom),
    ("AnkerJapanCom", RetailerCode::AnkerJapanCom),
    ("AnkerKr", RetailerCode::AnkerKr),
    ("AnkerItalyCom", RetailerCode::AnkerItalyCom),
    ("AnkerNordicsCom", RetailerCode::AnkerNordicsCom),
    ("AnkerUk", RetailerCode::AnkerUk),
    ("AnkerCa", RetailerCode::AnkerCa),
    ("AnkerEu", RetailerCode::AnkerEu),
    ("AnkerDe", RetailerCode::AnkerDe),
    ("AnkerFr", RetailerCode::AnkerFr),
    ("AnkerPl", RetailerCode::AnkerPl),
    ("AnkerAu", RetailerCode::AnkerAu),
    ("AnkerNz", RetailerCode::AnkerNz),
    ("AnkerMy", RetailerCode::AnkerMy),
    ("AnkerVn", RetailerCode::AnkerVn),
];

fn main() {
    let mut processed = 0usize;
    let mut failed = 0usize;

    for &(retailer, retailer_code) in RETAILERS {
        let input_dir = PathBuf::from("data/pages").join(retailer);
        let output_dir = PathBuf::from("data/pages-valueless").join(retailer);
        let sections_dir = PathBuf::from("data/pages-valueless-segments").join(retailer);

        if let Err(error) = fs::create_dir_all(&output_dir) {
            eprintln!("error creating {}: {error}", output_dir.display());
            std::process::exit(1);
        }

        let entries = match fs::read_dir(&input_dir) {
            Ok(entries) => entries,
            Err(error) => {
                eprintln!("error reading {}/: {error}", input_dir.display());
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
                    eprintln!("fail {}: {error}", path.display());
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
