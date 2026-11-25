//! Téléchargement des fichiers de polices (.ttf / .otf) depuis les catalogues en ligne.

use std::collections::HashSet;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

use flate2::read::GzDecoder;
use regex::Regex;
use reqwest::blocking::Client;
use tar::Archive;
use zip::ZipArchive;

use crate::models::{FontActionResult, PdfFontInfo, SourceKind};
use crate::normalizer::{normalize_key, sanitize_filename};

fn is_desktop_font_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    (lower.ends_with(".ttf") || lower.ends_with(".otf")) && !name.contains("__MACOSX")
}

/// Extrait les fichiers `.ttf` / `.otf` d'une archive ZIP.
/// Privilégie les fichiers dans `static/` s'il y en a (standard Google Fonts)
/// pour éviter d'installer en double variable + statique.
fn extract_fonts_from_zip(
    zip_bytes: &[u8],
    target_dir: &Path,
    family_filter: Option<&str>,
) -> Result<Vec<PathBuf>, String> {
    std::fs::create_dir_all(target_dir).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(Cursor::new(zip_bytes)).map_err(|e| e.to_string())?;

    let mut entries: Vec<(usize, String)> = Vec::new();
    for i in 0..archive.len() {
        let Ok(file) = archive.by_index(i) else {
            continue;
        };
        if file.is_dir() {
            continue;
        }
        let name = file.name().to_string();
        if is_desktop_font_name(&name) {
            entries.push((i, name));
        }
    }

    if let Some(filter) = family_filter {
        let filter_key = normalize_key(filter);
        let matched: Vec<_> = entries
            .iter()
            .filter(|(_, name)| {
                let stem = Path::new(name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                normalize_key(stem).contains(&filter_key)
            })
            .cloned()
            .collect();
        if !matched.is_empty() {
            entries = matched;
        }
    }

    let static_entries: Vec<_> = entries
        .iter()
        .filter(|(_, name)| {
            let normalized = format!("/{}", name.replace('\\', "/"));
            normalized.contains("/static/")
        })
        .cloned()
        .collect();
    let chosen = if !static_entries.is_empty() {
        static_entries
    } else {
        entries
    };

    let mut saved = Vec::new();
    for (idx, name) in chosen {
        let Some(fname) = Path::new(&name).file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if fname.starts_with('.') {
            continue;
        }
        let mut file = archive.by_index(idx).map_err(|e| e.to_string())?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).map_err(|e| e.to_string())?;
        let dest = target_dir.join(sanitize_filename(fname));
        std::fs::write(&dest, &buf).map_err(|e| e.to_string())?;
        saved.push(dest);
    }

    Ok(saved)
}

/// Extrait les fichiers `.ttf` / `.otf` d'une archive `.tar.gz`.
fn extract_fonts_from_targz(
    tar_bytes: &[u8],
    target_dir: &Path,
    family_filter: Option<&str>,
) -> Result<Vec<PathBuf>, String> {
    std::fs::create_dir_all(target_dir).map_err(|e| e.to_string())?;
    let gz = GzDecoder::new(Cursor::new(tar_bytes));
    let mut archive = Archive::new(gz);

    let filter_key = family_filter.map(normalize_key);
    let mut all_fonts: Vec<(String, Vec<u8>)> = Vec::new();

    for entry_res in archive.entries().map_err(|e| e.to_string())? {
        let Ok(mut entry) = entry_res else {
            continue;
        };
        let Ok(path) = entry.path() else {
            continue;
        };
        let path_str = path.to_string_lossy().to_string();
        if !is_desktop_font_name(&path_str) {
            continue;
        }
        let Some(fname) = path.file_name().and_then(|s| s.to_str()).map(str::to_string) else {
            continue;
        };
        if fname.starts_with('.') {
            continue;
        }
        let mut buf = Vec::new();
        if entry.read_to_end(&mut buf).is_ok() {
            all_fonts.push((fname, buf));
        }
    }

    let chosen = if let Some(fk) = filter_key {
        let matched: Vec<_> = all_fonts
            .iter()
            .filter(|(fname, _)| {
                let stem = Path::new(fname)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                normalize_key(stem).contains(&fk)
            })
            .cloned()
            .collect();
        if !matched.is_empty() {
            matched
        } else {
            all_fonts
        }
    } else {
        all_fonts
    };

    let mut saved = Vec::new();
    for (fname, buf) in chosen {
        let dest = target_dir.join(sanitize_filename(&fname));
        std::fs::write(&dest, &buf).map_err(|e| e.to_string())?;
        saved.push(dest);
    }

    Ok(saved)
}

