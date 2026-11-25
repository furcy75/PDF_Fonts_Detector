//! Interface graphique native (eframe / egui) pour PDF Fonts Detector.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Vec2};

use crate::font_catalog::{build_web_search_url, FontCatalog};
use crate::font_downloader::download_font;
use crate::font_installer::install_font_files;
use crate::models::{FontActionResult, InstallStatus, PdfAnalysisResult, PdfFontInfo, SourceKind};
use crate::pdf_analyzer::{analyze_pdfs, extract_embedded_font};
use crate::system_fonts::SystemFontIndex;

// Palette sombre ardoise moderne
const BG_APP: Color32 = Color32::from_rgb(15, 23, 42);
const BG_CARD: Color32 = Color32::from_rgb(30, 41, 59);
const BG_DROP_HOVER: Color32 = Color32::from_rgb(23, 37, 84);
const BG_LOG: Color32 = Color32::from_rgb(11, 18, 32);
const BORDER_SUBTLE: Color32 = Color32::from_rgb(51, 65, 85);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(241, 245, 249);
const TEXT_MUTED: Color32 = Color32::from_rgb(148, 163, 184);
const ACCENT_BLUE: Color32 = Color32::from_rgb(56, 189, 248);
const ACCENT_BLUE_BTN: Color32 = Color32::from_rgb(2, 132, 199);
const ACCENT_GREEN: Color32 = Color32::from_rgb(52, 211, 153);
const ACCENT_GREEN_BTN: Color32 = Color32::from_rgb(5, 150, 105);
const ACCENT_AMBER: Color32 = Color32::from_rgb(251, 191, 36);
const ACCENT_VIOLET: Color32 = Color32::from_rgb(167, 139, 250);
const ACCENT_VIOLET_BTN: Color32 = Color32::from_rgb(109, 40, 217);
const ACCENT_ROSE: Color32 = Color32::from_rgb(251, 113, 133);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterMode {
    All,
    Missing,
    Downloadable,
    Extractable,
    NotFound,
    Installed,
}

