//! Windows: Explorer context-menu verbs in `HKCU\Software\Classes`.
//!
//! No administrator rights are needed. On Windows 11 the entries appear under
//! "Show more options".

use std::path::Path;

use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

use super::{IntegrationItem, SORT_FLAG, menu_label};
use crate::error::{Error, Result};
use crate::lang::Lang;

/// Root of the per-user file associations.
pub const CLASSES_ROOT: &str = r"Software\Classes";

/// Name of the verb under each `shell` key.
pub const VERB: &str = "Ordo";

/// (class, argument placeholder): files, folders, folder background.
const TARGETS: [(&str, &str); 3] = [
    ("*", "%1"),
    ("Directory", "%1"),
    (r"Directory\Background", "%V"),
];

fn verb_key(root: &str, class: &str) -> String {
    format!(r"{root}\{class}\shell\{VERB}")
}

fn reg_err(key: &str, e: std::io::Error) -> Error {
    Error::Integration(format!(r"registry HKCU\{key}: {e}"))
}

pub fn command_line(app_exe: &Path, placeholder: &str) -> String {
    format!("\"{}\" {SORT_FLAG} \"{placeholder}\"", app_exe.display())
}

pub fn status(root: &str) -> IntegrationItem {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let installed = TARGETS
        .iter()
        .all(|(class, _)| hkcu.open_subkey(verb_key(root, class)).is_ok());
    IntegrationItem {
        id: "windows-explorer".into(),
        name: "Explorer".into(),
        available: true,
        installed,
        location: format!(r"HKCU\{root}\…\shell\{VERB}"),
    }
}

pub fn install(root: &str, app_exe: &Path, lang: Lang) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let icon = format!("\"{}\",0", app_exe.display());
    for (class, placeholder) in TARGETS {
        let path = verb_key(root, class);
        let (key, _) = hkcu.create_subkey(&path).map_err(|e| reg_err(&path, e))?;
        key.set_value("MUIVerb", &menu_label(lang))
            .and_then(|_| key.set_value("Icon", &icon))
            // Keep the entry visible with more than 15 selected items.
            .and_then(|_| key.set_value("MultiSelectModel", &"Player"))
            .map_err(|e| reg_err(&path, e))?;
        let cmd_path = format!(r"{path}\command");
        let (cmd, _) = hkcu
            .create_subkey(&cmd_path)
            .map_err(|e| reg_err(&cmd_path, e))?;
        cmd.set_value("", &command_line(app_exe, placeholder))
            .map_err(|e| reg_err(&cmd_path, e))?;
    }
    Ok(())
}

pub fn uninstall(root: &str) -> Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for (class, _) in TARGETS {
        let path = verb_key(root, class);
        match hkcu.delete_subkey_all(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(reg_err(&path, e)),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A private registry root so tests never touch the real associations.
    const TEST_ROOT: &str = r"Software\OrdoTests\Classes";

    #[test]
    fn command_lines() {
        assert_eq!(
            command_line(Path::new(r"C:\Program Files\Ordo\Ordo.exe"), "%1"),
            r#""C:\Program Files\Ordo\Ordo.exe" --sort "%1""#
        );
    }

    #[test]
    fn install_status_uninstall() {
        let exe = Path::new(r"C:\Ordo\Ordo.exe");
        uninstall(TEST_ROOT).unwrap();
        assert!(!status(TEST_ROOT).installed);
        install(TEST_ROOT, exe, Lang::It).unwrap();
        assert!(status(TEST_ROOT).installed);
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = hkcu.open_subkey(verb_key(TEST_ROOT, "*")).unwrap();
        let label: String = key.get_value("MUIVerb").unwrap();
        assert_eq!(label, "Ordina con Ordo");
        let cmd: String = hkcu
            .open_subkey(format!(
                r"{}\command",
                verb_key(TEST_ROOT, r"Directory\Background")
            ))
            .unwrap()
            .get_value("")
            .unwrap();
        assert!(cmd.ends_with(r#"--sort "%V""#));
        uninstall(TEST_ROOT).unwrap();
        assert!(!status(TEST_ROOT).installed);
        let _ = hkcu.delete_subkey_all(r"Software\OrdoTests");
    }
}
