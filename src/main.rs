//! Point d'entrée principal de PDF Fonts Detector (exécutable natif Windows & Linux).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod font_catalog;
mod font_downloader;
mod font_installer;
mod i18n;
mod models;
mod normalizer;
mod pdf_analyzer;
mod system_fonts;
mod ui;

use std::io::Cursor;
use std::path::PathBuf;

use eframe::egui;

const APP_ICON_PNG: &[u8] = include_bytes!("../assets/app_icon.png");

/// Décode l'icône PNG 256x256 embarquée dans le binaire pour la fenêtre native (Windows & Linux).
fn load_embedded_icon() -> Option<egui::IconData> {
    let decoder = png::Decoder::new(Cursor::new(APP_ICON_PNG));
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let rgba = buf[..info.buffer_size()].to_vec();
    Some(egui::IconData {
        rgba,
        width: info.width,
        height: info.height,
    })
}

fn main() -> eframe::Result<()> {
    let mut initial_pdfs: Vec<PathBuf> = Vec::new();
    let mut override_lang: Option<i18n::Language> = None;
    let mut screenshot_out: Option<PathBuf> = None;

    for arg in std::env::args_os().skip(1) {
        if let Some(s) = arg.to_str() {
            if let Some(code) = s.strip_prefix("--lang=") {
                override_lang = i18n::Language::from_locale_tag(code);
                continue;
            }
            if let Some(path) = s.strip_prefix("--screenshot=") {
                screenshot_out = Some(PathBuf::from(path));
                continue;
            }
        }
        let path = PathBuf::from(arg);
        if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
        {
            initial_pdfs.push(path);
        }
    }

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("PDF Fonts Detector")
        .with_inner_size([1200.0, 760.0])
        .with_min_inner_size([940.0, 600.0])
        .with_drag_and_drop(true);

    if let Some(icon) = load_embedded_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "PDF Fonts Detector",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(ui::PdfFontsApp::new(
                cc,
                initial_pdfs,
                override_lang,
                screenshot_out,
            )))
        }),
    )
}