impl FilterMode {
    fn label(self) -> &'static str {
        match self {
            FilterMode::All => "Toutes les polices",
            FilterMode::Missing => "Manquantes uniquement",
            FilterMode::Downloadable => "Téléchargeables en ligne",
            FilterMode::Extractable => "Extractibles du PDF",
            FilterMode::NotFound => "Introuvables",
            FilterMode::Installed => "Déjà installées",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BatchAction {
    DownloadOrExtract,
    ExtractOnly,
    Install,
}

enum WorkerEvent {
    Progress { fraction: f32, message: String },
    Log(String),
    SystemScanDone(SystemFontIndex),
    AnalysisDone {
        result: PdfAnalysisResult,
        system_index: SystemFontIndex,
        catalog: FontCatalog,
    },
    BatchDone {
        results: Vec<FontActionResult>,
        updated_files: Vec<(usize, Vec<PathBuf>)>,
        new_system_index: Option<SystemFontIndex>,
    },
}

pub struct PdfFontsApp {
    system_index: Option<SystemFontIndex>,
    catalog: FontCatalog,
    analysis: Option<PdfAnalysisResult>,
    download_dir: PathBuf,

    search_query: String,
    filter_mode: FilterMode,

    busy: bool,
    progress: f32,
    status_text: String,
    show_log: bool,
    logs: Vec<String>,
    banner_message: Option<(bool, String)>,

    rx: Receiver<WorkerEvent>,
    tx: Sender<WorkerEvent>,
    pending_initial_pdfs: Option<Vec<PathBuf>>,
}

impl PdfFontsApp {
    pub fn new(cc: &eframe::CreationContext<'_>, initial_pdfs: Vec<PathBuf>) -> Self {
        apply_custom_theme(&cc.egui_ctx);

        let (tx, rx) = mpsc::channel();
        let download_dir = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("PDF_Fonts_Detector");

        let mut app = Self {
            system_index: None,
            catalog: FontCatalog::default(),
            analysis: None,
            download_dir,
            search_query: String::new(),
            filter_mode: FilterMode::All,
            busy: false,
            progress: 0.0,
            status_text: "Prêt. Glissez un fichier PDF pour commencer.".to_string(),
            show_log: false,
            logs: Vec::new(),
            banner_message: None,
            rx,
            tx,
            pending_initial_pdfs: if initial_pdfs.is_empty() {
                None
            } else {
                Some(initial_pdfs)
            },
        };

        if app.pending_initial_pdfs.is_none() {
            app.start_system_scan(&cc.egui_ctx);
        }
        app
    }

    fn log(&mut self, msg: impl Into<String>) {
        self.logs.push(msg.into());
        if self.logs.len() > 250 {
            let drain = self.logs.len() - 250;
            self.logs.drain(..drain);
        }
    }

    fn start_system_scan(&mut self, ctx: &egui::Context) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.progress = 0.25;
        self.status_text = "Inventaire des polices installées sur l'ordinateur…".to_string();

        let tx = self.tx.clone();
        let ctx_clone = ctx.clone();
        thread::spawn(move || {
            let idx = SystemFontIndex::scan_system();
            let _ = tx.send(WorkerEvent::SystemScanDone(idx));
            ctx_clone.request_repaint();
        });
    }

    fn start_pdf_analysis(&mut self, paths: Vec<PathBuf>, ctx: &egui::Context) {
        if self.busy || paths.is_empty() {
            return;
        }
        self.busy = true;
        self.progress = 0.05;
        self.banner_message = None;
        self.status_text = format!("Analyse de {} fichier(s) PDF…", paths.len());
        self.log(format!("Analyse lancée pour {} PDF.", paths.len()));

        let existing_index = self.system_index.clone();
        let mut catalog = self.catalog.clone();
        let tx = self.tx.clone();
        let ctx_clone = ctx.clone();

        thread::spawn(move || {
            // 1. Inventaire système si pas encore fait
            let sys_index = match existing_index {
                Some(idx) => idx,
                None => {
                    let _ = tx.send(WorkerEvent::Progress {
                        fraction: 0.1,
                        message: "Inventaire des polices installées sur le PC…".to_string(),
                    });
                    ctx_clone.request_repaint();
                    let idx = SystemFontIndex::scan_system();
                    let _ = tx.send(WorkerEvent::Log(format!(
                        "{} polices système indexées.",
                        idx.fonts.len()
                    )));
                    idx
                }
            };

            // 2. Analyse PDF
            let mut result = analyze_pdfs(&paths, |cur, total, msg| {
                let frac = 0.15 + (cur as f32 / total.max(1) as f32) * 0.40;
                let _ = tx.send(WorkerEvent::Progress {
                    fraction: frac,
                    message: msg.to_string(),
                });
                ctx_clone.request_repaint();
            });

            for err in &result.errors {
                let _ = tx.send(WorkerEvent::Log(format!("Avertissement : {err}")));
            }

            // 3. Comparaison avec les polices du PC
            let _ = tx.send(WorkerEvent::Progress {
                fraction: 0.60,
                message: "Comparaison avec les polices installées sur le PC…".to_string(),
            });
            ctx_clone.request_repaint();

            let mut missing_indices = Vec::new();
            for (i, font) in result.fonts.iter_mut().enumerate() {
                let (st, matched) = sys_index.match_pdf_font(font);
                font.install_status = st;
                font.matched_system_font = matched;
                if st != InstallStatus::InstalledExact {
                    missing_indices.push(i);
                }
            }

            // 4. Recherche en ligne pour les polices manquantes
            if !missing_indices.is_empty() {
                let _ = tx.send(WorkerEvent::Progress {
                    fraction: 0.68,
                    message: "Chargement des catalogues gratuits (Google Fonts, Font Squirrel)…"
                        .to_string(),
                });
                ctx_clone.request_repaint();

                catalog.load_if_needed();
                if !catalog.last_network_warning.is_empty() {
                    let _ = tx.send(WorkerEvent::Log(format!(
                        "Réseau : {}",
                        catalog.last_network_warning
                    )));
                }

                let total_missing = missing_indices.len().max(1);
                for (step, &font_idx) in missing_indices.iter().enumerate() {
                    let frac = 0.72 + ((step + 1) as f32 / total_missing as f32) * 0.26;
                    let family_name = result.fonts[font_idx].family.clone();
                    let _ = tx.send(WorkerEvent::Progress {
                        fraction: frac,
                        message: format!("Recherche en ligne : {family_name}…"),
                    });
                    ctx_clone.request_repaint();

                    let matched_cat = catalog.lookup(&result.fonts[font_idx]);
                    result.fonts[font_idx].catalog_match = matched_cat;
                }
            }

            // Pré-cocher automatiquement les polices manquantes récupérables
            for font in &mut result.fonts {
                font.selected = font.is_recoverable();
            }

            let _ = tx.send(WorkerEvent::AnalysisDone {
                result,
                system_index: sys_index,
                catalog,
            });
            ctx_clone.request_repaint();
        });
    }

    fn start_batch_action(
        &mut self,
        action: BatchAction,
        target_indices: Vec<usize>,
        ctx: &egui::Context,
    ) {
        if self.busy || target_indices.is_empty() {
            return;
        }
        let Some(analysis) = self.analysis.as_ref() else {
            return;
        };

        let items: Vec<(usize, PdfFontInfo)> = target_indices
            .into_iter()
            .filter_map(|idx| analysis.fonts.get(idx).cloned().map(|f| (idx, f)))
            .collect();
        if items.is_empty() {
            return;
        }

        self.busy = true;
        self.progress = 0.05;
        self.banner_message = None;
        self.status_text = match action {
            BatchAction::DownloadOrExtract => "Récupération des polices en cours…".to_string(),
            BatchAction::ExtractOnly => "Extraction depuis le PDF en cours…".to_string(),
            BatchAction::Install => "Installation des polices sur le PC…".to_string(),
        };

        let output_dir = self.download_dir.clone();
        let tx = self.tx.clone();
        let ctx_clone = ctx.clone();

        thread::spawn(move || {
            let _ = std::fs::create_dir_all(&output_dir);
            let total = items.len().max(1);
            let mut results = Vec::new();
            let mut updated_files: Vec<(usize, Vec<PathBuf>)> = Vec::new();
            let mut family_cache: HashMap<String, Vec<PathBuf>> = HashMap::new();
            let mut any_installed = false;

            for (step, (idx, font)) in items.into_iter().enumerate() {
                let frac = (step as f32 + 0.5) / total as f32;
                let display = font.display_name();
                let _ = tx.send(WorkerEvent::Progress {
                    fraction: frac,
                    message: format!("Traitement : {display}…"),
                });
                ctx_clone.request_repaint();

                let acquire = |f: &PdfFontInfo,
                               cache: &mut HashMap<String, Vec<PathBuf>>,
                               allow_extract: bool|
                 -> FontActionResult {
                    if f.can_download_online() {
                        if let Some(m) = f.catalog_match.as_ref() {
                            let key = format!("{:?}:{}", m.source, m.download_id.to_lowercase());
                            if let Some(cached) = cache.get(&key) {
                                return FontActionResult {
                                    font_name: f.display_name(),
                                    success: true,
                                    message: format!(
                                        "Déjà téléchargée dans ce lot ({} fichier(s)).",
                                        cached.len()
                                    ),
                                    files: cached.clone(),
                                };
                            }
                            let res = download_font(f, &output_dir);
                            if res.success {
                                cache.insert(key, res.files.clone());
                            }
                            return res;
                        }
                    }
                    if allow_extract && f.can_extract_from_pdf() {
                        return extract_embedded_font(f, &output_dir);
                    }
                    FontActionResult {
                        font_name: f.display_name(),
                        success: false,
                        message: "Introuvable en source libre et non extractible complètement du PDF."
                            .to_string(),
                        files: Vec::new(),
                    }
                };

                match action {
                    BatchAction::DownloadOrExtract => {
                        let res = acquire(&font, &mut family_cache, true);
                        if res.success {
                            updated_files.push((idx, res.files.clone()));
                        }
                        let tag = if res.success { "OK" } else { "ERREUR" };
                        let _ = tx.send(WorkerEvent::Log(format!(
                            "[{tag}] {} : {}",
                            res.font_name, res.message
                        )));
                        results.push(res);
                    }
                    BatchAction::ExtractOnly => {
                        let res = extract_embedded_font(&font, &output_dir);
                        if res.success {
                            updated_files.push((idx, res.files.clone()));
                        }
                        let tag = if res.success { "OK" } else { "ERREUR" };
                        let _ = tx.send(WorkerEvent::Log(format!(
                            "[{tag}] {} : {}",
                            res.font_name, res.message
                        )));
                        results.push(res);
                    }
                    BatchAction::Install => {
                        let mut existing_files: Vec<PathBuf> = font
                            .downloaded_files
                            .iter()
                            .filter(|p| p.is_file())
                            .cloned()
                            .collect();

                        if existing_files.is_empty() {
                            let acq = acquire(&font, &mut family_cache, true);
                            if !acq.success {
                                let _ = tx.send(WorkerEvent::Log(format!(
                                    "[ERREUR] {} : {}",
                                    acq.font_name, acq.message
                                )));
                                results.push(acq);
                                continue;
                            }
                            updated_files.push((idx, acq.files.clone()));
                            existing_files = acq.files;
                        }

                        let inst = install_font_files(&display, &existing_files, None);
                        if inst.success {
                            any_installed = true;
                        }
                        let tag = if inst.success { "OK" } else { "ERREUR" };
                        let _ = tx.send(WorkerEvent::Log(format!(
                            "[{tag}] {} : {}",
                            inst.font_name, inst.message
                        )));
                        results.push(inst);
                    }
                }
            }

            let new_system_index = if any_installed {
                let _ = tx.send(WorkerEvent::Progress {
                    fraction: 0.95,
                    message: "Actualisation des polices du PC…".to_string(),
                });
                ctx_clone.request_repaint();
                Some(SystemFontIndex::scan_system())
            } else {
                None
            };

            let _ = tx.send(WorkerEvent::BatchDone {
                results,
                updated_files,
                new_system_index,
            });
            ctx_clone.request_repaint();
        });
    }

    fn process_worker_events(&mut self) {
        while let Ok(ev) = self.rx.try_recv() {
            match ev {
                WorkerEvent::Progress { fraction, message } => {
                    self.progress = fraction.clamp(0.0, 1.0);
                    self.status_text = message;
                }
                WorkerEvent::Log(msg) => {
                    self.log(msg);
                }
                WorkerEvent::SystemScanDone(idx) => {
                    let count = idx.fonts.len();
                    self.system_index = Some(idx);
                    self.busy = false;
                    self.progress = 1.0;
                    self.status_text =
                        format!("Prêt — {count} polices détectées sur cet ordinateur.");
                    self.log(format!("Polices système actualisées ({count})."));

                    if let (Some(analysis), Some(sys)) =
                        (self.analysis.as_mut(), self.system_index.as_ref())
                    {
                        for font in &mut analysis.fonts {
                            let (st, matched) = sys.match_pdf_font(font);
                            font.install_status = st;
                            font.matched_system_font = matched;
                        }
                    }
                }
                WorkerEvent::AnalysisDone {
                    result,
                    system_index,
                    catalog,
                } => {
                    let font_count = result.fonts.len();
                    let page_count = result.total_pages;
                    let warn_text = if !result.errors.is_empty() {
                        Some((false, result.errors.join(" • ")))
                    } else {
                        None
                    };
                    self.system_index = Some(system_index);
                    self.catalog = catalog;
                    self.analysis = Some(result);
                    self.busy = false;
                    self.progress = 1.0;
                    self.status_text = format!(
                        "Analyse terminée : {font_count} police(s) détectée(s) sur {page_count} page(s)."
                    );
                    self.banner_message = warn_text;
                    self.log(self.status_text.clone());
                }
                WorkerEvent::BatchDone {
                    results,
                    updated_files,
                    new_system_index,
                } => {
                    if let Some(analysis) = self.analysis.as_mut() {
                        for (idx, files) in updated_files {
                            if let Some(font) = analysis.fonts.get_mut(idx) {
                                for f in files {
                                    if !font.downloaded_files.contains(&f) {
                                        font.downloaded_files.push(f);
                                    }
                                }
                            }
                        }
                    }

                    if let Some(new_idx) = new_system_index {
                        self.system_index = Some(new_idx);
                        if let (Some(analysis), Some(sys)) =
                            (self.analysis.as_mut(), self.system_index.as_ref())
                        {
                            for font in &mut analysis.fonts {
                                let (st, matched) = sys.match_pdf_font(font);
                                font.install_status = st;
                                font.matched_system_font = matched;
                                if st == InstallStatus::InstalledExact {
                                    font.selected = false;
                                }
                            }
                        }
                    }

                    let ok_cnt = results.iter().filter(|r| r.success).count();
                    let err_cnt = results.len() - ok_cnt;
                    let summary = if err_cnt == 0 {
                        format!("Opération réussie : {ok_cnt} police(s) traitée(s).")
                    } else {
                        format!("Terminé : {ok_cnt} succès, {err_cnt} échec(s) (voir le Journal).")
                    };
                    self.busy = false;
                    self.progress = 1.0;
                    self.status_text = summary.clone();
                    self.banner_message = Some((err_cnt == 0, summary));
                }
            }
        }
    }

    fn matches_filter(&self, font: &PdfFontInfo) -> bool {
        let q = self.search_query.trim().to_lowercase();
        if !q.is_empty() {
            let haystack = format!(
                "{} {} {} {}",
                font.display_name(),
                font.raw_name,
                font.family,
                font.style
            )
            .to_lowercase();
            if !haystack.contains(&q) {
                return false;
            }
        }

        match self.filter_mode {
            FilterMode::All => true,
            FilterMode::Missing => font.install_status != InstallStatus::InstalledExact,
            FilterMode::Downloadable => {
                font.install_status != InstallStatus::InstalledExact && font.can_download_online()
            }
            FilterMode::Extractable => {
                font.install_status != InstallStatus::InstalledExact
                    && !font.can_download_online()
                    && font.can_extract_from_pdf()
            }
            FilterMode::NotFound => {
                font.install_status != InstallStatus::InstalledExact
                    && !font.can_download_online()
                    && !font.can_extract_from_pdf()
            }
            FilterMode::Installed => font.install_status == InstallStatus::InstalledExact,
        }
    }

    fn open_pdf_picker(&mut self, ctx: &egui::Context) {
        if self.busy {
            return;
        }
        if let Some(files) = rfd::FileDialog::new()
            .set_title("Sélectionner un ou plusieurs fichiers PDF")
            .add_filter("Fichiers PDF", &["pdf"])
            .pick_files()
        {
            if !files.is_empty() {
                self.start_pdf_analysis(files, ctx);
            }
        }
    }
}

impl eframe::App for PdfFontsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_worker_events();

