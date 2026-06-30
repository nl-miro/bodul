//! Destructures stored product pages into structured JSON.
//!
//! For each supported retailer it processes `data/pages/{Retailer}/*.html` and
//! writes one JSON file per page into `data/pages-destructed/{Retailer}/`.

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
        let output_dir = PathBuf::from("data/pages-destructed").join(retailer);

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

            match destructure_file(&path, &output_dir, retailer_code) {
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
