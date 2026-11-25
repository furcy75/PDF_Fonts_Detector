<div align="center">
  <p>
    <a href="../README.md">English</a> ·
    <a href="README.fr.md">Français</a> ·
    <a href="README.es.md">Español</a> ·
    <a href="README.de.md">Deutsch</a> ·
    <strong>Italiano</strong> ·
    <a href="README.pt.md">Português</a> ·
    <a href="README.nl.md">Nederlands</a> ·
    <a href="README.pl.md">Polski</a> ·
    <a href="README.ru.md">Русский</a>
  </p>

  <img src="../assets/app_icon.svg" alt="Logo PDF Fonts Detector" width="128" height="128" />

  <h1>PDF Fonts Detector</h1>

  <p>
    <strong>Rileva tutti i font presenti in un PDF, verifica quelli già installati sul PC e scarica, estrai o installa i font mancanti in un clic.</strong>
  </p>

  <p>
    <a href="https://github.com/furcy75/PDF_Fonts_Detector/releases/latest">
      <img src="https://img.shields.io/github/v/release/furcy75/PDF_Fonts_Detector?style=for-the-badge&color=0284c7" alt="Ultima versione" />
    </a>
    <a href="https://github.com/sponsors/furcy75">
      <img src="https://img.shields.io/badge/Sponsorizza-GitHub_Sponsors-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white" alt="Sponsorizza su GitHub" />
    </a>
  </p>
</div>

---

## Panoramica

**PDF Fonts Detector** è un'applicazione desktop nativa, veloce e **100% autonoma**, scritta in Rust (`egui` / `eframe`) per **Windows** e **Linux**. Trascina uno o più PDF nella finestra per vedere subito quali font sono installati, scaricare i font gratuiti (Google Fonts, Font Squirrel, repository open-source), estrarre i font completi incorporati nel PDF e installarli senza privilegi di amministratore.

---

## Schermate

<div align="center">
  <img src="../assets/screenshots/app_main_en.png" alt="PDF Fonts Detector" width="900" />
</div>

---

## Funzionalità principali

- **Eseguibili 100% autonomi**: un unico binario nativo (~7–12 MB), avvio istantaneo, senza Python, Qt, WebView, OpenSSL o GTK3.
- **Analisi PDF completa**: pagine, `/Resources` ereditati, `CIDFonts` e `Form XObjects`, rimozione dei prefissi subset (`ABCDEF+Roboto-Bold` → `Roboto Bold`) e alias PostScript.
- **Verifica accurata sul PC**: lettura delle tabelle interne OpenType/TrueType su **Windows** e **Linux**.
- **Download gratuito senza chiave API**: **Google Fonts**, **Font Squirrel** e mirror open-source (Liberation, DejaVu, Cascadia).
- **Estrazione dal PDF**: consente di estrarre direttamente i font completi incorporati nel PDF in formato `.ttf`/`.otf`.
- **Installazione senza permessi admin**: installazione nella sessione utente su Windows e Linux.
- **15 lingue integrate**: rilevamento automatico della lingua del sistema e cambio immediato.

---

## Download

👉 **[Scarica l'ultima versione](https://github.com/furcy75/PDF_Fonts_Detector/releases/latest)**

- **Windows (x64)**: scarica ed esegui `PDF-Fonts-Detector-Windows-x64.exe`.
- **Linux (x86_64)**: scarica `PDF-Fonts-Detector-Linux-x86_64` o `PDF-Fonts-Detector-Linux-x86_64.tar.gz` (con icona e script `.desktop`).

---

## Sostieni il progetto

<div align="center">
  <a href="https://github.com/sponsors/furcy75">
    <img src="https://img.shields.io/badge/♥_Sostieni_su_GitHub_Sponsors-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white" alt="GitHub Sponsors" />
  </a>
</div>

---

## Licenza

Distribuito con licenza **MIT**. Vedi [LICENSE](../LICENSE).