        if let Some(initial) = self.pending_initial_pdfs.take() {
            self.start_pdf_analysis(initial, ctx);
        }

        // Gestion du glisser-déposer de fichiers PDF sur la fenêtre
        let dropped_pdfs: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .filter(|p| {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
                })
                .collect()
        });
        if !dropped_pdfs.is_empty() {
            self.start_pdf_analysis(dropped_pdfs, ctx);
        }
        let is_hovering_files = ctx.input(|i| !i.raw.hovered_files.is_empty());

        // Barre inférieure (statut, progression, journal)
        egui::TopBottomPanel::bottom("bottom_status")
            .frame(
                Frame::new()
                    .fill(BG_APP)
                    .inner_margin(Margin::symmetric(16, 10))
                    .stroke(Stroke::new(1.0, BORDER_SUBTLE)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&self.status_text).color(TEXT_PRIMARY).size(12.5));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let log_btn_text = if self.show_log {
                            "Masquer journal"
                        } else {
                            "Journal"
                        };
                        if ui.button(log_btn_text).clicked() {
                            self.show_log = !self.show_log;
                        }
                        if self.busy {
                            ui.add(
                                egui::ProgressBar::new(self.progress)
                                    .desired_width(210.0)
                                    .animate(true),
                            );
                        }
                    });
                });

                if self.show_log {
                    ui.add_space(6.0);
                    Frame::new()
                        .fill(BG_LOG)
                        .stroke(Stroke::new(1.0, BORDER_SUBTLE))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(Margin::same(8))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(110.0)
                                .stick_to_bottom(true)
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    if self.logs.is_empty() {
                                        ui.label(
                                            RichText::new("Aucune activité enregistrée.")
                                                .color(TEXT_MUTED)
                                                .monospace()
                                                .size(11.5),
                                        );
                                    } else {
                                        for line in &self.logs {
                                            ui.label(
                                                RichText::new(line)
                                                    .color(Color32::from_rgb(203, 213, 225))
                                                    .monospace()
                                                    .size(11.5),
                                            );
                                        }
                                    }
                                });
                        });
                }
            });

        // Panneau central principal
        egui::CentralPanel::default()
            .frame(Frame::new().fill(BG_APP).inner_margin(Margin::same(16)))
            .show(ctx, |ui| {
                // 1. En-tête
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("PDF Fonts Detector")
                                .size(21.0)
                                .strong()
                                .color(TEXT_PRIMARY),
                        );
                        ui.label(
                            RichText::new(
                                "Détectez les polices d'un PDF, vérifiez leur présence sur Windows/Linux, téléchargez-les ou extrayez-les.",
                            )
                            .size(12.0)
                            .color(TEXT_MUTED),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let open_btn = egui::Button::new(
                            RichText::new("Ouvrir un PDF…").strong().color(Color32::WHITE),
                        )
                        .fill(ACCENT_BLUE_BTN)
                        .corner_radius(CornerRadius::same(7))
                        .min_size(Vec2::new(126.0, 32.0));
                        if ui.add_enabled(!self.busy, open_btn).clicked() {
                            self.open_pdf_picker(ctx);
                        }

                        let refresh_btn = egui::Button::new("Rafraîchir les polices du PC")
                            .corner_radius(CornerRadius::same(7))
                            .min_size(Vec2::new(0.0, 32.0));
                        if ui.add_enabled(!self.busy, refresh_btn).clicked() {
                            self.start_system_scan(ctx);
                        }
                    });
                });

                ui.add_space(10.0);

                // 2. Zone de glisser-déposer
                let drop_fill = if is_hovering_files {
                    BG_DROP_HOVER
                } else {
                    BG_CARD
                };
                let drop_stroke = if is_hovering_files {
                    Stroke::new(2.0, ACCENT_BLUE)
                } else {
                    Stroke::new(1.5, BORDER_SUBTLE)
                };
                let drop_resp = Frame::new()
                    .fill(drop_fill)
                    .stroke(drop_stroke)
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::symmetric(18, 14))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.vertical_centered(|ui| {
                            let drop_title = match &self.analysis {
                                Some(res) => {
                                    let preview: Vec<String> = res
                                        .pdf_paths
                                        .iter()
                                        .take(3)
                                        .filter_map(|p| {
                                            p.file_name().and_then(|s| s.to_str()).map(str::to_string)
                                        })
                                        .collect();
                                    let extra = if res.pdf_paths.len() > 3 {
                                        format!(" (+{} autres)", res.pdf_paths.len() - 3)
                                    } else {
                                        String::new()
                                    };
                                    format!("PDF chargé(s) : {}{}", preview.join(", "), extra)
                                }
                                None => {
                                    "Glissez-déposez vos fichiers PDF ici (ou cliquez pour parcourir)"
                                        .to_string()
                                }
                            };
                            ui.label(RichText::new(drop_title).size(14.5).strong().color(TEXT_PRIMARY));
                            ui.add_space(2.0);
                            let hint = match &self.analysis {
                                Some(res) => format!(
                                    "{} PDF • {} page(s) • Déposez un autre PDF ou cliquez ici pour changer de fichier",
                                    res.pdf_paths.len(),
                                    res.total_pages
                                ),
                                None => {
                                    "100 % autonome • Google Fonts, Font Squirrel, miroirs libres & extraction de police complète du PDF"
                                        .to_string()
                                }
                            };
                            ui.label(RichText::new(hint).size(11.5).color(TEXT_MUTED));
                        });
                    })
                    .response
                    .interact(egui::Sense::click());
                if drop_resp.clicked() && !self.busy {
                    self.open_pdf_picker(ctx);
                }

                ui.add_space(10.0);

                // 3. Cartes de statistiques
                let (total, installed, downloadable, extractable, not_found) =
                    if let Some(res) = &self.analysis {
                        let tot = res.fonts.len();
                        let inst = res
                            .fonts
                            .iter()
                            .filter(|f| f.install_status == InstallStatus::InstalledExact)
                            .count();
                        let dl = res
                            .fonts
                            .iter()
                            .filter(|f| {
                                f.install_status != InstallStatus::InstalledExact
                                    && f.can_download_online()
                            })
                            .count();
                        let ext = res
                            .fonts
                            .iter()
                            .filter(|f| {
                                f.install_status != InstallStatus::InstalledExact
                                    && !f.can_download_online()
                                    && f.can_extract_from_pdf()
                            })
                            .count();
                        let nf = res
                            .fonts
                            .iter()
                            .filter(|f| {
                                f.install_status != InstallStatus::InstalledExact
                                    && !f.can_download_online()
                                    && !f.can_extract_from_pdf()
                            })
                            .count();
                        (tot, inst, dl, ext, nf)
                    } else {
                        (0, 0, 0, 0, 0)
                    };

                ui.columns(5, |cols| {
                    stat_card(&mut cols[0], "Polices détectées", total, TEXT_PRIMARY);
                    stat_card(&mut cols[1], "Installées sur le PC", installed, ACCENT_GREEN);
                    stat_card(&mut cols[2], "Téléchargeables", downloadable, ACCENT_BLUE);
                    stat_card(&mut cols[3], "Extractibles du PDF", extractable, ACCENT_VIOLET);
                    stat_card(&mut cols[4], "Introuvables", not_found, ACCENT_ROSE);
                });

                // Bannière d'information / résultat
                if let Some((ok, msg)) = self.banner_message.clone() {
                    ui.add_space(8.0);
                    let border_col = if ok { ACCENT_GREEN } else { ACCENT_AMBER };
                    Frame::new()
                        .fill(BG_CARD)
                        .stroke(Stroke::new(1.0, border_col))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(Margin::symmetric(12, 7))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(msg).color(border_col).size(12.5));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.small_button("Fermer").clicked() {
                                            self.banner_message = None;
                                        }
                                    },
                                );
                            });
                        });
                }

                ui.add_space(10.0);

                // 4. Barre de filtres et sélection
                Frame::new()
                    .fill(BG_CARD)
                    .stroke(Stroke::new(1.0, BORDER_SUBTLE))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.search_query)
                                    .hint_text("Rechercher une police par nom…")
                                    .desired_width(260.0),
                            );

                            egui::ComboBox::from_id_salt("filter_mode")
                                .selected_text(self.filter_mode.label())
                                .width(200.0)
                                .show_ui(ui, |ui| {
                                    for mode in [
                                        FilterMode::All,
                                        FilterMode::Missing,
                                        FilterMode::Downloadable,
                                        FilterMode::Extractable,
                                        FilterMode::NotFound,
                                        FilterMode::Installed,
                                    ] {
                                        ui.selectable_value(
                                            &mut self.filter_mode,
                                            mode,
                                            mode.label(),
                                        );
                                    }
                                });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("Tout décocher").clicked() {
                                        if let Some(res) = self.analysis.as_mut() {
                                            for f in &mut res.fonts {
                                                f.selected = false;
                                            }
                                        }
                                    }
                                    if ui.button("Cocher les récupérables").clicked() {
                                        if let Some(res) = self.analysis.as_mut() {
                                            for f in &mut res.fonts {
                                                f.selected = f.is_recoverable();
                                            }
                                        }
                                    }
                                },
                            );
                        });
                    });

                ui.add_space(8.0);

                // 5. Barre dossier de téléchargement + actions groupées (ancrée en bas du panneau central)
                let mut pending_single_action: Option<(BatchAction, usize)> = None;
                let mut pending_web_url: Option<String> = None;

                let table_height = (ui.available_height() - 62.0).max(160.0);

                // Tableau des polices
                Frame::new()
                    .fill(BG_CARD)
                    .stroke(Stroke::new(1.0, BORDER_SUBTLE))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.set_min_height(table_height);

                        let visible_indices: Vec<usize> = self
                            .analysis
                            .as_ref()
                            .map(|res| {
                                res.fonts
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, f)| self.matches_filter(f))
                                    .map(|(i, _)| i)
                                    .collect()
                            })
                            .unwrap_or_default();

                        if self.analysis.is_none() {
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    RichText::new(
                                        "Aucun PDF analysé pour l'instant.\nGlissez un document PDF dans la fenêtre ou cliquez sur « Ouvrir un PDF… ».",
                                    )
                                    .color(TEXT_MUTED)
                                    .size(13.5),
                                );
                            });
                        } else if visible_indices.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    RichText::new(
                                        "Aucune police ne correspond au filtre actuel.",
                                    )
                                    .color(TEXT_MUTED),
                                );
                            });
                        } else {
                            egui::ScrollArea::vertical()
                                .max_height(table_height - 12.0)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    egui::Grid::new("fonts_table")
                                        .striped(true)
                                        .num_columns(7)
                                        .spacing([12.0, 8.0])
                                        .min_col_width(40.0)
                                        .show(ui, |ui| {
                                            // En-têtes
                                            ui.label(RichText::new("Sél.").strong().color(TEXT_MUTED));
                                            ui.label(RichText::new("Police").strong().color(TEXT_MUTED));
                                            ui.label(
                                                RichText::new("Type & intégration")
                                                    .strong()
                                                    .color(TEXT_MUTED),
                                            );
                                            ui.label(RichText::new("Pages").strong().color(TEXT_MUTED));
                                            ui.label(
                                                RichText::new("Présence PC")
                                                    .strong()
                                                    .color(TEXT_MUTED),
                                            );
                                            ui.label(
                                                RichText::new("Source disponible")
                                                    .strong()
                                                    .color(TEXT_MUTED),
                                            );
                                            ui.label(RichText::new("Action").strong().color(TEXT_MUTED));
                                            ui.end_row();

                                            let busy = self.busy;
                                            if let Some(res) = self.analysis.as_mut() {
                                                for idx in visible_indices {
                                                    let font = &mut res.fonts[idx];

                                                    // 0. Case à cocher
                                                    ui.checkbox(&mut font.selected, "");

                                                    // 1. Nom de la police + info-bulle détaillée
                                                    let display = font.display_name();
                                                    let mut tip = format!(
                                                        "Nom brut PDF : {}\nFamille : {}\nStyle : {}",
                                                        font.raw_name, font.family, font.style
                                                    );
                                                    if let Some(sys) = &font.matched_system_font {
                                                        tip.push_str(&format!(
                                                            "\nFichier PC : {}",
                                                            sys.file_path.display()
                                                        ));
                                                    }
                                                    if !font.downloaded_files.is_empty() {
                                                        tip.push_str(&format!(
                                                            "\nRécupérée : {} fichier(s)",
                                                            font.downloaded_files.len()
                                                        ));
                                                    }
                                                    ui.label(
                                                        RichText::new(display)
                                                            .strong()
                                                            .color(TEXT_PRIMARY),
                                                    )
                                                    .on_hover_text(tip);

                                                    // 2. Type & intégration
                                                    let embed_label = if font.is_complete_embedded {
                                                        format!(
                                                            "{} • Complète ({} glyphes)",
                                                            font.font_type, font.embedded_glyph_count
                                                        )
                                                    } else if font.is_embedded {
                                                        format!("{} • Sous-ensemble", font.font_type)
                                                    } else {
                                                        format!("{} • Non intégrée", font.font_type)
                                                    };
                                                    ui.label(
                                                        RichText::new(embed_label).color(TEXT_MUTED),
                                                    );

                                                    // 3. Pages
                                                    ui.label(font.pages_summary());

                                                    // 4. Statut sur le PC
                                                    let (pc_text, pc_color) = match font.install_status {
                                                        InstallStatus::InstalledExact => {
                                                            ("Installée", ACCENT_GREEN)
                                                        }
                                                        InstallStatus::InstalledFamily => {
                                                            ("Autre style installé", ACCENT_AMBER)
                                                        }
                                                        InstallStatus::Missing => {
                                                            ("Manquante", ACCENT_ROSE)
                                                        }
                                                    };
                                                    ui.label(RichText::new(pc_text).strong().color(pc_color));

                                                    // 5. Source disponible
                                                    if font.install_status
                                                        == InstallStatus::InstalledExact
                                                    {
                                                        ui.label(
                                                            RichText::new("Déjà sur le PC")
                                                                .color(TEXT_MUTED),
                                                        );
                                                    } else if let Some(cat) = &font.catalog_match {
                                                        let src_txt = match cat.source {
                                                            SourceKind::GoogleFonts => "Google Fonts",
                                                            SourceKind::FontSquirrel => {
                                                                "Font Squirrel (libre)"
                                                            }
                                                            SourceKind::OpenSourceMirror => {
                                                                "Dépôt open-source"
                                                            }
                                                            _ => "Catalogue libre",
                                                        };
                                                        ui.label(
                                                            RichText::new(src_txt)
                                                                .strong()
                                                                .color(ACCENT_BLUE),
                                                        )
                                                        .on_hover_text(format!(
                                                            "Famille : {}\n{}",
                                                            cat.catalog_family, cat.details
                                                        ));
                                                    } else if font.can_extract_from_pdf() {
                                                        ui.label(
                                                            RichText::new("Extractible du PDF")
                                                                .strong()
                                                                .color(ACCENT_VIOLET),
                                                        )
                                                        .on_hover_text(
                                                            "Introuvable en source gratuite, mais la police complète est intégrée dans ce PDF.",
                                                        );
                                                    } else {
                                                        let reason = if font.is_embedded {
                                                            "Police commerciale/personnalisée absente des catalogues gratuits (uniquement en sous-ensemble dans le PDF)."
                                                        } else {
                                                            "Police commerciale/personnalisée absente des catalogues gratuits et non intégrée au PDF."
                                                        };
                                                        ui.label(
                                                            RichText::new("Introuvable")
                                                                .strong()
                                                                .color(ACCENT_ROSE),
                                                        )
                                                        .on_hover_text(reason);
                                                    }

                                                    // 6. Action rapide par ligne
                                                    if font.install_status
                                                        == InstallStatus::InstalledExact
                                                    {
                                                        ui.label(
                                                            RichText::new("OK")
                                                                .strong()
                                                                .color(ACCENT_GREEN),
                                                        );
                                                    } else if !font.downloaded_files.is_empty() {
                                                        let btn = egui::Button::new(
                                                            RichText::new("Installer")
                                                                .color(Color32::WHITE)
                                                                .size(12.0),
                                                        )
                                                        .fill(ACCENT_GREEN_BTN)
                                                        .corner_radius(CornerRadius::same(6));
                                                        if ui.add_enabled(!busy, btn).clicked() {
                                                            pending_single_action =
                                                                Some((BatchAction::Install, idx));
                                                        }
                                                    } else if font.can_download_online() {
                                                        let btn = egui::Button::new(
                                                            RichText::new("Télécharger")
                                                                .color(Color32::WHITE)
                                                                .size(12.0),
                                                        )
                                                        .fill(ACCENT_BLUE_BTN)
                                                        .corner_radius(CornerRadius::same(6));
                                                        if ui.add_enabled(!busy, btn).clicked() {
                                                            pending_single_action = Some((
                                                                BatchAction::DownloadOrExtract,
                                                                idx,
                                                            ));
                                                        }
                                                    } else if font.can_extract_from_pdf() {
                                                        let btn = egui::Button::new(
                                                            RichText::new("Extraire du PDF")
                                                                .color(Color32::WHITE)
                                                                .size(12.0),
                                                        )
                                                        .fill(ACCENT_VIOLET_BTN)
                                                        .corner_radius(CornerRadius::same(6));
                                                        if ui.add_enabled(!busy, btn).clicked() {
                                                            pending_single_action = Some((
                                                                BatchAction::ExtractOnly,
                                                                idx,
                                                            ));
                                                        }
                                                    } else if ui
                                                        .button("Rechercher…")
                                                        .on_hover_text(
                                                            "Ouvrir une recherche web pour cette police",
                                                        )
                                                        .clicked()
                                                    {
                                                        pending_web_url =
                                                            Some(build_web_search_url(font));
                                                    }

                                                    ui.end_row();
                                                }
                                            }
                                        });
                                });
                        }
                    });

                ui.add_space(8.0);

                // 6. Barre dossier de téléchargement + actions groupées
                Frame::new()
                    .fill(BG_CARD)
                    .stroke(Stroke::new(1.0, BORDER_SUBTLE))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Dossier :")
                                    .strong()
                                    .color(TEXT_MUTED),
                            );
                            let mut dir_str = self.download_dir.to_string_lossy().to_string();
                            ui.add_enabled(
                                false,
                                egui::TextEdit::singleline(&mut dir_str).desired_width(300.0),
                            );

                            if ui.add_enabled(!self.busy, egui::Button::new("Parcourir…")).clicked() {
                                if let Some(chosen) = rfd::FileDialog::new()
                                    .set_title("Choisir le dossier de téléchargement")
                                    .set_directory(&self.download_dir)
                                    .pick_folder()
                                {
                                    self.download_dir = chosen;
                                }
                            }
                            if ui.button("Ouvrir").clicked() {
                                let _ = std::fs::create_dir_all(&self.download_dir);
                                let _ = open::that(&self.download_dir);
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let has_analysis = self.analysis.is_some();

                                    let install_btn = egui::Button::new(
                                        RichText::new("Installer sur le PC")
                                            .strong()
                                            .color(Color32::WHITE),
                                    )
                                    .fill(ACCENT_GREEN_BTN)
                                    .corner_radius(CornerRadius::same(7))
                                    .min_size(Vec2::new(146.0, 30.0));
                                    if ui
                                        .add_enabled(!self.busy && has_analysis, install_btn)
                                        .clicked()
                                    {
                                        let indices = self.selected_recoverable_indices(true);
                                        if indices.is_empty() {
                                            self.banner_message = Some((
                                                false,
                                                "Cochez au moins une police manquante (téléchargeable ou extractible du PDF)."
                                                    .to_string(),
                                            ));
                                        } else {
                                            self.start_batch_action(
                                                BatchAction::Install,
                                                indices,
                                                ctx,
                                            );
                                        }
                                    }

                                    let dl_btn = egui::Button::new(
                                        RichText::new("Télécharger la sélection")
                                            .strong()
                                            .color(Color32::WHITE),
                                    )
                                    .fill(ACCENT_BLUE_BTN)
                                    .corner_radius(CornerRadius::same(7))
                                    .min_size(Vec2::new(166.0, 30.0));
                                    if ui
                                        .add_enabled(!self.busy && has_analysis, dl_btn)
                                        .clicked()
                                    {
                                        let indices = self.selected_recoverable_indices(false);
                                        if indices.is_empty() {
                                            self.banner_message = Some((
                                                false,
                                                "Cochez au moins une police téléchargeable ou extractible du PDF."
                                                    .to_string(),
                                            ));
                                        } else {
                                            self.start_batch_action(
                                                BatchAction::DownloadOrExtract,
                                                indices,
                                                ctx,
                                            );
                                        }
                                    }
                                },
                            );
                        });
                    });

                // Exécuter les actions déclenchées depuis une ligne du tableau
                if let Some((act, idx)) = pending_single_action {
                    self.start_batch_action(act, vec![idx], ctx);
                }
                if let Some(url) = pending_web_url {
                    let _ = open::that(url);
                }
            });
    }
}

