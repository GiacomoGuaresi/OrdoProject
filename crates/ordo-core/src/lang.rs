//! Supported languages and system language detection.

use serde::{Deserialize, Serialize};

/// A language supported by Ordo's user-facing texts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    En,
    It,
}

impl Lang {
    /// All supported languages.
    pub const ALL: [Lang; 2] = [Lang::En, Lang::It];

    /// ISO 639-1 code of the language.
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::It => "it",
        }
    }

    /// Parses a locale/language tag such as `it`, `it-IT`, `it_IT.UTF-8` or `EN`.
    pub fn from_tag(tag: &str) -> Option<Lang> {
        let primary = tag
            .split(['-', '_', '.', '@'])
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        match primary.as_str() {
            "en" => Some(Lang::En),
            "it" => Some(Lang::It),
            _ => None,
        }
    }

    /// The operating system language, falling back to English.
    pub fn system() -> Lang {
        sys_locale::get_locale()
            .and_then(|l| Lang::from_tag(&l))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tags() {
        assert_eq!(Lang::from_tag("it"), Some(Lang::It));
        assert_eq!(Lang::from_tag("it-IT"), Some(Lang::It));
        assert_eq!(Lang::from_tag("it_IT.UTF-8"), Some(Lang::It));
        assert_eq!(Lang::from_tag("EN-us"), Some(Lang::En));
        assert_eq!(Lang::from_tag("fr-FR"), None);
        assert_eq!(Lang::from_tag(""), None);
    }

    #[test]
    fn serializes_lowercase() {
        assert_eq!(serde_json::to_string(&Lang::It).unwrap(), "\"it\"");
    }
}
