<div align="center">
  <p>
    <a href="../README.md">English</a> ·
    <a href="README.fr.md">Français</a> ·
    <a href="README.es.md">Español</a> ·
    <a href="README.de.md">Deutsch</a> ·
    <a href="README.it.md">Italiano</a> ·
    <a href="README.pt.md">Português</a> ·
    <a href="README.nl.md">Nederlands</a> ·
    <a href="README.pl.md">Polski</a> ·
    <strong>Русский</strong>
  </p>

  <img src="../assets/app_icon.svg" alt="Логотип PDF Fonts Detector" width="128" height="128" />

  <h1>PDF Fonts Detector</h1>

  <p>
    <strong>Определите все шрифты в любом PDF, проверьте их наличие на компьютере и скачайте, извлеките или установите недостающие в один клик.</strong>
  </p>

  <p>
    <a href="https://github.com/furcy75/PDF_Fonts_Detector/releases/latest">
      <img src="https://img.shields.io/github/v/release/furcy75/PDF_Fonts_Detector?style=for-the-badge&color=0284c7" alt="Последняя версия" />
    </a>
    <a href="https://github.com/sponsors/furcy75">
      <img src="https://img.shields.io/badge/Поддержать-GitHub_Sponsors-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white" alt="Поддержать на GitHub" />
    </a>
  </p>
</div>

---

## Обзор

**PDF Fonts Detector** — быстрое нативное и **на 100% автономное настольное приложение** на Rust (`egui` / `eframe`) для **Windows** и **Linux**. Перетащите один или несколько PDF-файлов в окно, чтобы мгновенно увидеть установленные и отсутствующие шрифты, автоматически скачать бесплатные шрифты (Google Fonts, Font Squirrel, open-source репозитории), извлечь полностью встроенные шрифты из PDF и установить их без прав администратора.

---

## Скриншоты

<div align="center">
  <img src="../assets/screenshots/app_main_en.png" alt="PDF Fonts Detector" width="900" />
</div>

---

## Основные возможности

- **100% автономные исполняемые файлы**: один нативный файл (~7–12 МБ) без Python, Qt, WebView, OpenSSL и GTK3.
- **Глубокий анализ PDF**: страницы, наследуемые `/Resources`, `CIDFonts` и `Form XObjects`, очистка префиксов подмножеств (`ABCDEF+Roboto-Bold` → `Roboto Bold`).
- **Точная проверка на ПК**: чтение внутренних таблиц OpenType/TrueType в **Windows** и **Linux**.
- **Бесплатная загрузка без API-ключей**: **Google Fonts**, **Font Squirrel** и open-source зеркала (Liberation, DejaVu, Cascadia).
- **Извлечение из PDF**: кнопка «Извлечь из PDF», если полный шрифт встроен в документ.
- **Установка без прав администратора** и **15 встроенных языков интерфейса**.

---

## Скачать

👉 **[Скачать последнюю версию](https://github.com/furcy75/PDF_Fonts_Detector/releases/latest)**

- **Windows (x64)**: скачайте и запустите `PDF-Fonts-Detector-Windows-x64.exe`.
- **Linux (x86_64)**: скачайте `PDF-Fonts-Detector-Linux-x86_64` или архив `PDF-Fonts-Detector-Linux-x86_64.tar.gz`.

---

## Поддержать проект

<div align="center">
  <a href="https://github.com/sponsors/furcy75">
    <img src="https://img.shields.io/badge/♥_Поддержать_на_GitHub_Sponsors-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white" alt="GitHub Sponsors" />
  </a>
</div>

---

## Лицензия

Лицензия **MIT**. Подробнее см. в [LICENSE](../LICENSE).
