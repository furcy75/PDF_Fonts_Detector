//! Modèles de données pour PDF Fonts Detector.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStatus {
    InstalledExact,
    InstalledFamily,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    GoogleFonts,
    FontSquirrel,
    OpenSourceMirror,
    PdfEmbeddedComplete,
    NotFound,
}

#[derive(Debug, Clone)]
pub struct SystemFontInfo {
    pub file_path: PathBuf,
    pub family: String,
    pub subfamily: String,
    pub full_name: String,
    pub postscript_name: String,
    pub typographic_family: String,
    pub typographic_subfamily: String,
}

#[derive(Debug, Clone)]
pub struct CatalogMatch {
    pub source: SourceKind,
    pub catalog_family: String,
    pub download_id: String,
    pub preview_url: String,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct PdfFontInfo {
    pub raw_name: String,
    pub clean_name: String,
    pub family: String,
    pub style: String,
    pub font_type: String,
    pub extension: String,
    pub is_embedded: bool,
    pub is_subset: bool,
    pub is_complete_embedded: bool,
    pub embedded_glyph_count: u16,
    /// Identifiant de l'objet PDF contenant le flux binaire (FontFile2 / FontFile3)
    pub embedded_stream_obj: Option<(u32, u16)>,
    pub source_pdf: PathBuf,
    pub pages: Vec<u32>,

    // État enrichi après comparaison système & recherche en ligne
    pub install_status: InstallStatus,
    pub matched_system_font: Option<SystemFontInfo>,
    pub catalog_match: Option<CatalogMatch>,
    pub downloaded_files: Vec<PathBuf>,
    pub selected: bool,
}

impl PdfFontInfo {
    pub fn display_name(&self) -> String {
        if !self.style.is_empty() && !self.style.eq_ignore_ascii_case("regular") {
            format!("{} {}", self.family, self.style)
        } else {
            self.family.clone()
        }
    }

    pub fn can_download_online(&self) -> bool {
        matches!(
            self.catalog_match.as_ref().map(|m| m.source),
            Some(SourceKind::GoogleFonts | SourceKind::FontSquirrel | SourceKind::OpenSourceMirror)
        )
    }

    pub fn can_extract_from_pdf(&self) -> bool {
        self.is_complete_embedded
            && self.embedded_stream_obj.is_some()
            && self.source_pdf.is_file()
    }

    pub fn is_recoverable(&self) -> bool {
        self.install_status != InstallStatus::InstalledExact
            && (self.can_download_online() || self.can_extract_from_pdf())
    }

    pub fn pages_summary(&self) -> String {
        if self.pages.is_empty() {
            return "-".to_string();
        }
        let mut sorted = self.pages.clone();
        sorted.sort_unstable();
        sorted.dedup();
        if sorted.len() <= 5 {
            return sorted
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ");
        }

        let mut ranges = Vec::new();
        let mut start = sorted[0];
        let mut prev = sorted[0];
        for &p in &sorted[1..] {
            if p == prev + 1 {
                prev = p;
            } else {
                ranges.push(if start == prev {
                    start.to_string()
                } else {
                    format!("{start}-{prev}")
                });
                start = p;
                prev = p;
            }
        }
        ranges.push(if start == prev {
            start.to_string()
        } else {
            format!("{start}-{prev}")
        });

        let summary = ranges.join(", ");
        if summary.len() > 28 {
            format!(
                "{} pages ({}–{})",
                sorted.len(),
                sorted[0],
                sorted[sorted.len() - 1]
            )
        } else {
            summary
        }
    }
}

#[derive(Debug, Clone)]
pub struct PdfAnalysisResult {
    pub pdf_paths: Vec<PathBuf>,
    pub total_pages: usize,
    pub fonts: Vec<PdfFontInfo>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FontActionResult {
    pub font_name: String,
    pub success: bool,
    pub message: String,
    pub files: Vec<PathBuf>,
}
