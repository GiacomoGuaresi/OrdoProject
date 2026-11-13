//! Rule model, matching and JSON persistence.
//!
//! Rules form a single ordered list; the first enabled rule whose pattern
//! matches the file name wins.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::lang::Lang;
use crate::paths::write_atomic;

/// Current version of the `rules.json` schema.
pub const RULES_FILE_VERSION: u32 = 1;

/// Kind of pattern used to match the file name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatternKind {
    /// Shell-style glob, e.g. `*.{jpg,png}`.
    Glob,
    /// Regular expression, e.g. `^invoice_\d{4}\.pdf$`.
    Regex,
}

/// A sorting rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub kind: PatternKind,
    pub pattern: String,
    /// Case-sensitive matching (default: case-insensitive).
    #[serde(default)]
    pub case_sensitive: bool,
    /// Fixed destination folder.
    pub destination: PathBuf,
}

impl Rule {
    /// Creates an enabled, case-insensitive rule with a fresh id.
    pub fn new(
        name: impl Into<String>,
        kind: PatternKind,
        pattern: impl Into<String>,
        destination: impl Into<PathBuf>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            enabled: true,
            kind,
            pattern: pattern.into(),
            case_sensitive: false,
            destination: destination.into(),
        }
    }

    /// Checks that the rule is well formed and its pattern compiles.
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::InvalidRule("the name is empty".into()));
        }
        if self.pattern.is_empty() {
            return Err(Error::InvalidPattern {
                rule: self.name.clone(),
                message: "the pattern is empty".into(),
            });
        }
        if !self.destination.is_absolute() {
            return Err(Error::InvalidRule(format!(
                "the destination of \"{}\" must be an absolute path",
                self.name
            )));
        }
        Matcher::compile(self).map(|_| ())
    }
}

/// A compiled rule pattern.
#[derive(Debug, Clone)]
enum Matcher {
    Glob(globset::GlobMatcher),
    Regex(regex::Regex),
}

impl Matcher {
    fn compile(rule: &Rule) -> Result<Self> {
        let invalid = |message: String| Error::InvalidPattern {
            rule: rule.name.clone(),
            message,
        };
        match rule.kind {
            PatternKind::Glob => globset::GlobBuilder::new(&rule.pattern)
                .case_insensitive(!rule.case_sensitive)
                .literal_separator(true)
                .build()
                .map(|g| Matcher::Glob(g.compile_matcher()))
                .map_err(|e| invalid(e.kind().to_string())),
            PatternKind::Regex => regex::RegexBuilder::new(&rule.pattern)
                .case_insensitive(!rule.case_sensitive)
                .build()
                .map(Matcher::Regex)
                .map_err(|e| invalid(e.to_string())),
        }
    }

    fn is_match(&self, file_name: &str) -> bool {
        match self {
            Matcher::Glob(g) => g.is_match(file_name),
            Matcher::Regex(r) => r.is_match(file_name),
        }
    }
}

/// The ordered list of rules, as stored in `rules.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleSet {
    pub version: u32,
    pub rules: Vec<Rule>,
}

impl Default for RuleSet {
    fn default() -> Self {
        Self {
            version: RULES_FILE_VERSION,
            rules: Vec::new(),
        }
    }
}

impl RuleSet {
    pub fn new(rules: Vec<Rule>) -> Self {
        Self {
            version: RULES_FILE_VERSION,
            rules,
        }
    }

    /// Parses a rule set from JSON (used for `rules.json` and imports).
    pub fn from_json(json: &str, origin: &Path) -> Result<Self> {
        let set: RuleSet = serde_json::from_str(json).map_err(|source| Error::Json {
            path: origin.to_path_buf(),
            source,
        })?;
        if set.version > RULES_FILE_VERSION {
            return Err(Error::UnsupportedVersion {
                path: origin.to_path_buf(),
                found: set.version,
                supported: RULES_FILE_VERSION,
            });
        }
        Ok(set)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("rules are always serializable")
    }

