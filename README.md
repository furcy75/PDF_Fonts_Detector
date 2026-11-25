<div align="center">
  <p>
    <strong>English</strong> ·
    <a href="docs/README.fr.md">Français</a> ·
    <a href="docs/README.es.md">Español</a> ·
    <a href="docs/README.de.md">Deutsch</a> ·
    <a href="docs/README.it.md">Italiano</a> ·
    <a href="docs/README.pt.md">Português</a> ·
    <a href="docs/README.nl.md">Nederlands</a> ·
    <a href="docs/README.pl.md">Polski</a> ·
    <a href="docs/README.ru.md">Русский</a>
  </p>

  <img src="assets/app_icon.svg" alt="PDF Fonts Detector Logo" width="128" height="128" />

  <h1>PDF Fonts Detector</h1>

  <p>
    <strong>Inspect every font inside any PDF, check what is already installed on your PC, and download, extract, or install missing fonts in one click.</strong>
  </p>

  <p>
    <a href="https://github.com/furcy75/PDF_Fonts_Detector/releases/latest">
      <img src="https://img.shields.io/github/v/release/furcy75/PDF_Fonts_Detector?style=for-the-badge&color=0284c7" alt="Latest Release" />
    </a>
    <a href="https://github.com/furcy75/PDF_Fonts_Detector/actions/workflows/build.yml">
      <img src="https://img.shields.io/github/actions/workflow/status/furcy75/PDF_Fonts_Detector/build.yml?style=for-the-badge&label=Build" alt="Build Status" />
    </a>
    <img src="https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-1e293b?style=for-the-badge" alt="Platform Windows & Linux" />
    <img src="https://img.shields.io/badge/Built%20with-Rust-f97316?style=for-the-badge&logo=rust&logoColor=white" alt="Built with Rust" />
    <a href="LICENSE">
      <img src="https://img.shields.io/badge/License-MIT-10b981?style=for-the-badge" alt="MIT License" />
    </a>
    <a href="https://github.com/sponsors/furcy75">
      <img src="https://img.shields.io/badge/Sponsor-GitHub_Sponsors-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white" alt="Sponsor on GitHub" />
    </a>
  </p>

  <p>
    <a href="#screenshots">Screenshots</a> •
    <a href="#download">Download</a> •
    <a href="#key-features">Features</a> •
    <a href="#how-it-works">How It Works</a> •
    <a href="#multilingual-support">Languages</a> •
    <a href="#building-from-source">Build from Source</a> •
    <a href="#support-the-project">Sponsor</a>
  </p>
</div>

---

## Overview

**PDF Fonts Detector** is a fast, native, **100% standalone desktop application** written in Rust (`egui` / `eframe`) for **Windows** and **Linux**.

Opening a PDF in a design tool, office suite, or publishing workflow often breaks typography when fonts are missing locally. PDF Fonts Detector eliminates manual font hunting: drop one or more PDF files onto the window, immediately see which fonts are installed vs. missing, download free fonts automatically from official catalogs, extract complete embedded fonts directly from the PDF when available, and install them in one click without administrator privileges.

---

## Screenshots

<div align="center">
  <img src="assets/screenshots/app_main_en.png" alt="PDF Fonts Detector — English Interface" width="900" />
  <p><em>Analyzing a PDF document: instant detection of installed fonts, Google Fonts downloads, and unavailable commercial fonts.</em></p>
  <br />
  <img src="assets/screenshots/app_main_fr.png" alt="PDF Fonts Detector — Multilingual Interface (French)" width="900" />
  <p><em>Live multilingual switching across 15 languages directly from the header bar.</em></p>
</div>

---

## Key Features

| Feature | Description |
| :--- | :--- |
| **100% Standalone Executables** | Single native binary (~7–12 MB) with instant startup. **Zero runtime dependencies**: no Python, no Qt, no Electron/WebView, no OpenSSL, and no GTK3 required. |
| **Deep PDF Font Inspection** | Recursively parses pages, inherited `/Resources`, Type0/CID `/DescendantFonts`, and nested Form `/XObject` streams. Strips subset tags (`ABCDEF+Roboto-Bold` → `Roboto Bold`) and resolves PostScript/Microsoft aliases (`ArialMT`, `TimesNewRomanPSMT`, `Calibri,BoldItalic`). |
| **Accurate Local PC Verification** | Reads internal OpenType/TrueType `name` tables (`ttf-parser`) across **Windows** (`C:\Windows\Fonts`, `%LOCALAPPDATA%\Microsoft\Windows\Fonts`, Registry) and **Linux** (`fc-list`, `/usr/share/fonts`, `/usr/local/share/fonts`, `~/.local/share/fonts`). Distinguishes exact style matches from family-only matches. |
| **Zero-Config Free Font Downloads** | Automatically queries **Google Fonts** (official `.ttf` ZIP archives + CSS2 fallback), **Font Squirrel**, and curated **open-source mirrors** (Liberation, DejaVu, Cascadia Code/Mono). No API keys required. |
| **Complete Embedded Font Extraction** | For fonts unavailable in free catalogs, inspects the embedded binary stream (`ttf-parser` glyph count & alphabet coverage) and enables **"Extract from PDF"** whenever a complete, non-subsetted `.ttf`/`.otf` font is embedded. |
| **One-Click No-Admin Installation** | Installs fonts into the current user session: **Windows** (`%LOCALAPPDATA%\Microsoft\Windows\Fonts` + `HKCU` registry + live `WM_FONTCHANGE` broadcast) and **Linux** (`~/.local/share/fonts/PDF_Fonts_Detector` + `fc-cache -f`). |
| **15 Languages Built In** | Automatically detects your OS language on startup and allows live switching from the header bar with persistent preferences. |

