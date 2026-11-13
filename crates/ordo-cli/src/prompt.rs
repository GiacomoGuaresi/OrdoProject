//! Interactive conflict prompt on the terminal.

use std::io::{BufRead, Write};
use std::path::Path;

use chrono::{DateTime, Local};
use ordo_core::fsops::files_identical;
use ordo_core::{Conflict, ConflictResolver, Decision, Resolution};

use crate::i18n::Tr;

/// Asks the user on stderr/stdin how to resolve each conflict.
pub struct TtyResolver {
    pub tr: Tr,
}

/// Parses an answer: lowercase applies to this conflict, uppercase to all.
pub fn parse_answer(answer: &str) -> Option<Decision> {
    let c = answer.trim().chars().next()?;
    let resolution = match c.to_ascii_lowercase() {
        'o' => Resolution::Overwrite,
        'k' => Resolution::KeepBoth,
        's' => Resolution::Skip,
        _ => return None,
    };
    Some(Decision {
        resolution,
        apply_to_all: c.is_ascii_uppercase(),
    })
}

pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

fn describe(path: &Path) -> (String, String) {
    match std::fs::metadata(path) {
        Ok(m) => {
            let date = m
                .modified()
                .map(|t| {
                    DateTime::<Local>::from(t)
                        .format("%Y-%m-%d %H:%M")
                        .to_string()
                })
                .unwrap_or_default();
            (human_size(m.len()), date)
        }
        Err(_) => ("?".into(), "?".into()),
    }
}

impl ConflictResolver for TtyResolver {
    fn resolve(&mut self, conflict: &Conflict) -> Decision {
        let tr = self.tr;
        let name = conflict
            .existing
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let dir = conflict
            .existing
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let title = if conflict.is_undo {
            "conflict.title_undo"
        } else {
            "conflict.title"
        };
        let mut err = std::io::stderr().lock();
        let _ = writeln!(err, "\n{}", tr.t(title, &[&name, &dir]));
        if files_identical(&conflict.existing, &conflict.incoming).unwrap_or(false) {
            let _ = writeln!(err, "  {}", tr.t("conflict.identical", &[]));
        } else {
            let (es, ed) = describe(&conflict.existing);
            let (is, id) = describe(&conflict.incoming);
            let _ = writeln!(
                err,
                "  {}",
                tr.t("conflict.different", &[&es, &ed, &is, &id])
            );
        }
        let _ = writeln!(err, "  {}", tr.t("conflict.choices", &[]));

        let stdin = std::io::stdin();
        loop {
            let _ = write!(err, "> ");
            let _ = err.flush();
            let mut line = String::new();
            match stdin.lock().read_line(&mut line) {
                // EOF: fall back to the safe default for this conflict only.
                Ok(0) | Err(_) => {
                    return Decision {
                        resolution: Resolution::KeepBoth,
                        apply_to_all: false,
                    };
                }
                Ok(_) => {}
            }
            if let Some(d) = parse_answer(&line) {
                return d;
            }
            let _ = writeln!(err, "{}", tr.t("conflict.invalid", &[]));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers() {
        let d = parse_answer("o\n").unwrap();
        assert_eq!(d.resolution, Resolution::Overwrite);
        assert!(!d.apply_to_all);
        let d = parse_answer(" K").unwrap();
        assert_eq!(d.resolution, Resolution::KeepBoth);
        assert!(d.apply_to_all);
        assert_eq!(parse_answer("S").unwrap().resolution, Resolution::Skip);
        assert!(parse_answer("x").is_none());
        assert!(parse_answer("").is_none());
    }

    #[test]
    fn sizes() {
        assert_eq!(human_size(12), "12 B");
        assert_eq!(human_size(2048), "2.0 KB");
        assert_eq!(human_size(5 * 1024 * 1024), "5.0 MB");
    }
}
