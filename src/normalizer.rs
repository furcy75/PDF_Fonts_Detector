//! Normalisation et décomposition intelligente des noms de polices PDF.

use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFontName {
    pub raw_name: String,
    pub clean_name: String,
    pub family: String,
    pub style: String,
    pub has_subset_prefix: bool,
}

fn subset_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Z]{6}\+").unwrap())
}

fn tech_suffix_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)(?:[-_](?:Identity-H|Identity-V|WinAnsiEncoding|MacRomanEncoding|OTF|TTF))+$")
            .unwrap()
    })
}

fn lookup_postscript_alias(key: &str) -> Option<(&'static str, &'static str)> {
    match key {
        "arialmt" => Some(("Arial", "Regular")),
        "arial-boldmt" => Some(("Arial", "Bold")),
        "arial-italicmt" => Some(("Arial", "Italic")),
        "arial-bolditalicmt" => Some(("Arial", "Bold Italic")),
        "arialnarrow" => Some(("Arial Narrow", "Regular")),
        "arialnarrow-bold" => Some(("Arial Narrow", "Bold")),
        "arialnarrow-italic" => Some(("Arial Narrow", "Italic")),
        "arialnarrow-bolditalic" => Some(("Arial Narrow", "Bold Italic")),
        "arialblack" => Some(("Arial Black", "Regular")),
        "timesnewromanpsmt" => Some(("Times New Roman", "Regular")),
        "timesnewromanps-boldmt" => Some(("Times New Roman", "Bold")),
        "timesnewromanps-italicmt" => Some(("Times New Roman", "Italic")),
        "timesnewromanps-bolditalicmt" => Some(("Times New Roman", "Bold Italic")),
        "times-roman" => Some(("Times", "Regular")),
        "times-bold" => Some(("Times", "Bold")),
        "times-italic" => Some(("Times", "Italic")),
        "times-bolditalic" => Some(("Times", "Bold Italic")),
        "couriernewpsmt" => Some(("Courier New", "Regular")),
        "couriernewps-boldmt" => Some(("Courier New", "Bold")),
        "couriernewps-italicmt" => Some(("Courier New", "Italic")),
        "couriernewps-bolditalicmt" => Some(("Courier New", "Bold Italic")),
        "helvetica" => Some(("Helvetica", "Regular")),
        "helvetica-bold" => Some(("Helvetica", "Bold")),
        "helvetica-oblique" => Some(("Helvetica", "Oblique")),
        "helvetica-boldoblique" => Some(("Helvetica", "Bold Oblique")),
        "trebuchetms" => Some(("Trebuchet MS", "Regular")),
        "trebuchetms-bold" => Some(("Trebuchet MS", "Bold")),
        "trebuchetms-italic" => Some(("Trebuchet MS", "Italic")),
        "trebuchet-bolditalic" => Some(("Trebuchet MS", "Bold Italic")),
        "comicsansms" => Some(("Comic Sans MS", "Regular")),
        "comicsansms-bold" => Some(("Comic Sans MS", "Bold")),
        "segoeui" => Some(("Segoe UI", "Regular")),
        "segoeui-bold" => Some(("Segoe UI", "Bold")),
        "segoeui-italic" => Some(("Segoe UI", "Italic")),
        "segoeui-bolditalic" => Some(("Segoe UI", "Bold Italic")),
        "segoeui-semibold" => Some(("Segoe UI", "SemiBold")),
        "segoeui-light" => Some(("Segoe UI", "Light")),
        "lucidasansunicode" => Some(("Lucida Sans Unicode", "Regular")),
        "palatinolinotype-roman" => Some(("Palatino Linotype", "Regular")),
        "palatinolinotype-bold" => Some(("Palatino Linotype", "Bold")),
        "palatinolinotype-italic" => Some(("Palatino Linotype", "Italic")),
        "palatinolinotype-bolditalic" => Some(("Palatino Linotype", "Bold Italic")),
        "symbolmt" => Some(("Symbol", "Regular")),
        "wingdings-regular" => Some(("Wingdings", "Regular")),
        "zapfdingbats" => Some(("Zapf Dingbats", "Regular")),
        _ => None,
    }
}

