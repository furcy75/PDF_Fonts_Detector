//! Inventaire ultra-rapide des polices installées sur Windows, Linux (et macOS).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use ttf_parser::{name_id, Face};

use crate::models::{InstallStatus, PdfFontInfo, SystemFontInfo};
use crate::normalizer::{family_search_candidates, normalize_key, normalize_style_key};

fn is_font_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "ttf" | "otf" | "ttc" | "otc"))
}

fn read_name(face: &Face<'_>, target_id: u16) -> String {
    // 1. Privilégier Windows Unicode anglais US
    for name in face.names() {
        if name.name_id == target_id && name.is_unicode() && name.language_id == 0x0409 {
            if let Some(s) = name.to_string() {
                let t = s.trim();
                if !t.is_empty() {
                    return t.to_string();
                }
            }
        }
    }
    // 2. N'importe quelle entrée Unicode
    for name in face.names() {
        if name.name_id == target_id && name.is_unicode() {
            if let Some(s) = name.to_string() {
                let t = s.trim();
                if !t.is_empty() {
                    return t.to_string();
                }
            }
        }
    }
    // 3. Fallback générique
    for name in face.names() {
        if name.name_id == target_id {
            if let Some(s) = name.to_string() {
                let t = s.trim();
                if !t.is_empty() {
                    return t.to_string();
                }
            }
        }
    }
    String::new()
}

/// Lit les métadonnées OpenType/TrueType d'un fichier `.ttf`, `.otf`, `.ttc` ou `.otc`.
pub fn read_font_metadata(path: &Path) -> Vec<SystemFontInfo> {
    let Ok(data) = std::fs::read(path) else {
        return Vec::new();
    };

    let face_count = ttf_parser::fonts_in_collection(&data).unwrap_or(1);
    let mut results = Vec::new();

    for index in 0..face_count {
        let Ok(face) = Face::parse(&data, index) else {
            continue;
        };

        let family = read_name(&face, name_id::FAMILY);
        let subfamily = {
            let s = read_name(&face, name_id::SUBFAMILY);
            if s.is_empty() { "Regular".to_string() } else { s }
        };
        let full_name = read_name(&face, name_id::FULL_NAME);
        let postscript_name = read_name(&face, name_id::POST_SCRIPT_NAME);
        let typographic_family = read_name(&face, name_id::TYPOGRAPHIC_FAMILY);
        let typographic_subfamily = read_name(&face, name_id::TYPOGRAPHIC_SUBFAMILY);

        if family.is_empty()
            && typographic_family.is_empty()
            && full_name.is_empty()
            && postscript_name.is_empty()
        {
            continue;
        }

        let effective_fam = if !family.is_empty() {
            family
        } else if !typographic_family.is_empty() {
            typographic_family.clone()
        } else if !full_name.is_empty() {
            full_name.clone()
        } else {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        };

        let effective_full = if !full_name.is_empty() {
            full_name
        } else {
            let f = if !typographic_family.is_empty() {
                &typographic_family
            } else {
                &effective_fam
            };
            let s = if !typographic_subfamily.is_empty() {
                &typographic_subfamily
            } else {
                &subfamily
            };
            format!("{f} {s}").trim().to_string()
        };

        results.push(SystemFontInfo {
            file_path: path.to_path_buf(),
            family: effective_fam,
            subfamily,
            full_name: effective_full,
            postscript_name,
            typographic_family,
            typographic_subfamily,
        });
    }

    results
}

fn collect_font_files_recursive(dir: &Path, out: &mut HashSet<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_font_files_recursive(&path, out);
        } else if is_font_file(&path) {
            out.insert(path);
        }
    }
}

#[cfg(windows)]
fn discover_system_font_files() -> HashSet<PathBuf> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::types::FromRegValue;
    use winreg::RegKey;

    let mut files = HashSet::new();
    let windir = std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    let sys_fonts_dir = windir.join("Fonts");
    collect_font_files_recursive(&sys_fonts_dir, &mut files);

    let user_fonts_dir = dirs::data_local_dir()
        .map(|d| d.join("Microsoft").join("Windows").join("Fonts"));
    if let Some(ref udir) = user_fonts_dir {
        collect_font_files_recursive(udir, &mut files);
    }

    // Compléter avec le registre Windows
    let reg_subkey = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts";
    for (hive, base_dir) in [
        (HKEY_LOCAL_MACHINE, sys_fonts_dir.clone()),
        (
            HKEY_CURRENT_USER,
            user_fonts_dir.unwrap_or(sys_fonts_dir),
        ),
    ] {
        if let Ok(key) = RegKey::predef(hive).open_subkey(reg_subkey) {
            for item in key.enum_values().flatten() {
                let Ok(value) = String::from_reg_value(&item.1) else {
                    continue;
                };
                if value.is_empty() {
                    continue;
                }
                let p = PathBuf::from(&value);
                let full = if p.is_absolute() { p } else { base_dir.join(p) };
                if full.is_file() && is_font_file(&full) {
                    files.insert(full);
                }
            }
        }
    }

    files
}