    /// Loads the rules from `path`. When the file does not exist, the default
    /// rules are written to it and returned.
    pub fn load_or_init(path: &Path, lang: Lang) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(json) => Self::from_json(&json, path),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let set = Self::new(default_rules(lang)?);
                set.save(path)?;
                Ok(set)
            }
            Err(e) => Err(Error::io(path, e)),
        }
    }

    /// Reads a rule set from an arbitrary file (import).
    pub fn import(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
        let set = Self::from_json(&json, path)?;
        for rule in &set.rules {
            rule.validate()?;
        }
        Ok(set)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        write_atomic(path, self.to_json().as_bytes())
    }

    /// Finds a rule by id, or by 1-based position when `key` is a number.
    pub fn find_index(&self, key: &str) -> Result<usize> {
        if let Ok(pos) = key.parse::<usize>() {
            if (1..=self.rules.len()).contains(&pos) {
                return Ok(pos - 1);
            }
        } else if let Some(i) = self.rules.iter().position(|r| r.id.to_string() == key) {
            return Ok(i);
        }
        Err(Error::RuleNotFound(key.to_string()))
    }

    /// Moves the rule at `from` to position `to` (both 0-based).
    pub fn move_rule(&mut self, from: usize, to: usize) -> Result<()> {
        if from >= self.rules.len() || to >= self.rules.len() {
            return Err(Error::RuleNotFound((from.max(to) + 1).to_string()));
        }
        let rule = self.rules.remove(from);
        self.rules.insert(to, rule);
        Ok(())
    }

    /// Compiles the enabled rules for matching. Fails on the first invalid pattern.
    pub fn compile(&self) -> Result<CompiledRules> {
        let rules = self
            .rules
            .iter()
            .filter(|r| r.enabled)
            .map(|r| Ok((r.clone(), Matcher::compile(r)?)))
            .collect::<Result<_>>()?;
        Ok(CompiledRules { rules })
    }
}

/// Enabled rules ready for matching.
#[derive(Debug, Clone)]
pub struct CompiledRules {
    rules: Vec<(Rule, Matcher)>,
}

impl CompiledRules {
    /// Returns the first rule matching `file_name`.
    pub fn first_match(&self, file_name: &str) -> Option<&Rule> {
        self.rules
            .iter()
            .find(|(_, m)| m.is_match(file_name))
            .map(|(r, _)| r)
    }
}

/// Builds the default rules, pointing to the standard user folders of the OS.
pub fn default_rules(lang: Lang) -> Result<Vec<Rule>> {
    let dirs = directories::UserDirs::new().ok_or(Error::NoHomeDir)?;
    let home = dirs.home_dir().to_path_buf();
    let or_home = |dir: Option<&Path>, fallback: &str| {
        dir.map(Path::to_path_buf)
            .unwrap_or_else(|| home.join(fallback))
    };
    let documents = or_home(dirs.document_dir(), "Documents");
    let pictures = or_home(dirs.picture_dir(), "Pictures");
    let music = or_home(dirs.audio_dir(), "Music");
    let videos = or_home(dirs.video_dir(), "Videos");
    let downloads = or_home(dirs.download_dir(), "Downloads");
    Ok(default_rules_in(
        lang,
        &StandardDirs {
            documents,
            pictures,
            music,
            videos,
            downloads,
        },
    ))
}

/// Standard user folders used by the default rules.
#[derive(Debug, Clone)]
pub struct StandardDirs {
    pub documents: PathBuf,
    pub pictures: PathBuf,
    pub music: PathBuf,
    pub videos: PathBuf,
    pub downloads: PathBuf,
}

/// Installer extensions for the current OS.
fn installer_pattern() -> &'static str {
    if cfg!(target_os = "macos") {
        "*.{dmg,pkg}"
    } else if cfg!(windows) {
        "*.{exe,msi,msix}"
    } else {
        "*.{deb,rpm,AppImage,flatpak}"
    }
}

