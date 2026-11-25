//! Analyse PDF native (lopdf + ttf-parser) : détection des polices, vérification de complétude et extraction.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use lopdf::{Dictionary, Document, Object, ObjectId};
use ttf_parser::Face;

use crate::models::{FontActionResult, InstallStatus, PdfAnalysisResult, PdfFontInfo};
use crate::normalizer::{normalize_key, normalize_style_key, parse_pdf_font_name, sanitize_filename};

/// Limite de sécurité lors de la décompression d'un flux de police (32 Mo).
const MAX_FONT_STREAM_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone)]
struct EmbeddedStreamInfo {
    stream_id: ObjectId,
    extension: String,
    is_complete: bool,
    glyph_count: u16,
}

fn resolve_dict<'a>(doc: &'a Document, obj: &'a Object) -> Option<&'a Dictionary> {
    match obj {
        Object::Dictionary(d) => Some(d),
        Object::Reference(id) => doc.get_object(*id).ok().and_then(|o| o.as_dict().ok()),
        _ => None,
    }
}

fn object_name_to_string(obj: &Object) -> Option<String> {
    match obj {
        Object::Name(bytes) => Some(String::from_utf8_lossy(bytes).to_string()),
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).to_string()),
        _ => None,
    }
}

fn read_stream_bytes(doc: &Document, id: ObjectId) -> Option<Vec<u8>> {
    let stream = doc.get_object(id).ok()?.as_stream().ok()?;
    match stream.decompressed_content() {
        Ok(bytes) if bytes.len() <= MAX_FONT_STREAM_BYTES => Some(bytes),
        Err(_) if stream.content.len() <= MAX_FONT_STREAM_BYTES => Some(stream.content.clone()),
        _ => None,
    }
}

/// Vérifie via `ttf-parser` si un flux TTF/OTF embarqué représente une police complète.
fn inspect_ttf_completeness(bytes: &[u8], has_subset_prefix: bool) -> (bool, u16) {
    let Ok(face) = Face::parse(bytes, 0) else {
        return (false, 0);
    };
    let glyph_count = face.number_of_glyphs();
    if has_subset_prefix {
        // Selon ISO 32000-1 §9.6.4, le préfixe ABCDEF+ identifie explicitement un sous-ensemble
        return (false, glyph_count);
    }

    let mut ascii_letters_present = 0usize;
    for ch in ('A'..='Z').chain('a'..='z') {
        if face.glyph_index(ch).is_some() {
            ascii_letters_present += 1;
        }
    }

    // Une police latine complète couvre quasi-intégralement A-Z et a-z (>= 50/52) et >= 85 glyphes ;
    // une police symbolique/non-latine non-subset possède généralement >= 180 glyphes.
    let is_complete = (ascii_letters_present >= 50 && glyph_count >= 85) || glyph_count >= 180;
    (is_complete, glyph_count)
}

