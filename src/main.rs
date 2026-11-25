//! Point d'entrée principal de PDF Fonts Detector (exécutable natif Windows & Linux).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod font_catalog;
mod font_downloader;
mod font_installer;
mod models;
mod normalizer;
mod pdf_analyzer;
mod system_fonts;
mod ui;

use std::path::PathBuf;

use eframe::egui;

/// Génère une icône RGBA 64x64 propre intégrée directement dans le binaire.
fn build_app_icon() -> egui::IconData {
    let size: u32 = 64;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let in_box = x >= 4 && x < size - 4 && y >= 4 && y < size - 4;
            if !in_box {
                continue;
            }
            let is_border = x < 7 || x >= size - 7 || y < 7 || y >= size - 7;
            let is_pdf_bar = x >= 14 && x <= 38 && y >= 14 && y <= 23;
            let is_accent_block = x >= 26 && x <= 50 && y >= 32 && y <= 50;

            let (r, g, b, a) = if is_border {
                (56, 189, 248, 255)
            } else if is_pdf_bar {
                (225, 29, 72, 255)
            } else if is_accent_block {
                (56, 189, 248, 255)
            } else {
                (15, 23, 42, 255)
            };
            rgba[idx] = r;
            rgba[idx + 1] = g;
            rgba[idx + 2] = b;
            rgba[idx + 3] = a;
        }
    }

    egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}

fn main() -> eframe::Result<()> {
    let initial_pdfs: Vec<PathBuf> = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
        })
        .collect();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("PDF Fonts Detector")
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([920.0, 600.0])
            .with_drag_and_drop(true)
            .with_icon(build_app_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "PDF Fonts Detector",
        native_options,
        Box::new(move |cc| Ok(Box::new(ui::PdfFontsApp::new(cc, initial_pdfs)))),
    )
}
