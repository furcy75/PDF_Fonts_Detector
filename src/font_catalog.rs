//! Recherche en ligne dans les catalogues de polices gratuites / open-source (sans clé API).

use std::collections::HashMap;
use std::time::Duration;

use reqwest::blocking::Client;
use serde::Deserialize;

use crate::models::{CatalogMatch, PdfFontInfo, SourceKind};
use crate::normalizer::{family_search_candidates, normalize_key};

const GOOGLE_FONTS_METADATA_URL: &str = "https://fonts.google.com/metadata/fonts";
const FONTSQUIRREL_LIST_URL: &str = "https://www.fontsquirrel.com/api/fontlist/all";

#[derive(Deserialize)]
struct GoogleFontsResponse {
    #[serde(rename = "familyMetadataList", default)]
    family_metadata_list: Vec<GoogleFontEntry>,
}

#[derive(Deserialize)]
struct GoogleFontEntry {
    family: String,
}

#[derive(Deserialize)]
struct FontSquirrelEntry {
    family_name: String,
    family_urlname: String,
}

/// Miroirs officiels vérifiés pour les grandes familles open-source hors Google Fonts.
fn lookup_open_source_mirror(norm_key: &str) -> Option<(&'static str, &'static str, &'static str)> {
    match norm_key {
        "liberationsans" => Some((
            "Liberation Sans",
            "https://github.com/liberationfonts/liberation-fonts/files/7261482/liberation-fonts-ttf-2.1.5.tar.gz",
            "https://github.com/liberationfonts/liberation-fonts",
        )),
        "liberationserif" => Some((
            "Liberation Serif",
            "https://github.com/liberationfonts/liberation-fonts/files/7261482/liberation-fonts-ttf-2.1.5.tar.gz",
            "https://github.com/liberationfonts/liberation-fonts",
        )),
        "liberationmono" => Some((
            "Liberation Mono",
            "https://github.com/liberationfonts/liberation-fonts/files/7261482/liberation-fonts-ttf-2.1.5.tar.gz",
            "https://github.com/liberationfonts/liberation-fonts",
        )),
        "dejavusans" => Some((
            "DejaVu Sans",
            "https://github.com/dejavu-fonts/dejavu-fonts/releases/download/version_2_37/dejavu-fonts-ttf-2.37.zip",
            "https://dejavu-fonts.github.io/",
        )),
        "dejavuserif" => Some((
            "DejaVu Serif",
            "https://github.com/dejavu-fonts/dejavu-fonts/releases/download/version_2_37/dejavu-fonts-ttf-2.37.zip",
            "https://dejavu-fonts.github.io/",
        )),
        "dejavusansmono" => Some((
            "DejaVu Sans Mono",
            "https://github.com/dejavu-fonts/dejavu-fonts/releases/download/version_2_37/dejavu-fonts-ttf-2.37.zip",
            "https://dejavu-fonts.github.io/",
        )),
        "dejavusanscondensed" => Some((
            "DejaVu Sans Condensed",
            "https://github.com/dejavu-fonts/dejavu-fonts/releases/download/version_2_37/dejavu-fonts-ttf-2.37.zip",
            "https://dejavu-fonts.github.io/",
        )),
        "cascadiacode" => Some((
            "Cascadia Code",
            "https://github.com/microsoft/cascadia-code/releases/download/v2407.24/CascadiaCode-2407.24.zip",
            "https://github.com/microsoft/cascadia-code",
        )),
        "cascadiamono" => Some((
            "Cascadia Mono",
            "https://github.com/microsoft/cascadia-code/releases/download/v2407.24/CascadiaCode-2407.24.zip",
            "https://github.com/microsoft/cascadia-code",
        )),
        _ => None,
    }
}

pub fn build_web_search_url(font_info: &PdfFontInfo) -> String {
    let query = format!("\"{}\" font download", font_info.family);
    format!(
        "https://www.google.com/search?q={}",
        urlencoding::encode(&query)
    )
}

#[derive(Debug, Clone, Default)]
pub struct FontCatalog {
    google_index: HashMap<String, String>,
    fontsquirrel_index: HashMap<String, (String, String)>, // norm -> (family_name, urlname)
    google_loaded: bool,
    fontsquirrel_loaded: bool,
    pub last_network_warning: String,
}

