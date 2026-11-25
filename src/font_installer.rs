//! Installation des fichiers de polices (.ttf / .otf) sur Windows, Linux (et macOS) sans droits admin.

use std::path::{Path, PathBuf};

use crate::models::FontActionResult;

fn is_installable(path: &Path) -> bool {
    path.is_file()
        && path.extension().and_then(|e| e.to_str()).is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "ttf" | "otf" | "ttc" | "otc"
            )
        })
}

/// Dossier d'installation utilisateur des polices (sans droits administrateur).
pub fn get_user_fonts_dir() -> PathBuf {
    #[cfg(windows)]
    {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Microsoft")
            .join("Windows")
            .join("Fonts")
    }

    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library")
            .join("Fonts")
    }

    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local")
            .join("share")
            .join("fonts")
            .join("PDF_Fonts_Detector")
    }
}

#[cfg(windows)]
fn install_platform(font_files: &[PathBuf], target_dir: &Path) -> Result<Vec<PathBuf>, String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Graphics::Gdi::AddFontResourceW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_FONTCHANGE,
    };
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    use crate::system_fonts::read_font_metadata;

    std::fs::create_dir_all(target_dir).map_err(|e| e.to_string())?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (reg_key, _) = hkcu
        .create_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts")
        .map_err(|e| format!("Registre Windows inaccessible : {e}"))?;

    let mut installed = Vec::new();
    for src in font_files {
        let Some(fname) = src.file_name() else {
            continue;
        };
        let dest = target_dir.join(fname);
        if src != &dest {
            std::fs::copy(src, &dest).map_err(|e| e.to_string())?;
        }

        let ext = dest
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("ttf")
            .to_ascii_lowercase();
        let type_tag = if ext == "otf" || ext == "otc" {
            "(OpenType)"
        } else {
            "(TrueType)"
        };

        let metas = read_font_metadata(&dest);
        let base_label = metas
            .first()
            .filter(|m| !m.full_name.is_empty())
            .map(|m| m.full_name.clone())
            .unwrap_or_else(|| {
                dest.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Font")
                    .to_string()
            });
        let reg_label = format!("{base_label} {type_tag}");
        let dest_str = dest.to_string_lossy().to_string();
        reg_key
            .set_value(&reg_label, &dest_str)
            .map_err(|e| format!("Erreur écriture registre : {e}"))?;

        let wide: Vec<u16> = OsStr::new(&dest)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        unsafe {
            AddFontResourceW(wide.as_ptr());
        }
        installed.push(dest);
    }

    if !installed.is_empty() {
        unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_FONTCHANGE,
                0,
                0,
                SMTO_ABORTIFHUNG,
                1000,
                std::ptr::null_mut(),
            );
        }
    }

    Ok(installed)
}

#[cfg(not(windows))]
fn install_platform(font_files: &[PathBuf], target_dir: &Path) -> Result<Vec<PathBuf>, String> {
    std::fs::create_dir_all(target_dir).map_err(|e| e.to_string())?;
    let mut installed = Vec::new();
    for src in font_files {
        let Some(fname) = src.file_name() else {
            continue;
        };
        let dest = target_dir.join(fname);
        if src != &dest {
            std::fs::copy(src, &dest).map_err(|e| e.to_string())?;
        }
        installed.push(dest);
    }

    #[cfg(target_os = "linux")]
    if !installed.is_empty() {
        if let Some(home) = dirs::home_dir() {
            let _ = std::process::Command::new("fc-cache")
                .arg("-f")
                .arg(home.join(".local/share/fonts"))
                .output();
        }
    }

    Ok(installed)
}

/// Installe une liste de fichiers de polices sur l'OS courant.
pub fn install_font_files(
    font_name: &str,
    font_files: &[PathBuf],
    custom_target_dir: Option<&Path>,
) -> FontActionResult {
    let valid: Vec<PathBuf> = font_files
        .iter()
        .filter(|p| is_installable(p))
        .cloned()
        .collect();
    if valid.is_empty() {
        return FontActionResult {
            font_name: font_name.to_string(),
            success: false,
            message: "Aucun fichier de police valide (.ttf / .otf) à installer.".to_string(),
            files: Vec::new(),
        };
    }

    let default_dir = get_user_fonts_dir();
    let target_dir = custom_target_dir.unwrap_or(default_dir.as_path());

    match install_platform(&valid, target_dir) {
        Ok(installed) => FontActionResult {
            font_name: font_name.to_string(),
            success: true,
            message: format!("{} fichier(s) installé(s) sur le PC.", installed.len()),
            files: installed,
        },
        Err(err) => FontActionResult {
            font_name: font_name.to_string(),
            success: false,
            message: format!("Erreur d'installation : {err}"),
            files: Vec::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rejects_non_font_files() {
        let tmp = std::env::temp_dir().join("pdf_fonts_detector_bad.txt");
        let _ = std::fs::write(&tmp, "not a font");
        let res = install_font_files("Fake", std::slice::from_ref(&tmp), None);
        let _ = std::fs::remove_file(&tmp);
        assert!(!res.success);
    }
}