fn lookup_style_token(key: &str) -> Option<&'static str> {
    match key {
        "ultralightitalic" | "extralightitalic" => Some("ExtraLight Italic"),
        "extrabolditalic" | "ultrabolditalic" => Some("ExtraBold Italic"),
        "semibolditalic" | "demibolditalic" => Some("SemiBold Italic"),
        "condensedbolditalic" => Some("Condensed Bold Italic"),
        "condenseditalic" => Some("Condensed Italic"),
        "condensedbold" => Some("Condensed Bold"),
        "hairlineitalic" | "thinitalic" => Some("Thin Italic"),
        "lightitalic" => Some("Light Italic"),
        "mediumitalic" => Some("Medium Italic"),
        "blackitalic" | "heavyitalic" => Some("Black Italic"),
        "bolditalic" | "bi" => Some("Bold Italic"),
        "boldoblique" => Some("Bold Oblique"),
        "extralight" | "ultralight" => Some("ExtraLight"),
        "extrabold" | "ultrabold" => Some("ExtraBold"),
        "semibold" | "demibold" => Some("SemiBold"),
        "hairline" | "thin" => Some("Thin"),
        "condensed" => Some("Condensed"),
        "extended" => Some("Extended"),
        "expanded" => Some("Expanded"),
        "narrow" => Some("Narrow"),
        "regular" | "normal" | "roman" | "reg" => Some("Regular"),
        "book" => Some("Book"),
        "light" => Some("Light"),
        "medium" | "med" => Some("Medium"),
        "black" => Some("Black"),
        "heavy" => Some("Heavy"),
        "bold" | "bd" => Some("Bold"),
        "italic" | "ital" | "it" => Some("Italic"),
        "oblique" | "obli" => Some("Oblique"),
        _ => None,
    }
}