---

## Download

Pre-built standalone binaries are published automatically via GitHub Actions on every release:

👉 **[Download Latest Release](https://github.com/furcy75/PDF_Fonts_Detector/releases/latest)**

### Windows (x64)
1. Download **`PDF-Fonts-Detector-Windows-x64.exe`**.
2. Double-click to run — no installer or runtime needed.

### Linux (x86_64)
Compiled on Ubuntu 22.04 (`glibc 2.35`, linking only against `libc`, `libm`, and `libgcc_s`) with pure-Rust TLS (`rustls`) and XDG Desktop Portal file dialogs (`rfd`):

- **Direct standalone binary**:
  ```bash
  chmod +x PDF-Fonts-Detector-Linux-x86_64
  ./PDF-Fonts-Detector-Linux-x86_64
  ```
- **Portable archive with desktop shortcut & icon** (`PDF-Fonts-Detector-Linux-x86_64.tar.gz`):
  ```bash
  tar -xzf PDF-Fonts-Detector-Linux-x86_64.tar.gz
  cd PDF-Fonts-Detector-Linux-x86_64
  ./install-desktop-shortcut.sh   # Optional: adds app launcher & icon to your desktop menu
  ./pdf-fonts-detector
  ```

> **Tip:** You can also pass PDF file paths or a language override on the command line:
> ```bash
> pdf-fonts-detector --lang=en document.pdf
> ```

---

## How It Works

```mermaid
flowchart LR
    A["Drop PDF(s)"] --> B["Parse PDF Objects\n(Pages, XObjects, CIDFonts)"]
    B --> C["Normalize Names\n(Strip Subset Tags & Aliases)"]
    C --> D["Scan System Fonts\n(Windows / Linux)"]
    D --> E{"Installed on PC?"}
    E -- "Yes" --> F["Marked Installed"]
    E -- "No" --> G["Query Free Catalogs\n(Google Fonts, Font Squirrel, Mirrors)"]
    G -- "Found" --> H["Download .ttf / .otf"]
    G -- "Not Found" --> I{"Complete Font\nEmbedded in PDF?"}
    I -- "Yes" --> J["Extract from PDF"]
    I -- "Subset / Missing" --> K["Not Found + Web Search"]
    H --> L["Install to User Fonts\n(No Admin Required)"]
    J --> L
```

### About Commercial & Subsetted PDF Fonts

- **Commercial / proprietary fonts** (e.g., *Helvetica Neue*, *Futura*, *Gotham*, *Adobe Fonts*) cannot be legally downloaded from free repositories. PDF Fonts Detector clearly marks them as **Not found** and provides a one-click web search button.
- **Subsets vs. Complete embedded fonts**: Most PDF generators embed only a *subset* of glyphs actually used in the document (identified by a 6-letter `ABCDEF+` tag or incomplete character maps). PDF Fonts Detector verifies glyph coverage via `ttf-parser` and only unlocks **Extract from PDF** when a complete, usable TrueType/OpenType font is embedded.

---

## Multilingual Support

PDF Fonts Detector automatically detects your operating system locale and lets you switch languages live at any time from the top-right selector. Supported UI languages (**15**):

| | | | | |
| :--- | :--- | :--- | :--- | :--- |
| **English** (`en`) | **Français** (`fr`) | **Español** (`es`) | **Deutsch** (`de`) | **Italiano** (`it`) |
| **Português** (`pt`) | **Nederlands** (`nl`) | **Polski** (`pl`) | **Svenska** (`sv`) | **Dansk** (`da`) |
| **Čeština** (`cs`) | **Română** (`ro`) | **Türkçe** (`tr`) | **Русский** (`ru`) | **Українська** (`uk`) |

Selected language and download folder preferences are saved automatically in your OS user configuration directory (`pdf-fonts-detector/settings.json`).

---

## Building from Source

### Prerequisites

- **Rust** (stable toolchain — install via [rustup.rs](https://rustup.rs/))
- **Linux build headers** (compile-time only; not required at runtime on target machines):
  ```bash
  sudo apt-get update
  sudo apt-get install -y libxkbcommon-dev libwayland-dev libgl1-mesa-dev libx11-dev libxi-dev libxrandr-dev libxcursor-dev
  ```

### Build & Test

```bash
git clone https://github.com/furcy75/PDF_Fonts_Detector.git
cd PDF_Fonts_Detector

# Run unit tests
cargo test

# Build optimized standalone binary
cargo build --release
```

---

## Support the Project

If **PDF Fonts Detector** saves you time, please consider supporting its ongoing development and maintenance:

<div align="center">
  <a href="https://github.com/sponsors/furcy75">
    <img src="https://img.shields.io/badge/♥_Sponsor_on_GitHub-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white" alt="Sponsor furcy75 on GitHub" />
  </a>
</div>

You can also support the project by starring the repository, reporting bugs, or contributing new translations and features!

---

## License

Distributed under the **MIT License**. See [LICENSE](LICENSE) for more information.