impl PdfFontsApp {
    fn selected_recoverable_indices(&self, for_install: bool) -> Vec<usize> {
        let Some(res) = self.analysis.as_ref() else {
            return Vec::new();
        };
        res.fonts
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                f.selected
                    && (!for_install || f.install_status != InstallStatus::InstalledExact)
                    && (!f.downloaded_files.is_empty()
                        || f.can_download_online()
                        || f.can_extract_from_pdf())
            })
            .map(|(i, _)| i)
            .collect()
    }
}

fn stat_card(ui: &mut egui::Ui, label: &str, value: usize, accent: Color32) {
    Frame::new()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(12, 9))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new(value.to_string()).size(20.0).strong().color(accent));
            ui.label(RichText::new(label).size(11.0).color(TEXT_MUTED));
        });
}

fn apply_custom_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = BG_APP;
    visuals.panel_fill = BG_APP;
    visuals.extreme_bg_color = BG_LOG;
    visuals.faint_bg_color = Color32::from_rgb(23, 32, 51);
    visuals.widgets.noninteractive.bg_fill = BG_CARD;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(51, 65, 85);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(51, 65, 85);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(6);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(71, 85, 105);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(6);
    visuals.widgets.active.bg_fill = Color32::from_rgb(30, 41, 59);
    visuals.selection.bg_fill = Color32::from_rgb(30, 58, 95);
    ctx.set_visuals(visuals);
}
