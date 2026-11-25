<div align="center">
  <p>
    <a href="../README.md">English</a> ·
    <a href="README.fr.md">Français</a> ·
    <a href="README.es.md">Español</a> ·
    <strong>Deutsch</strong> ·
    <a href="README.it.md">Italiano</a> ·
    <a href="README.pt.md">Português</a> ·
    <a href="README.nl.md">Nederlands</a> ·
    <a href="README.pl.md">Polski</a> ·
    <a href="README.ru.md">Русский</a>
  </p>

  <img src="../assets/app_icon.svg" alt="PDF Fonts Detector Logo" width="128" height="128" />

  <h1>PDF Fonts Detector</h1>

  <p>
    <strong>Erkennen Sie alle Schriftarten in jedem PDF, prüfen Sie Ihren PC und laden, extrahieren oder installieren Sie fehlende Schriften mit einem Klick.</strong>
  </p>

  <p>
    <a href="https://github.com/furcy75/PDF_Fonts_Detector/releases/latest">
      <img src="https://img.shields.io/github/v/release/furcy75/PDF_Fonts_Detector?style=for-the-badge&color=0284c7" alt="Neueste Version" />
    </a>
    <a href="https://github.com/sponsors/furcy75">
      <img src="https://img.shields.io/badge/Sponsern-GitHub_Sponsors-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white" alt="Auf GitHub sponsern" />
    </a>
  </p>
</div>

---

## Überblick

**PDF Fonts Detector** ist eine schnelle, native und **100 % eigenständige Desktop-Anwendung** in Rust (`egui` / `eframe`) für **Windows** und **Linux**. Ziehen Sie eine oder mehrere PDF-Dateien ins Fenster, um sofort zu sehen, welche Schriftarten bereits installiert sind, kostenlose Schriften automatisch herunterzuladen (Google Fonts, Font Squirrel, Open-Source-Quellen), vollständig eingebettete Schriften direkt aus dem PDF zu extrahieren und sie ohne Administratorrechte zu installieren.

---

## Screenshots

<div align="center">
  <img src="../assets/screenshots/app_main_en.png" alt="PDF Fonts Detector" width="900" />
</div>

---

## Hauptfunktionen

- **100 % eigenständige Programme**: eine einzige native Datei (~7–12 MB) ohne Python, Qt, WebView, OpenSSL oder GTK3.
- **Tiefgehende PDF-Analyse**: liest Seiten, vererbte `/Resources`, `CIDFonts` und `Form XObjects`; bereinigt Teilmengen-Präfixe (`ABCDEF+Roboto-Bold` → `Roboto Bold`) und PostScript-Aliase.
- **Prüfung der lokalen PC-Schriften**: liest interne OpenType/TrueType-Tabellen auf **Windows** und **Linux**.
- **Kostenlose Downloads ohne API-Schlüssel**: **Google Fonts**, **Font Squirrel** und Open-Source-Quellen (Liberation, DejaVu, Cascadia).
- **Extraktion aus dem PDF**: ermöglicht „Aus PDF extrahieren“, wenn eine vollständige Schriftart direkt im PDF eingebettet ist.
- **Installation ohne Admin-Rechte**: benutzerbezogene Installation unter Windows und Linux.
- **15 Sprachen integriert**: automatische Erkennung der Systemsprache und sofortiger Wechsel in der Oberfläche.

---

## Download

👉 **[Neueste Version herunterladen](https://github.com/furcy75/PDF_Fonts_Detector/releases/latest)**

- **Windows (x64)**: `PDF-Fonts-Detector-Windows-x64.exe` herunterladen und starten.
- **Linux (x86_64)**: `PDF-Fonts-Detector-Linux-x86_64` oder das Archiv `PDF-Fonts-Detector-Linux-x86_64.tar.gz` (inkl. Icon & `.desktop`-Skript).

---

## Projekt unterstützen

<div align="center">
  <a href="https://github.com/sponsors/furcy75">
    <img src="https://img.shields.io/badge/♥_Auf_GitHub_Sponsors_unterstützen-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white" alt="GitHub Sponsors" />
  </a>
</div>

---

## Lizenz

Veröffentlicht unter der **MIT-Lizenz**. Siehe [LICENSE](../LICENSE).
