<div align="center">
  <p>
    <a href="../README.md">English</a> ·
    <a href="README.fr.md">Français</a> ·
    <strong>Español</strong> ·
    <a href="README.de.md">Deutsch</a> ·
    <a href="README.it.md">Italiano</a> ·
    <a href="README.pt.md">Português</a> ·
    <a href="README.nl.md">Nederlands</a> ·
    <a href="README.pl.md">Polski</a> ·
    <a href="README.ru.md">Русский</a>
  </p>

  <img src="../assets/app_icon.svg" alt="Logo PDF Fonts Detector" width="128" height="128" />

  <h1>PDF Fonts Detector</h1>

  <p>
    <strong>Detecta todas las fuentes de cualquier PDF, comprueba cuáles están instaladas en tu PC y descarga, extrae o instala las faltantes con un solo clic.</strong>
  </p>

  <p>
    <a href="https://github.com/furcy75/PDF_Fonts_Detector/releases/latest">
      <img src="https://img.shields.io/github/v/release/furcy75/PDF_Fonts_Detector?style=for-the-badge&color=0284c7" alt="Última versión" />
    </a>
    <a href="https://github.com/sponsors/furcy75">
      <img src="https://img.shields.io/badge/Patrocinar-GitHub_Sponsors-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white" alt="Patrocinar en GitHub" />
    </a>
  </p>
</div>

---

## Descripción general

**PDF Fonts Detector** es una aplicación de escritorio nativa, rápida y **100 % autónoma**, escrita en Rust (`egui` / `eframe`) para **Windows** y **Linux**. Arrastra uno o varios archivos PDF para ver al instante qué fuentes están instaladas, descargar fuentes gratuitas (Google Fonts, Font Squirrel, repositorios libres), extraer fuentes incrustadas completas directamente del PDF e instalarlas sin permisos de administrador.

---

## Capturas de pantalla

<div align="center">
  <img src="../assets/screenshots/app_main_en.png" alt="PDF Fonts Detector" width="900" />
</div>

---

## Características principales

- **Ejecutables 100 % autónomos**: un único binario nativo (~7–12 MB), arranque instantáneo, sin Python, Qt, WebView, OpenSSL ni GTK3.
- **Análisis profundo de PDF**: inspecciona páginas, `/Resources` heredados, `CIDFonts` y `Form XObjects`, limpia prefijos de subconjunto (`ABCDEF+Roboto-Bold` → `Roboto Bold`) y alias PostScript.
- **Verificación precisa en el PC**: lee las tablas internas OpenType/TrueType en **Windows** y **Linux**.
- **Descarga automática gratuita**: **Google Fonts**, **Font Squirrel** y repositorios open-source (Liberation, DejaVu, Cascadia), sin clave API.
- **Extracción desde el PDF**: si una fuente no está en catálogos gratuitos pero está incrustada completamente en el PDF, permite extraerla en `.ttf`/`.otf`.
- **Instalación sin administrador**: instalación a nivel de usuario en Windows y Linux.
- **15 idiomas integrados**: detección automática del idioma del sistema y cambio en vivo.

---

## Descarga

👉 **[Descargar la última versión](https://github.com/furcy75/PDF_Fonts_Detector/releases/latest)**

- **Windows (x64)**: descarga y ejecuta `PDF-Fonts-Detector-Windows-x64.exe`.
- **Linux (x86_64)**: descarga `PDF-Fonts-Detector-Linux-x86_64` o `PDF-Fonts-Detector-Linux-x86_64.tar.gz` (incluye icono y script `.desktop`).

---

## Apoya el proyecto

<div align="center">
  <a href="https://github.com/sponsors/furcy75">
    <img src="https://img.shields.io/badge/♥_Patrocinar_en_GitHub_Sponsors-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white" alt="Patrocinar en GitHub" />
  </a>
</div>

---

## Licencia

Licencia **MIT**. Consulta [LICENSE](../LICENSE).