/// Normalise une chaîne en clé alphanumérique minuscule pour comparaison tolérante.
pub fn normalize_key(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Normalise un style de police en clé canonique comparable.
pub fn normalize_style_key(style: &str) -> String {
    let cleaned = normalize_key(style);
    if cleaned.is_empty()
        || matches!(
            cleaned.as_str(),
            "regular" | "normal" | "roman" | "plain" | "standard"
        )
    {
        return "regular".to_string();
    }
    if let Some(canonical) = lookup_style_token(&cleaned) {
        return normalize_key(canonical);
    }
    cleaned
        .replace("oblique", "italic")
        .replace("demibold", "semibold")
        .replace("ultrabold", "extrabold")
        .replace("ultralight", "extralight")
}

/// Insère des espaces lisibles dans un nom CamelCase (ex. `OpenSans` -> `Open Sans`).
pub fn split_camel_case(name: &str) -> String {
    if name.contains(' ') {
        return name.split_whitespace().collect::<Vec<_>>().join(" ");
    }

    for preserved in ["DejaVu", "ABeeZee", "McLaren", "PT"] {
        if let Some(rest) = name.strip_prefix(preserved) {
            if !rest.is_empty() {
                let trimmed = rest.trim_start_matches(['-', '_']);
                let spaced_rest = split_camel_case(trimmed);
                if spaced_rest.is_empty() {
                    return preserved.to_string();
                }
                return format!("{preserved} {spaced_rest}");
            }
        }
    }

    let chars: Vec<char> = name.chars().collect();
    let mut out = String::with_capacity(name.len() + 6);
    for (i, &ch) in chars.iter().enumerate() {
        if ch == '_' {
            out.push(' ');
            continue;
        }
        if i > 0 {
            let prev = chars[i - 1];
            let next = chars.get(i + 1).copied();
            // Transition minuscule/chiffre -> Majuscule (OpenSans)
            let lower_to_upper = (prev.is_ascii_lowercase() || prev.is_ascii_digit())
                && ch.is_ascii_uppercase();
            // Acronyme suivi d'un mot capitalisé (IBMPlex -> IBM Plex)
            let acronym_end = prev.is_ascii_uppercase()
                && ch.is_ascii_uppercase()
                && next.is_some_and(|n| n.is_ascii_lowercase());
            // Lettre suivie d'un chiffre (SourceSans3 -> Source Sans 3)
            let alpha_to_digit = prev.is_ascii_alphabetic() && ch.is_ascii_digit();

            if lower_to_upper || acronym_end || alpha_to_digit {
                out.push(' ');
            }
        }
        out.push(ch);
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_style_part(style_part: &str) -> Option<String> {
    let key = normalize_key(style_part);
    if key.is_empty() {
        return None;
    }
    if let Some(canonical) = lookup_style_token(&key) {
        return Some(canonical.to_string());
    }

    let parts: Vec<&str> = style_part
        .split(|c: char| c.is_whitespace() || matches!(c, ',' | '_' | '/' | '-'))
        .filter(|s| !s.is_empty())
        .collect();
    if parts.len() > 1 {
        let mut mapped = Vec::with_capacity(parts.len());
        for p in parts {
            let pk = normalize_key(p);
            if let Some(canonical) = lookup_style_token(&pk) {
                mapped.push(canonical);
            } else {
                return None;
            }
        }
        return Some(mapped.join(" "));
    }
    None
}

fn strip_ps_suffix(name: &str) -> &str {
    for suffix in ["PSMT", "MT"] {
        if let Some(stripped) = name.strip_suffix(suffix) {
            if stripped.len() > 2 {
                return stripped.trim_end_matches('-');
            }
        }
    }
    name
}

/// Analyse un nom brut de police issu d'un PDF en `(famille, style, nom propre)`.
pub fn parse_pdf_font_name(raw_name: &str) -> ParsedFontName {
    let raw = raw_name.trim().trim_start_matches('/');
    let has_subset = subset_regex().is_match(raw);
    let without_subset = subset_regex().replace(raw, "");
    let clean = tech_suffix_regex()
        .replace(without_subset.as_ref(), "")
        .trim()
        .to_string();

    if clean.is_empty() {
        let fallback = if raw.is_empty() { "Unknown" } else { raw };
        return ParsedFontName {
            raw_name: raw_name.to_string(),
            clean_name: fallback.to_string(),
            family: fallback.to_string(),
            style: "Regular".to_string(),
            has_subset_prefix: has_subset,
        };
    }

    // 1. Alias PostScript standards (ArialMT, TimesNewRomanPSMT…)
    let alias_key: String = clean
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_lowercase())
        .collect();
    if let Some((fam, sty)) = lookup_postscript_alias(&alias_key) {
        return ParsedFontName {
            raw_name: raw_name.to_string(),
            clean_name: clean,
            family: fam.to_string(),
            style: sty.to_string(),
            has_subset_prefix: has_subset,
        };
    }

    // 2. Format Windows avec virgule ("Calibri,BoldItalic")
    if let Some((left, right)) = clean.split_once(',') {
        let parsed_style = parse_style_part(right).unwrap_or_else(|| {
            let trimmed = right.trim();
            if trimmed.is_empty() {
                "Regular".to_string()
            } else {
                trimmed.to_string()
            }
        });
        let family_str = split_camel_case(strip_ps_suffix(left.trim()));
        return ParsedFontName {
            raw_name: raw_name.to_string(),
            clean_name: clean,
            family: family_str,
            style: parsed_style,
            has_subset_prefix: has_subset,
        };
    }

    // 3. Format PostScript avec tiret ("Roboto-BoldItalic", "Roboto-Condensed-Bold")
    if clean.contains('-') {
        let parts: Vec<&str> = clean.split('-').collect();
        if parts.len() >= 2 {
            let last_clean = strip_ps_suffix(parts[parts.len() - 1]);
            if let Some(mut parsed_style) = parse_style_part(last_clean) {
                let mut family_end = parts.len() - 1;
                if parts.len() >= 3 {
                    if let Some(prev_style) = parse_style_part(parts[parts.len() - 2]) {
                        family_end = parts.len() - 2;
                        parsed_style = format!("{prev_style} {parsed_style}");
                    }
                }
                let family_raw = parts[..family_end].join("-");
                let family_str = split_camel_case(strip_ps_suffix(&family_raw));
                return ParsedFontName {
                    raw_name: raw_name.to_string(),
                    clean_name: clean,
                    family: family_str,
                    style: parsed_style,
                    has_subset_prefix: has_subset,
                };
            }
        }
    }

    // 4. Espaces ou CamelCase collé ("Roboto Bold", "OpenSansSemiBold")
    let working = strip_ps_suffix(&clean);
    let spaced = if working.contains(' ') {
        working.to_string()
    } else {
        split_camel_case(working)
    };
    let words: Vec<&str> = spaced.split_whitespace().collect();
    if words.len() >= 3 {
        let last_two = format!("{} {}", words[words.len() - 2], words[words.len() - 1]);
        if let Some(two_word_style) = parse_style_part(&last_two) {
            return ParsedFontName {
                raw_name: raw_name.to_string(),
                clean_name: clean,
                family: words[..words.len() - 2].join(" "),
                style: two_word_style,
                has_subset_prefix: has_subset,
            };
        }
    }
    if words.len() >= 2 {
        if let Some(one_word_style) = parse_style_part(words[words.len() - 1]) {
            return ParsedFontName {
                raw_name: raw_name.to_string(),
                clean_name: clean,
                family: words[..words.len() - 1].join(" "),
                style: one_word_style,
                has_subset_prefix: has_subset,
            };
        }
    }

    ParsedFontName {
        raw_name: raw_name.to_string(),
        clean_name: clean,
        family: spaced,
        style: "Regular".to_string(),
        has_subset_prefix: has_subset,
    }
}

/// Génère plusieurs variantes de recherche de famille pour maximiser la correspondance catalogue.
pub fn family_search_candidates(family: &str) -> Vec<String> {
    static STRIP_TAGS_RE: OnceLock<Regex> = OnceLock::new();
    static STRIP_WIDTH_RE: OnceLock<Regex> = OnceLock::new();

    let base = family.trim();
    if base.is_empty() {
        return Vec::new();
    }
    let mut candidates = vec![base.to_string()];

    let re_tags = STRIP_TAGS_RE
        .get_or_init(|| Regex::new(r"(?i)\b(?:Pro|Std|LT|MT|PS|Web|VF)\b").unwrap());
    let stripped = re_tags
        .replace_all(base, "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if !stripped.is_empty() && !candidates.contains(&stripped) {
        candidates.push(stripped.clone());
    }

    let re_width = STRIP_WIDTH_RE.get_or_init(|| {
        Regex::new(r"(?i)\b(?:Condensed|Narrow|Extended|Expanded|SemiCondensed)\b").unwrap()
    });
    let base_for_width = if stripped.is_empty() {
        base
    } else {
        stripped.as_str()
    };
    let no_width = re_width
        .replace_all(base_for_width, "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if !no_width.is_empty() && !candidates.contains(&no_width) {
        candidates.push(no_width);
    }

    candidates
}

/// Nettoie un nom de fichier/dossier pour Windows et Linux.
pub fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || c.is_control()
            {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches(|c: char| c.is_whitespace() || c == '.');
    if trimmed.is_empty() {
        "font".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_subset_and_parse_hyphen() {
        let parsed = parse_pdf_font_name("ABCDEF+Roboto-BoldItalic");
        assert!(parsed.has_subset_prefix);
        assert_eq!(parsed.clean_name, "Roboto-BoldItalic");
        assert_eq!(parsed.family, "Roboto");
        assert_eq!(parsed.style, "Bold Italic");
    }

    #[test]
    fn test_postscript_aliases() {
        let p1 = parse_pdf_font_name("BCDEEE+Arial-BoldMT");
        assert_eq!(p1.family, "Arial");
        assert_eq!(p1.style, "Bold");
        assert!(p1.has_subset_prefix);

        let p2 = parse_pdf_font_name("TimesNewRomanPS-ItalicMT");
        assert_eq!(p2.family, "Times New Roman");
        assert_eq!(p2.style, "Italic");
        assert!(!p2.has_subset_prefix);
    }

    #[test]
    fn test_comma_and_camel_case() {
        let p1 = parse_pdf_font_name("Calibri,BoldItalic");
        assert_eq!(p1.family, "Calibri");
        assert_eq!(p1.style, "Bold Italic");

        let p2 = parse_pdf_font_name("XYZABC+OpenSans-SemiBold");
        assert_eq!(p2.family, "Open Sans");
        assert_eq!(p2.style, "SemiBold");
    }

    #[test]
    fn test_family_candidates() {
        let cands = family_search_candidates("Source Sans Pro Condensed");
        assert!(cands.contains(&"Source Sans Pro Condensed".to_string()));
        assert!(cands.contains(&"Source Sans Condensed".to_string()));
        assert!(cands.contains(&"Source Sans".to_string()));
    }
}