/// Recherche le `/FontDescriptor` (directement ou via `/DescendantFonts` pour Type0)
/// et inspecte `/FontFile`, `/FontFile2`, `/FontFile3`.
fn inspect_font_descriptor(
    doc: &Document,
    font_dict: &Dictionary,
    has_subset_prefix: bool,
    cache: &mut HashMap<ObjectId, (bool, Option<EmbeddedStreamInfo>)>,
) -> (bool, Option<EmbeddedStreamInfo>) {
    // Trouver le FontDescriptor
    let descriptor_obj = if let Ok(fd) = font_dict.get(b"FontDescriptor") {
        Some(fd)
    } else if let Ok(desc_arr) = font_dict.get(b"DescendantFonts") {
        // Police Type0 / CIDFont : le FontDescriptor se trouve dans le premier DescendantFont
        let arr = match desc_arr {
            Object::Array(a) => Some(a),
            Object::Reference(id) => doc.get_object(*id).ok().and_then(|o| o.as_array().ok()),
            _ => None,
        };
        arr.and_then(|a| a.first())
            .and_then(|first| resolve_dict(doc, first))
            .and_then(|cid_dict| cid_dict.get(b"FontDescriptor").ok())
    } else {
        None
    };

    let Some(desc_obj) = descriptor_obj else {
        return (false, None);
    };
    let Some(desc_dict) = resolve_dict(doc, desc_obj) else {
        return (false, None);
    };

    // Vérifier FontFile2 (TrueType), FontFile3 (OpenType/CFF), FontFile (Type1)
    let candidates: [(&[u8], &str); 3] = [
        (b"FontFile2", "ttf"),
        (b"FontFile3", "otf"),
        (b"FontFile", "pfb"),
    ];

    for (key, default_ext) in candidates {
        if let Ok(file_ref) = desc_dict.get(key) {
            let Ok(stream_id) = file_ref.as_reference() else {
                return (true, None);
            };
            if let Some(cached) = cache.get(&stream_id) {
                return cached.clone();
            }

            let result = if default_ext == "ttf" || default_ext == "otf" {
                if let Some(bytes) = read_stream_bytes(doc, stream_id) {
                    // Déterminer l'extension réelle à partir de la signature binaire
                    let ext = if bytes.starts_with(b"OTTO") {
                        "otf".to_string()
                    } else if bytes.starts_with(&[0x00, 0x01, 0x00, 0x00])
                        || bytes.starts_with(b"true")
                    {
                        "ttf".to_string()
                    } else {
                        default_ext.to_string()
                    };

                    let (is_complete, glyph_count) =
                        inspect_ttf_completeness(&bytes, has_subset_prefix);
                    (
                        true,
                        Some(EmbeddedStreamInfo {
                            stream_id,
                            extension: ext,
                            is_complete,
                            glyph_count,
                        }),
                    )
                } else {
                    (true, None)
                }
            } else {
                (true, None)
            };

            cache.insert(stream_id, result.clone());
            return result;
        }
    }

    (false, None)
}