impl FontCatalog {
    pub fn load_if_needed(&mut self) {
        if self.google_loaded && self.fontsquirrel_loaded {
            return;
        }

        let client = match Client::builder()
            .user_agent("PDF-Fonts-Detector/1.0")
            .timeout(Duration::from_secs(15))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                self.last_network_warning = format!("Client HTTP indisponible ({e})");
                return;
            }
        };

        let mut warnings = Vec::new();

        if !self.google_loaded {
            match self.fetch_google_fonts(&client) {
                Ok(()) => self.google_loaded = true,
                Err(e) => warnings.push(format!("Google Fonts indisponible ({e})")),
            }
        }

        if !self.fontsquirrel_loaded {
            match self.fetch_fontsquirrel(&client) {
                Ok(()) => self.fontsquirrel_loaded = true,
                Err(e) => warnings.push(format!("Font Squirrel indisponible ({e})")),
            }
        }

        self.last_network_warning = warnings.join(" ; ");
    }

    fn fetch_google_fonts(&mut self, client: &Client) -> Result<(), String> {
        let text = client
            .get(GOOGLE_FONTS_METADATA_URL)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.text())
            .map_err(|e| e.to_string())?;

        // Google préfixe souvent les JSON publics avec )]}'
        let trimmed = text.trim_start();
        let json_body = if trimmed.starts_with(")]}'") {
            trimmed
                .split_once('\n')
                .map(|(_, rest)| rest)
                .unwrap_or(&trimmed[4..])
        } else {
            trimmed
        };

        let parsed: GoogleFontsResponse =
            serde_json::from_str(json_body).map_err(|e| e.to_string())?;
        for item in parsed.family_metadata_list {
            let fam = item.family.trim().to_string();
            if !fam.is_empty() {
                self.google_index.insert(normalize_key(&fam), fam);
            }
        }
        Ok(())
    }

    fn fetch_fontsquirrel(&mut self, client: &Client) -> Result<(), String> {
        let entries: Vec<FontSquirrelEntry> = client
            .get(FONTSQUIRREL_LIST_URL)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.json())
            .map_err(|e| e.to_string())?;

        for item in entries {
            let fam = item.family_name.trim().to_string();
            let slug = item.family_urlname.trim().to_string();
            if !fam.is_empty() && !slug.is_empty() {
                self.fontsquirrel_index
                    .entry(normalize_key(&fam))
                    .or_insert((fam, slug));
            }
        }
        Ok(())
    }

    pub fn lookup(&self, pdf_font: &PdfFontInfo) -> Option<CatalogMatch> {
        let candidates = family_search_candidates(&pdf_font.family);

        // 1. Google Fonts (prioritaire : fichiers TTF complets officiels)
        for cand in &candidates {
            let key = normalize_key(cand);
            if let Some(exact_family) = self.google_index.get(&key) {
                return Some(CatalogMatch {
                    source: SourceKind::GoogleFonts,
                    catalog_family: exact_family.clone(),
                    download_id: exact_family.clone(),
                    preview_url: format!(
                        "https://fonts.google.com/specimen/{}",
                        urlencoding::encode(&exact_family.replace(' ', "+"))
                    ),
                    details: "Catalogue officiel Google Fonts (TTF)".to_string(),
                });
            }
        }

        // 2. Miroirs open-source officiels (Liberation, DejaVu, Cascadia…)
        for cand in &candidates {
            let key = normalize_key(cand);
            if let Some((canonical, archive_url, project_url)) = lookup_open_source_mirror(&key) {
                return Some(CatalogMatch {
                    source: SourceKind::OpenSourceMirror,
                    catalog_family: canonical.to_string(),
                    download_id: archive_url.to_string(),
                    preview_url: project_url.to_string(),
                    details: "Dépôt open-source officiel".to_string(),
                });
            }
        }

        // 3. Font Squirrel (polices gratuites 100 % bureau TTF/OTF)
        for cand in &candidates {
            let key = normalize_key(cand);
            if let Some((fam, slug)) = self.fontsquirrel_index.get(&key) {
                return Some(CatalogMatch {
                    source: SourceKind::FontSquirrel,
                    catalog_family: fam.clone(),
                    download_id: slug.clone(),
                    preview_url: format!("https://www.fontsquirrel.com/fonts/{slug}"),
                    details: "Catalogue libre Font Squirrel (TTF/OTF)".to_string(),
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::InstallStatus;
    use std::path::PathBuf;

    fn dummy_font(family: &str) -> PdfFontInfo {
        PdfFontInfo {
            raw_name: format!("{family}-Regular"),
            clean_name: format!("{family}-Regular"),
            family: family.to_string(),
            style: "Regular".to_string(),
            font_type: "TrueType".to_string(),
            extension: String::new(),
            is_embedded: false,
            is_subset: false,
            is_complete_embedded: false,
            embedded_glyph_count: 0,
            embedded_stream_obj: None,
            source_pdf: PathBuf::new(),
            pages: vec![1],
            install_status: InstallStatus::Missing,
            matched_system_font: None,
            catalog_match: None,
            downloaded_files: Vec::new(),
            selected: false,
        }
    }

    #[test]
    fn test_catalog_priority_and_mirrors() {
        let mut cat = FontCatalog::default();
        cat.google_loaded = true;
        cat.fontsquirrel_loaded = true;
        cat.google_index
            .insert("opensans".to_string(), "Open Sans".to_string());
        cat.fontsquirrel_index
            .insert("chunkfive".to_string(), ("ChunkFive".to_string(), "chunkfive".to_string()));

        let g = cat.lookup(&dummy_font("Open Sans")).expect("google match");
        assert_eq!(g.source, SourceKind::GoogleFonts);

        let m = cat.lookup(&dummy_font("DejaVu Sans")).expect("mirror match");
        assert_eq!(m.source, SourceKind::OpenSourceMirror);

        let fs = cat.lookup(&dummy_font("ChunkFive")).expect("fontsquirrel match");
        assert_eq!(fs.source, SourceKind::FontSquirrel);

        assert!(cat.lookup(&dummy_font("ProprietaryFontXYZ999")).is_none());
    }
}