/// Fallback Google Fonts via CSS2 avec User-Agent renvoyant des URLs `.ttf` directes.
fn download_google_css2_fallback(
    client: &Client,
    family: &str,
    target_dir: &Path,
) -> Result<Vec<PathBuf>, String> {
    let encoded = urlencoding::encode(family).replace("%20", "+");
    let urls = [
        format!(
            "https://fonts.googleapis.com/css2?family={encoded}:ital,wght@0,300;0,400;0,500;0,600;0,700;1,400;1,700"
        ),
        format!("https://fonts.googleapis.com/css2?family={encoded}:wght@400;700"),
        format!("https://fonts.googleapis.com/css2?family={encoded}"),
    ];

    let mut css = String::new();
    for url in &urls {
        if let Ok(resp) = client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 6.1)")
            .send()
        {
            if resp.status().is_success() {
                if let Ok(body) = resp.text() {
                    if body.contains("@font-face") {
                        css = body;
                        break;
                    }
                }
            }
        }
    }

    if css.is_empty() {
        return Ok(Vec::new());
    }

    std::fs::create_dir_all(target_dir).map_err(|e| e.to_string())?;

    let re_block = Regex::new(r"(?s)@font-face\s*\{([^}]+)\}").unwrap();
    let re_url = Regex::new(r"(?i)url\((https?://[^)]+\.(?:ttf|otf))\)").unwrap();
    let re_style = Regex::new(r"(?i)font-style:\s*([a-z]+)").unwrap();
    let re_weight = Regex::new(r"(?i)font-weight:\s*(\d+)").unwrap();

    let mut saved = Vec::new();
    let mut seen_urls = HashSet::new();
    let safe_fam = sanitize_filename(family).replace(' ', "");

    for (idx, cap) in re_block.captures_iter(&css).enumerate() {
        let block = &cap[1];
        let Some(url_cap) = re_url.captures(block) else {
            continue;
        };
        let font_url = url_cap[1].to_string();
        if !seen_urls.insert(font_url.clone()) {
            continue;
        }

        let f_style = re_style
            .captures(block)
            .map(|c| c[1].to_ascii_lowercase())
            .unwrap_or_else(|| "normal".to_string());
        let f_weight = re_weight
            .captures(block)
            .map(|c| c[1].to_string())
            .unwrap_or_else(|| (idx + 1).to_string());

        let ext = if font_url.to_ascii_lowercase().ends_with(".otf") {
            "otf"
        } else {
            "ttf"
        };

        let bytes = client
            .get(&font_url)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.bytes())
            .map_err(|e| e.to_string())?;

        let suffix = if f_style == "italic" {
            format!("{f_weight}Italic")
        } else {
            f_weight
        };
        let dest = target_dir.join(format!("{safe_fam}-{suffix}.{ext}"));
        std::fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
        saved.push(dest);
    }

    Ok(saved)
}

/// Télécharge la police correspondant à `pdf_font.catalog_match` dans `output_dir`.
pub fn download_font(pdf_font: &PdfFontInfo, output_dir: &Path) -> FontActionResult {
    let display = pdf_font.display_name();
    let Some(match_info) = pdf_font.catalog_match.as_ref() else {
        return FontActionResult {
            font_name: display,
            success: false,
            message: "Aucune source gratuite en ligne trouvée pour cette police.".to_string(),
            files: Vec::new(),
        };
    };

    let client = match Client::builder()
        .user_agent("PDF-Fonts-Detector/1.0")
        .timeout(Duration::from_secs(40))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return FontActionResult {
                font_name: display,
                success: false,
                message: format!("Erreur HTTP : {e}"),
                files: Vec::new(),
            };
        }
    };

    let family_dir = output_dir.join(sanitize_filename(&match_info.catalog_family));

    let download_result: Result<Vec<PathBuf>, String> = match match_info.source {
        SourceKind::GoogleFonts => {
            let zip_url = format!(
                "https://fonts.google.com/download?family={}",
                urlencoding::encode(&match_info.download_id)
            );
            let mut files = Vec::new();
            if let Ok(resp) = client.get(&zip_url).send() {
                if resp.status().is_success() {
                    if let Ok(bytes) = resp.bytes() {
                        if let Ok(extracted) = extract_fonts_from_zip(&bytes, &family_dir, None) {
                            files = extracted;
                        }
                    }
                }
            }
            if files.is_empty() {
                download_google_css2_fallback(&client, &match_info.download_id, &family_dir)
            } else {
                Ok(files)
            }
        }
        SourceKind::FontSquirrel => {
            let zip_url = format!(
                "https://www.fontsquirrel.com/fonts/download/{}",
                urlencoding::encode(&match_info.download_id)
            );
            client
                .get(&zip_url)
                .send()
                .and_then(|r| r.error_for_status())
                .and_then(|r| r.bytes())
                .map_err(|e| e.to_string())
                .and_then(|bytes| extract_fonts_from_zip(&bytes, &family_dir, None))
        }
        SourceKind::OpenSourceMirror => {
            let url = &match_info.download_id;
            client
                .get(url)
                .send()
                .and_then(|r| r.error_for_status())
                .and_then(|r| r.bytes())
                .map_err(|e| e.to_string())
                .and_then(|bytes| {
                    if url.to_ascii_lowercase().ends_with(".zip") {
                        extract_fonts_from_zip(&bytes, &family_dir, Some(&match_info.catalog_family))
                    } else {
                        extract_fonts_from_targz(
                            &bytes,
                            &family_dir,
                            Some(&match_info.catalog_family),
                        )
                    }
                })
        }
        SourceKind::PdfEmbeddedComplete | SourceKind::NotFound => {
            Err("Police non téléchargeable en ligne.".to_string())
        }
    };

    match download_result {
        Ok(files) if !files.is_empty() => FontActionResult {
            font_name: display,
            success: true,
            message: format!(
                "{} fichier(s) téléchargé(s) dans {}/",
                files.len(),
                family_dir
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("fonts")
            ),
            files,
        },
        Ok(_) => FontActionResult {
            font_name: display,
            success: false,
            message: "Le téléchargement n'a renvoyé aucun fichier .ttf ou .otf exploitable."
                .to_string(),
            files: Vec::new(),
        },
        Err(err) => FontActionResult {
            font_name: display,
            success: false,
            message: format!("Échec du téléchargement : {err}"),
            files: Vec::new(),
        },
    }
}