/// Parcourt un dictionnaire `/Resources` et récursivement ses `/XObject` (Form XObjects).
fn collect_fonts_from_resources(
    doc: &Document,
    resources: &Dictionary,
    visited_xobjects: &mut HashSet<ObjectId>,
    out_font_dicts: &mut Vec<(Option<ObjectId>, Dictionary)>,
) {
    // 1. Entrées /Font directes
    if let Ok(fonts_obj) = resources.get(b"Font") {
        if let Some(fonts_dict) = resolve_dict(doc, fonts_obj) {
            for (_font_key, font_val) in fonts_dict.iter() {
                let obj_id = font_val.as_reference().ok();
                if let Some(fdict) = resolve_dict(doc, font_val) {
                    out_font_dicts.push((obj_id, fdict.clone()));
                }
            }
        }
    }

    // 2. Form XObjects imbriqués qui déclarent leurs propres /Resources
    if let Ok(xobj_entry) = resources.get(b"XObject") {
        if let Some(xobjs_dict) = resolve_dict(doc, xobj_entry) {
            for (_name, xobj_val) in xobjs_dict.iter() {
                if let Ok(xid) = xobj_val.as_reference() {
                    if !visited_xobjects.insert(xid) {
                        continue;
                    }
                    if let Ok(Object::Stream(stream)) = doc.get_object(xid) {
                        let is_form = stream
                            .dict
                            .get(b"Subtype")
                            .ok()
                            .and_then(object_name_to_string)
                            .is_some_and(|s| s == "Form");
                        if is_form {
                            if let Ok(sub_res) = stream.dict.get(b"Resources") {
                                if let Some(sub_res_dict) = resolve_dict(doc, sub_res) {
                                    collect_fonts_from_resources(
                                        doc,
                                        sub_res_dict,
                                        visited_xobjects,
                                        out_font_dicts,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Analyse un ou plusieurs fichiers PDF et retourne la liste dédupliquée des polices.
pub fn analyze_pdfs<F>(pdf_paths: &[PathBuf], mut on_progress: F) -> PdfAnalysisResult
where
    F: FnMut(usize, usize, &str),
{
    let mut merged: HashMap<(String, String), PdfFontInfo> = HashMap::new();
    let mut errors = Vec::new();
    let mut total_pages = 0usize;

    let total_files = pdf_paths.len().max(1);

    for (idx, path) in pdf_paths.iter().enumerate() {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("document.pdf");
        on_progress(idx + 1, total_files, &format!("Analyse de {filename}…"));

        if !path.is_file() {
            errors.push(format!("Fichier introuvable : {}", path.display()));
            continue;
        }

        let doc = match Document::load(path) {
            Ok(d) => d,
            Err(err) => {
                errors.push(format!("Impossible d'ouvrir {filename} : {err}"));
                continue;
            }
        };

        if doc.is_encrypted() {
            errors.push(format!("Le PDF {filename} est chiffré/protégé par mot de passe."));
            continue;
        }

        let pages = doc.get_pages();
        total_pages += pages.len();

        let mut stream_cache: HashMap<ObjectId, (bool, Option<EmbeddedStreamInfo>)> =
            HashMap::new();

        for (page_num, page_id) in pages {
            let mut page_fonts = Vec::new();
            let mut visited_xobjects = HashSet::new();

            if let Ok((direct_res, res_ids)) = doc.get_page_resources(page_id) {
                if let Some(res_dict) = direct_res {
                    collect_fonts_from_resources(
                        &doc,
                        res_dict,
                        &mut visited_xobjects,
                        &mut page_fonts,
                    );
                }
                for rid in res_ids {
                    if let Ok(Object::Dictionary(res_dict)) = doc.get_object(rid) {
                        collect_fonts_from_resources(
                            &doc,
                            res_dict,
                            &mut visited_xobjects,
                            &mut page_fonts,
                        );
                    }
                }
            }

            for (_font_obj_id, font_dict) in page_fonts {
                let raw_name = font_dict
                    .get(b"BaseFont")
                    .ok()
                    .and_then(object_name_to_string)
                    .or_else(|| {
                        font_dict
                            .get(b"Name")
                            .ok()
                            .and_then(object_name_to_string)
                    })
                    .unwrap_or_default();

                if raw_name.is_empty() {
                    continue;
                }

                let font_type = font_dict
                    .get(b"Subtype")
                    .ok()
                    .and_then(object_name_to_string)
                    .unwrap_or_else(|| "Unknown".to_string());

                let parsed = parse_pdf_font_name(&raw_name);
                let (is_embedded, embed_info) =
                    inspect_font_descriptor(&doc, &font_dict, parsed.has_subset_prefix, &mut stream_cache);

                let (is_complete_embedded, glyph_count, extension, stream_obj) = match embed_info {
                    Some(info) => (
                        info.is_complete,
                        info.glyph_count,
                        info.extension,
                        Some(info.stream_id),
                    ),
                    None => (false, 0, String::new(), None),
                };

                let is_subset = is_embedded && (parsed.has_subset_prefix || !is_complete_embedded);

                let dedup_key = (
                    normalize_key(&parsed.family),
                    normalize_style_key(&parsed.style),
                );

                if let Some(existing) = merged.get_mut(&dedup_key) {
                    if !existing.pages.contains(&page_num) {
                        existing.pages.push(page_num);
                    }
                    if is_complete_embedded && !existing.is_complete_embedded {
                        existing.is_complete_embedded = true;
                        existing.is_subset = false;
                        existing.is_embedded = true;
                        existing.embedded_glyph_count = glyph_count;
                        existing.embedded_stream_obj = stream_obj;
                        existing.extension = extension;
                        existing.source_pdf = path.clone();
                    } else if is_embedded && !existing.is_embedded {
                        existing.is_embedded = true;
                        existing.is_subset = is_subset;
                        existing.extension = extension;
                    }
                } else {
                    merged.insert(
                        dedup_key,
                        PdfFontInfo {
                            raw_name,
                            clean_name: parsed.clean_name,
                            family: parsed.family,
                            style: parsed.style,
                            font_type,
                            extension,
                            is_embedded,
                            is_subset,
                            is_complete_embedded,
                            embedded_glyph_count: glyph_count,
                            embedded_stream_obj: stream_obj,
                            source_pdf: path.clone(),
                            pages: vec![page_num],
                            install_status: InstallStatus::Missing,
                            matched_system_font: None,
                            catalog_match: None,
                            downloaded_files: Vec::new(),
                            selected: false,
                        },
                    );
                }
            }
        }
    }

    let mut fonts: Vec<PdfFontInfo> = merged.into_values().collect();
    fonts.sort_by(|a, b| {
        a.family
            .to_lowercase()
            .cmp(&b.family.to_lowercase())
            .then_with(|| a.style.to_lowercase().cmp(&b.style.to_lowercase()))
    });

    PdfAnalysisResult {
        pdf_paths: pdf_paths.to_vec(),
        total_pages,
        fonts,
        errors,
    }
}

/// Extrait une police complètement intégrée d'un PDF vers `output_dir`.
pub fn extract_embedded_font(font_info: &PdfFontInfo, output_dir: &Path) -> FontActionResult {
    let display = font_info.display_name();
    let Some(stream_id) = font_info.embedded_stream_obj else {
        return FontActionResult {
            font_name: display,
            success: false,
            message: "Cette police n'est pas intégrée complètement dans le PDF.".to_string(),
            files: Vec::new(),
        };
    };

    if !font_info.is_complete_embedded {
        return FontActionResult {
            font_name: display,
            success: false,
            message: "Uniquement présente en sous-ensemble (non extractible complète).".to_string(),
            files: Vec::new(),
        };
    }

    let doc = match Document::load(&font_info.source_pdf) {
        Ok(d) => d,
        Err(err) => {
            return FontActionResult {
                font_name: display,
                success: false,
                message: format!("Impossible de rouvrir le PDF source : {err}"),
                files: Vec::new(),
            };
        }
    };

    let Some(bytes) = read_stream_bytes(&doc, stream_id) else {
        return FontActionResult {
            font_name: display,
            success: false,
            message: "Impossible de lire le flux binaire de la police intégrée.".to_string(),
            files: Vec::new(),
        };
    };

    if Face::parse(&bytes, 0).is_err() {
        return FontActionResult {
            font_name: display,
            success: false,
            message: "Le flux binaire extrait n'est pas une police TrueType/OpenType valide."
                .to_string(),
            files: Vec::new(),
        };
    }

    let ext = if bytes.starts_with(b"OTTO") {
        "otf"
    } else if !font_info.extension.is_empty() {
        font_info.extension.as_str()
    } else {
        "ttf"
    };

    let family_dir = output_dir.join(sanitize_filename(&font_info.family));
    if let Err(err) = std::fs::create_dir_all(&family_dir) {
        return FontActionResult {
            font_name: display,
            success: false,
            message: format!("Impossible de créer le dossier de sortie : {err}"),
            files: Vec::new(),
        };
    }

    let file_stem = sanitize_filename(&font_info.clean_name);
    let target_path = family_dir.join(format!("{file_stem}.{ext}"));
    if let Err(err) = std::fs::write(&target_path, &bytes) {
        return FontActionResult {
            font_name: display,
            success: false,
            message: format!("Erreur d'écriture du fichier : {err}"),
            files: Vec::new(),
        };
    }

    FontActionResult {
        font_name: display,
        success: true,
        message: format!(
            "Police complète extraite du PDF ({})",
            target_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("font")
        ),
        files: vec![target_path],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::dictionary;

    #[test]
    fn test_analyze_minimal_pdf_with_standard_font() {
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica-Bold",
        });
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! {
                "F1" => font_id,
            },
        });
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            "Resources" => resources_id,
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);

        let tmp_pdf = std::env::temp_dir().join("pdf_fonts_detector_test_sample.pdf");
        doc.save(&tmp_pdf).expect("save sample pdf");

        let result = analyze_pdfs(std::slice::from_ref(&tmp_pdf), |_, _, _| {});
        let _ = std::fs::remove_file(&tmp_pdf);

        assert_eq!(result.total_pages, 1);
        assert!(result.errors.is_empty());
        assert_eq!(result.fonts.len(), 1);
        assert_eq!(result.fonts[0].family, "Helvetica");
        assert_eq!(result.fonts[0].style, "Bold");
        assert!(!result.fonts[0].is_complete_embedded);
    }
}
