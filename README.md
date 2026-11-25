# PDF Fonts Detector

Application de bureau native en **Rust** (**exécutables 100 % autonomes pour Windows & Linux**, ~10 Mo, démarrage instantané, sans Python, sans Qt, sans WebView, sans OpenSSL) permettant de détecter toutes les polices d'un PDF, vérifier leur présence sur votre ordinateur, les télécharger ou les extraire du PDF, puis les installer en un clic.

## Fonctionnalités

- **Exécutables 100 % autonomes** :
  - **Windows** : un seul fichier `PDF-Fonts-Detector-Windows-x64.exe` (aucun runtime à installer).
  - **Linux** : un seul binaire `PDF-Fonts-Detector-Linux-x86_64` (compilé sur Ubuntu 22.04 avec TLS natif `rustls` et boîtes de dialogue XDG Portal sans GTK3/Qt).
- **Analyse PDF complète** : glisser-déposer d'un ou plusieurs PDF, parcours des pages et Form XObjects, nettoyage des préfixes de sous-ensembles (`ABCDEF+Roboto-Bold` → `Roboto Bold`) et alias PostScript (`ArialMT`, `TimesNewRomanPSMT`, `Calibri,BoldItalic`…).
- **Comparaison ultra-rapide avec le PC** :
  - **Windows** : scan de `C:\Windows\Fonts`, `%LOCALAPPDATA%\Microsoft\Windows\Fonts` et du registre Windows.
  - **Linux** : interrogation de `fc-list` + scan de `/usr/share/fonts`, `/usr/local/share/fonts` et `~/.local/share/fonts`.
  - Lecture native des tables OpenType/TrueType (`ttf-parser`).
- **Téléchargement automatique (sans clé API)** :
  - **Google Fonts** (archive officielle `.ttf` + fallback API CSS2),
  - **Miroirs open-source officiels** (Liberation, DejaVu, Cascadia…),
  - **Font Squirrel** (catalogue libre `.ttf`/`.otf`).
- **Polices commerciales / introuvables** :
  - Si la police est **intégrée complètement** dans le PDF (non sous-ensemble), le bouton **« Extraire du PDF »** permet de la récupérer directement en `.ttf` / `.otf`.
  - Sinon, le logiciel affiche **« Introuvable »** avec un raccourci de recherche web.
- **Installation sans droits administrateur** :
  - **Windows** : copie dans `%LOCALAPPDATA%\Microsoft\Windows\Fonts`, inscription dans `HKCU\...\Fonts` et notification immédiate (`WM_FONTCHANGE`).
  - **Linux** : copie dans `~/.local/share/fonts/PDF_Fonts_Detector` + `fc-cache -f`.

## Télécharger les exécutables

Rendez-vous dans l'onglet **[Releases](https://github.com/furcy75/PDF_Fonts_Detector/releases)** (ou dans les artefacts de **GitHub Actions**) :

- **Windows** : `PDF-Fonts-Detector-Windows-x64.exe`
- **Linux** : `PDF-Fonts-Detector-Linux-x86_64` (ou l'archive `PDF-Fonts-Detector-Linux-x86_64.tar.gz` incluant le script de raccourci `.desktop`)

## Compiler depuis les sources

Nécessite [Rust](https://rustup.rs/) (stable) :

```bash
cargo test
cargo build --release
```

Le binaire généré se trouve dans `target/release/pdf-fonts-detector` (ou `.exe` sous Windows).