#[cfg(not(windows))]
fn discover_system_font_files() -> HashSet<PathBuf> {
    let mut files = HashSet::new();
    let mut dirs_to_scan = Vec::new();

    #[cfg(target_os = "macos")]
    {
        dirs_to_scan.push(PathBuf::from("/System/Library/Fonts"));
        dirs_to_scan.push(PathBuf::from("/Library/Fonts"));
        if let Some(home) = dirs::home_dir() {
            dirs_to_scan.push(home.join("Library/Fonts"));
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        dirs_to_scan.push(PathBuf::from("/usr/share/fonts"));
        dirs_to_scan.push(PathBuf::from("/usr/local/share/fonts"));
        if let Some(home) = dirs::home_dir() {
            dirs_to_scan.push(home.join(".local/share/fonts"));
            dirs_to_scan.push(home.join(".fonts"));
        }
    }

    for dir in dirs_to_scan {
        if dir.is_dir() {
            collect_font_files_recursive(&dir, &mut files);
        }
    }

    files
}

#[cfg(target_os = "linux")]
fn discover_linux_fc_list() -> Vec<SystemFontInfo> {
    use std::process::Command;

    let Ok(output) = Command::new("fc-list")
        .arg("--format=%{file}\t%{family}\t%{style}\t%{postscriptname}\t%{fullname}\n")
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut fonts = Vec::new();
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 5 {
            continue;
        }
        let file_path = parts[0].trim();
        let families_raw = parts[1].trim();
        let styles_raw = parts[2].trim();
        let ps_name = parts[3].trim();
        let full_raw = parts[4].trim();
        if file_path.is_empty() {
            continue;
        }
        let style = styles_raw
            .split(',')
            .next()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("Regular");
        let full_first = full_raw
            .split(',')
            .next()
            .map(str::trim)
            .unwrap_or("");

        for fam in families_raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            fonts.push(SystemFontInfo {
                file_path: PathBuf::from(file_path),
                family: fam.to_string(),
                subfamily: style.to_string(),
                full_name: if !full_first.is_empty() {
                    full_first.to_string()
                } else {
                    format!("{fam} {style}")
                },
                postscript_name: ps_name.to_string(),
                typographic_family: String::new(),
                typographic_subfamily: String::new(),
            });
        }
    }
    fonts
}

/// Index en mémoire des polices installées sur l'ordinateur.
#[derive(Debug, Clone, Default)]
pub struct SystemFontIndex {
    pub fonts: Vec<SystemFontInfo>,
    by_ps: HashMap<String, usize>,
    by_full: HashMap<String, usize>,
    by_family_style: HashMap<(String, String), usize>,
    by_family: HashMap<String, Vec<usize>>,
}

impl SystemFontIndex {
    pub fn from_fonts(fonts: Vec<SystemFontInfo>) -> Self {
        let mut idx = Self::default();
        for f in fonts {
            idx.add(f);
        }
        idx
    }

    pub fn add(&mut self, info: SystemFontInfo) {
        let pos = self.fonts.len();
        if !info.postscript_name.is_empty() {
            self.by_ps
                .entry(normalize_key(&info.postscript_name))
                .or_insert(pos);
        }
        if !info.full_name.is_empty() {
            self.by_full
                .entry(normalize_key(&info.full_name))
                .or_insert(pos);
        }

        let families = [&info.family, &info.typographic_family];
        let styles = [&info.subfamily, &info.typographic_subfamily];

        for fam in families {
            if fam.is_empty() {
                continue;
            }
            let fam_key = normalize_key(fam);
            if fam_key.is_empty() {
                continue;
            }
            self.by_family.entry(fam_key.clone()).or_default().push(pos);
            for sty in styles {
                if sty.is_empty() {
                    continue;
                }
                let sty_key = normalize_style_key(sty);
                self.by_family_style
                    .entry((fam_key.clone(), sty_key))
                    .or_insert(pos);
            }
        }

        self.fonts.push(info);
    }