/// Builds the default rules for the given folders.
pub fn default_rules_in(lang: Lang, d: &StandardDirs) -> Vec<Rule> {
    let it = lang == Lang::It;
    let name = |en: &str, it_name: &str| if it { it_name } else { en }.to_string();
    let glob = |n: String, p: &str, dest: PathBuf| Rule::new(n, PatternKind::Glob, p, dest);
    vec![
        glob(
            name("Documents", "Documenti"),
            "*.{pdf,doc,docx,odt,rtf,txt,md,pages,epub}",
            d.documents.clone(),
        ),
        glob(
            name("Spreadsheets", "Fogli di calcolo"),
            "*.{xls,xlsx,ods,csv,numbers}",
            d.documents.join(name("Spreadsheets", "Fogli di calcolo")),
        ),
        glob(
            name("Presentations", "Presentazioni"),
            "*.{ppt,pptx,odp,key}",
            d.documents.join(name("Presentations", "Presentazioni")),
        ),
        glob(
            name("Images", "Immagini"),
            "*.{jpg,jpeg,png,gif,bmp,tif,tiff,webp,heic,heif,svg,raw,cr2,nef,arw,dng}",
            d.pictures.clone(),
        ),
        glob(
            name("Audio", "Audio"),
            "*.{mp3,wav,flac,aac,m4a,ogg,opus,aiff,wma}",
            d.music.clone(),
        ),
        glob(
            name("Videos", "Video"),
            "*.{mp4,mov,mkv,avi,webm,m4v,wmv,flv,mpg,mpeg}",
            d.videos.clone(),
        ),
        glob(
            name("Archives", "Archivi"),
            "*.{zip,rar,7z,tar,gz,tgz,bz2,xz,zst}",
            d.downloads.join(name("Archives", "Archivi")),
        ),
        glob(
            name("Installers", "Installer"),
            installer_pattern(),
            d.downloads.join("Installer"),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn abs(p: &str) -> PathBuf {
        std::env::temp_dir().join(p)
    }

    fn rule(kind: PatternKind, pattern: &str) -> Rule {
        Rule::new("r", kind, pattern, abs("dest"))
    }

    fn compiled(rules: Vec<Rule>) -> CompiledRules {
        RuleSet::new(rules).compile().unwrap()
    }

    #[test]
    fn glob_matches_case_insensitive_by_default() {
        let c = compiled(vec![rule(PatternKind::Glob, "*.{jpg,png}")]);
        assert!(c.first_match("a.jpg").is_some());
        assert!(c.first_match("A.JPG").is_some());
        assert!(c.first_match("b.Png").is_some());
        assert!(c.first_match("c.gif").is_none());
        assert!(c.first_match("jpg").is_none());
    }

    #[test]
    fn glob_case_sensitive() {
        let mut r = rule(PatternKind::Glob, "*.jpg");
        r.case_sensitive = true;
        let c = compiled(vec![r]);
        assert!(c.first_match("a.jpg").is_some());
        assert!(c.first_match("a.JPG").is_none());
    }

    #[test]
    fn regex_matching() {
        let c = compiled(vec![rule(PatternKind::Regex, r"^invoice_\d{4}\.pdf$")]);
        assert!(c.first_match("invoice_2024.pdf").is_some());
        assert!(c.first_match("INVOICE_2024.PDF").is_some());
        assert!(c.first_match("invoice_24.pdf").is_none());

        let mut r = rule(PatternKind::Regex, r"^Report");
        r.case_sensitive = true;
        let c = compiled(vec![r]);
        assert!(c.first_match("Report.txt").is_some());
        assert!(c.first_match("report.txt").is_none());
    }

    #[test]
    fn first_match_wins_and_disabled_rules_are_skipped() {
        let mut first = Rule::new("first", PatternKind::Glob, "*.pdf", abs("one"));
        let second = Rule::new("second", PatternKind::Glob, "*", abs("two"));
        let c = compiled(vec![first.clone(), second.clone()]);
        assert_eq!(c.first_match("x.pdf").unwrap().name, "first");
        assert_eq!(c.first_match("x.txt").unwrap().name, "second");

        first.enabled = false;
        let c = compiled(vec![first, second]);
        assert_eq!(c.first_match("x.pdf").unwrap().name, "second");
    }

    #[test]
    fn validation() {
        assert!(rule(PatternKind::Glob, "*.txt").validate().is_ok());
        assert!(matches!(
            rule(PatternKind::Regex, "(").validate(),
            Err(Error::InvalidPattern { .. })
        ));
        assert!(matches!(
            rule(PatternKind::Glob, "{a").validate(),
            Err(Error::InvalidPattern { .. })
        ));
        assert!(rule(PatternKind::Glob, "").validate().is_err());
        let mut r = rule(PatternKind::Glob, "*");
        r.destination = PathBuf::from("relative/dir");
        assert!(matches!(r.validate(), Err(Error::InvalidRule(_))));
        let mut r = rule(PatternKind::Glob, "*");
        r.name = "  ".into();
        assert!(matches!(r.validate(), Err(Error::InvalidRule(_))));
    }

    #[test]
    fn json_roundtrip_and_init() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("rules.json");
        let set = RuleSet::load_or_init(&file, Lang::It).unwrap();
        assert!(file.exists());
        assert!(!set.rules.is_empty());
        assert_eq!(set.rules[0].name, "Documenti");
        let again = RuleSet::load_or_init(&file, Lang::En).unwrap();
        assert_eq!(set, again);
    }

    #[test]
    fn json_uses_camel_case_and_defaults() {
        let json = r#"{"version":1,"rules":[{"id":"6f9619ff-8b86-d011-b42d-00cf4fc964ff",
            "name":"Pdf","enabled":true,"kind":"glob","pattern":"*.pdf","destination":"/tmp/x"}]}"#;
        let set = RuleSet::from_json(json, Path::new("t")).unwrap();
        assert!(!set.rules[0].case_sensitive);
        assert!(set.to_json().contains("\"caseSensitive\": false"));
    }

    #[test]
    fn rejects_newer_versions() {
        let json = r#"{"version":99,"rules":[]}"#;
        assert!(matches!(
            RuleSet::from_json(json, Path::new("t")),
            Err(Error::UnsupportedVersion { found: 99, .. })
        ));
    }

    #[test]
    fn import_validates_rules() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("export.json");
        let mut set = RuleSet::new(vec![rule(PatternKind::Regex, "(")]);
        set.save(&file).unwrap();
        assert!(RuleSet::import(&file).is_err());
        set.rules[0].pattern = "ok".into();
        set.save(&file).unwrap();
        assert_eq!(RuleSet::import(&file).unwrap(), set);
    }

    #[test]
    fn find_and_move() {
        let a = Rule::new("a", PatternKind::Glob, "*", abs("a"));
        let b = Rule::new("b", PatternKind::Glob, "*", abs("b"));
        let c = Rule::new("c", PatternKind::Glob, "*", abs("c"));
        let mut set = RuleSet::new(vec![a.clone(), b, c]);
        assert_eq!(set.find_index("1").unwrap(), 0);
        assert_eq!(set.find_index(&a.id.to_string()).unwrap(), 0);
        assert!(set.find_index("0").is_err());
        assert!(set.find_index("4").is_err());
        assert!(set.find_index("nope").is_err());
        set.move_rule(0, 2).unwrap();
        let names: Vec<_> = set.rules.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["b", "c", "a"]);
        assert!(set.move_rule(0, 3).is_err());
    }

    #[test]
    fn default_rules_are_valid_and_localized() {
        let d = StandardDirs {
            documents: abs("Docs"),
            pictures: abs("Pics"),
            music: abs("Music"),
            videos: abs("Videos"),
            downloads: abs("Downloads"),
        };
        let en = default_rules_in(Lang::En, &d);
        let it = default_rules_in(Lang::It, &d);
        assert_eq!(en.len(), it.len());
        for r in en.iter().chain(&it) {
            r.validate().unwrap();
        }
        assert_eq!(en[0].name, "Documents");
        assert_eq!(it[0].name, "Documenti");
        let c = RuleSet::new(en).compile().unwrap();
        assert_eq!(
            c.first_match("photo.HEIC").unwrap().destination,
            abs("Pics")
        );
        assert_eq!(
            c.first_match("table.xlsx").unwrap().destination,
            abs("Docs").join("Spreadsheets")
        );
        assert!(c.first_match("script.sh").is_none());
    }
}