    /// Scanne toutes les polices installées sur l'OS courant.
    pub fn scan_system() -> Self {
        let mut index = Self::default();
        let mut seen_files: HashSet<PathBuf> = HashSet::new();

        #[cfg(target_os = "linux")]
        {
            for info in discover_linux_fc_list() {
                seen_files.insert(info.file_path.clone());
                index.add(info);
            }
        }

        for path in discover_system_font_files() {
            if seen_files.contains(&path) {
                continue;
            }
            for info in read_font_metadata(&path) {
                index.add(info);
            }
        }

        index
    }

    /// Compare une police PDF aux polices installées sur le PC.
    pub fn match_pdf_font(&self, pdf_font: &PdfFontInfo) -> (InstallStatus, Option<SystemFontInfo>) {
        // 1. Nom PostScript nettoyé
        let clean_key = normalize_key(&pdf_font.clean_name);
        if let Some(&i) = self.by_ps.get(&clean_key) {
            return (InstallStatus::InstalledExact, Some(self.fonts[i].clone()));
        }

        // 2. Nom complet
        let display_key = normalize_key(&pdf_font.display_name());
        if let Some(&i) = self.by_full.get(&display_key) {
            return (InstallStatus::InstalledExact, Some(self.fonts[i].clone()));
        }
        if let Some(&i) = self.by_full.get(&clean_key) {
            return (InstallStatus::InstalledExact, Some(self.fonts[i].clone()));
        }

        // 3. Famille + Style
        let style_key = normalize_style_key(&pdf_font.style);
        let candidates = family_search_candidates(&pdf_font.family);
        for cand in &candidates {
            let fam_key = normalize_key(cand);
            if let Some(&i) = self.by_family_style.get(&(fam_key.clone(), style_key.clone())) {
                return (InstallStatus::InstalledExact, Some(self.fonts[i].clone()));
            }
            if style_key == "regular" {
                if let Some(&i) = self.by_full.get(&fam_key) {
                    return (InstallStatus::InstalledExact, Some(self.fonts[i].clone()));
                }
            }
        }

        // 4. Famille uniquement (autre graisse/style)
        for cand in &candidates {
            let fam_key = normalize_key(cand);
            if let Some(indices) = self.by_family.get(&fam_key) {
                if let Some(&first) = indices.first() {
                    return (
                        InstallStatus::InstalledFamily,
                        Some(self.fonts[first].clone()),
                    );
                }
            }
        }

        (InstallStatus::Missing, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_pdf_font(raw: &str, family: &str, style: &str) -> PdfFontInfo {
        PdfFontInfo {
            raw_name: raw.to_string(),
            clean_name: raw.to_string(),
            family: family.to_string(),
            style: style.to_string(),
            font_type: "TrueType".to_string(),
            extension: "ttf".to_string(),
            is_embedded: true,
            is_subset: true,
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
    fn test_system_font_matching() {
        let idx = SystemFontIndex::from_fonts(vec![
            SystemFontInfo {
                file_path: PathBuf::from("/fonts/Roboto-Regular.ttf"),
                family: "Roboto".to_string(),
                subfamily: "Regular".to_string(),
                full_name: "Roboto Regular".to_string(),
                postscript_name: "Roboto-Regular".to_string(),
                typographic_family: String::new(),
                typographic_subfamily: String::new(),
            },
            SystemFontInfo {
                file_path: PathBuf::from("/fonts/Roboto-Bold.ttf"),
                family: "Roboto".to_string(),
                subfamily: "Bold".to_string(),
                full_name: "Roboto Bold".to_string(),
                postscript_name: "Roboto-Bold".to_string(),
                typographic_family: String::new(),
                typographic_subfamily: String::new(),
            },
        ]);

        let (st, matched) = idx.match_pdf_font(&dummy_pdf_font("Roboto-Bold", "Roboto", "Bold"));
        assert_eq!(st, InstallStatus::InstalledExact);
        assert!(matched.is_some());

        let (st_fam, _) = idx.match_pdf_font(&dummy_pdf_font("Roboto-Light", "Roboto", "Light"));
        assert_eq!(st_fam, InstallStatus::InstalledFamily);

        let (st_miss, m_none) =
            idx.match_pdf_font(&dummy_pdf_font("Montserrat-Regular", "Montserrat", "Regular"));
        assert_eq!(st_miss, InstallStatus::Missing);
        assert!(m_none.is_none());
    }
}
