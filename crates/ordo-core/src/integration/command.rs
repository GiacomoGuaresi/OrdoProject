//! Makes the `ordo` command available in the terminal.
//!
//! - macOS: symlink in `/usr/local/bin` (asking for the administrator
//!   password if needed), otherwise in `~/.local/bin`.
//! - Linux: symlink in `~/.local/bin` (the `.deb` package already installs
//!   `/usr/bin/ordo`).
//! - Windows: the folder of `ordo.exe` is added to the user `Path`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// File name of the CLI executable.
pub const CLI_NAME: &str = if cfg!(windows) { "ordo.exe" } else { "ordo" };

/// State of the `ordo` command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct CommandStatus {
    pub installed: bool,
    /// Where the command was installed (link or folder added to `PATH`).
    pub location: Option<PathBuf>,
    /// The location is in the `PATH` of new terminals.
    pub on_path: bool,
    /// Installed by a system package: Ordo cannot remove it.
    pub managed: bool,
}

/// The CLI shipped next to the app executable.
pub fn cli_next_to(app_exe: &Path) -> PathBuf {
    app_exe.with_file_name(CLI_NAME)
}

fn path_dirs() -> Vec<PathBuf> {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default()
}

/// Folders where a link may be created, in order of preference.
#[cfg(unix)]
pub fn link_dirs(home: &Path) -> Vec<PathBuf> {
    let local = home.join(".local/bin");
    if cfg!(target_os = "macos") {
        vec![PathBuf::from("/usr/local/bin"), local]
    } else {
        vec![local]
    }
}

#[cfg(unix)]
fn points_to_ordo(link: &Path) -> bool {
    std::fs::read_link(link).is_ok_and(|target| target.file_name().is_some_and(|n| n == CLI_NAME))
}

/// Status, looking for links in `dirs`.
#[cfg(unix)]
pub fn status_in(dirs: &[PathBuf]) -> CommandStatus {
    let system = Path::new("/usr/bin").join(CLI_NAME);
    if cfg!(target_os = "linux") && system.is_file() && !system.is_symlink() {
        return CommandStatus {
            installed: true,
            location: Some(system),
            on_path: true,
            managed: true,
        };
    }
    let path = path_dirs();
    for dir in dirs {
        let link = dir.join(CLI_NAME);
        if link.is_symlink() && points_to_ordo(&link) {
            return CommandStatus {
                installed: true,
                on_path: path.iter().any(|p| p == dir),
                location: Some(link),
                managed: false,
            };
        }
    }
    CommandStatus {
        installed: false,
        location: None,
        on_path: false,
        managed: false,
    }
}

#[cfg(unix)]
fn link_in(dir: &Path, cli_exe: &Path) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let link = dir.join(CLI_NAME);
    if link.is_symlink() {
        std::fs::remove_file(&link)?;
    } else if link.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("{} exists and is not a link", link.display()),
        ));
    }
    std::os::unix::fs::symlink(cli_exe, &link)?;
    Ok(link)
}

/// Runs a shell command as administrator through the standard macOS prompt.
#[cfg(target_os = "macos")]
fn run_as_admin(shell: &str) -> bool {
    let escaped = shell.replace('\\', "\\\\").replace('"', "\\\"");
    std::process::Command::new("osascript")
        .arg("-e")
        .arg(format!(
            "do shell script \"{escaped}\" with administrator privileges"
        ))
        .status()
        .is_ok_and(|s| s.success())
}

/// Installs a link to `cli_exe` in the first usable folder of `dirs`.
#[cfg(unix)]
pub fn install_in(cli_exe: &Path, dirs: &[PathBuf], allow_admin: bool) -> Result<CommandStatus> {
    if !cli_exe.is_file() {
        return Err(Error::Integration(format!(
            "the command {} does not exist",
            cli_exe.display()
        )));
    }
    let mut last_err = None;
    for dir in dirs {
        match link_in(dir, cli_exe) {
            Ok(_) => return Ok(status_in(dirs)),
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied && allow_admin => {
                #[cfg(target_os = "macos")]
                {
                    let link = dir.join(CLI_NAME);
                    let cmd = format!(
                        "mkdir -p {} && ln -sf {} {}",
                        super::shell_quote(dir),
                        super::shell_quote(cli_exe),
                        super::shell_quote(&link)
                    );
                    if run_as_admin(&cmd) {
                        return Ok(status_in(dirs));
                    }
                }
                last_err = Some(Error::io(dir, e));
            }
            Err(e) => last_err = Some(Error::io(dir, e)),
        }
    }
    Err(last_err.unwrap_or_else(|| Error::Integration("no folder available".into())))
}

/// Removes the links created by [`install_in`].
#[cfg(unix)]
pub fn uninstall_in(dirs: &[PathBuf], allow_admin: bool) -> Result<CommandStatus> {
    for dir in dirs {
        let link = dir.join(CLI_NAME);
        if !(link.is_symlink() && points_to_ordo(&link)) {
            continue;
        }
        match std::fs::remove_file(&link) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied && allow_admin => {
                #[cfg(target_os = "macos")]
                if run_as_admin(&format!("rm -f {}", super::shell_quote(&link))) {
                    continue;
                }
                return Err(Error::io(&link, e));
            }
            Err(e) => return Err(Error::io(&link, e)),
        }
    }
    Ok(status_in(dirs))
}

fn home() -> Result<PathBuf> {
    Ok(directories::BaseDirs::new()
        .ok_or(Error::NoHomeDir)?
        .home_dir()
        .to_path_buf())
}

/// Current state of the `ordo` command.
pub fn status() -> Result<CommandStatus> {
    #[cfg(unix)]
    {
        Ok(status_in(&link_dirs(&home()?)))
    }
    #[cfg(windows)]
    {
        let _ = home;
        windows_path::status()
    }
}

/// Installs the `ordo` command pointing at `cli_exe`.
pub fn install(cli_exe: &Path) -> Result<CommandStatus> {
    #[cfg(unix)]
    {
        install_in(cli_exe, &link_dirs(&home()?), true)
    }
    #[cfg(windows)]
    {
        windows_path::install(cli_exe)
    }
}

/// Removes the `ordo` command installed by Ordo.
pub fn uninstall() -> Result<CommandStatus> {
    #[cfg(unix)]
    {
        uninstall_in(&link_dirs(&home()?), true)
    }
    #[cfg(windows)]
    {
        windows_path::uninstall()
    }
}

#[cfg(windows)]
mod windows_path {
    use std::path::{Path, PathBuf};

    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, RegType};
    use winreg::{RegKey, RegValue};

    use super::CommandStatus;
    use crate::error::{Error, Result};

    const ENV_KEY: &str = "Environment";

    fn reg_err(e: std::io::Error) -> Error {
        Error::Integration(format!(r"registry HKCU\{ENV_KEY}: {e}"))
    }

    fn read_path(key: &RegKey) -> Vec<String> {
        key.get_value::<String, _>("Path")
            .unwrap_or_default()
            .split(';')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect()
    }

    fn write_path(key: &RegKey, entries: &[String]) -> Result<()> {
        let value: Vec<u16> = entries
            .join(";")
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let bytes = value.iter().flat_map(|u| u.to_le_bytes()).collect();
        key.set_raw_value(
            "Path",
            &RegValue {
                bytes,
                vtype: RegType::REG_EXPAND_SZ,
            },
        )
        .map_err(reg_err)?;
        broadcast_change();
        Ok(())
    }

    /// Tells Explorer (and new terminals) that the environment changed.
    fn broadcast_change() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
        };
        let name: Vec<u16> = "Environment\0".encode_utf16().collect();
        let mut result = 0usize;
        unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                name.as_ptr() as isize,
                SMTO_ABORTIFHUNG,
                2000,
                &mut result,
            );
        }
    }

    fn same_dir(a: &str, b: &Path) -> bool {
        Path::new(a.trim_end_matches('\\')) == b
            || a.trim_end_matches('\\')
                .eq_ignore_ascii_case(&b.to_string_lossy())
    }

    fn is_ordo_dir(entry: &str) -> bool {
        Path::new(entry).join(super::CLI_NAME).is_file()
    }

    pub fn status() -> Result<CommandStatus> {
        let key = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags(ENV_KEY, KEY_READ)
            .map_err(reg_err)?;
        let found = read_path(&key).into_iter().find(|e| is_ordo_dir(e));
        Ok(CommandStatus {
            installed: found.is_some(),
            on_path: found.is_some(),
            location: found.map(PathBuf::from),
            managed: false,
        })
    }

    pub fn install(cli_exe: &Path) -> Result<CommandStatus> {
        let dir = cli_exe
            .parent()
            .ok_or_else(|| Error::Integration("invalid command path".into()))?;
        let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey_with_flags(ENV_KEY, KEY_READ | KEY_WRITE)
            .map_err(reg_err)?;
        let mut entries = read_path(&key);
        if !entries.iter().any(|e| same_dir(e, dir)) {
            entries.push(dir.to_string_lossy().into_owned());
            write_path(&key, &entries)?;
        }
        Ok(CommandStatus {
            installed: true,
            location: Some(dir.to_path_buf()),
            on_path: true,
            managed: false,
        })
    }

    pub fn uninstall() -> Result<CommandStatus> {
        let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey_with_flags(ENV_KEY, KEY_READ | KEY_WRITE)
            .map_err(reg_err)?;
        let entries = read_path(&key);
        let kept: Vec<String> = entries
            .iter()
            .filter(|e| !is_ordo_dir(e))
            .cloned()
            .collect();
        if kept.len() != entries.len() {
            write_path(&key, &kept)?;
        }
        Ok(CommandStatus {
            installed: false,
            location: None,
            on_path: false,
            managed: false,
        })
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn link_lifecycle() {
        let tmp = tempfile::tempdir().unwrap();
        let cli = tmp.path().join("app/ordo");
        std::fs::create_dir_all(cli.parent().unwrap()).unwrap();
        std::fs::write(&cli, "").unwrap();
        let blocked = tmp.path().join("blocked");
        let local = tmp.path().join("home/.local/bin");
        // A regular file named `ordo` is never replaced: falls back to the next folder.
        std::fs::create_dir_all(&blocked).unwrap();
        std::fs::write(blocked.join("ordo"), "other program").unwrap();
        let dirs = vec![blocked.clone(), local.clone()];

        assert!(!status_in(&dirs).installed);
        let s = install_in(&cli, &dirs, false).unwrap();
        assert!(s.installed);
        assert_eq!(s.location, Some(local.join("ordo")));
        assert_eq!(std::fs::read_link(local.join("ordo")).unwrap(), cli);
        assert_eq!(
            std::fs::read_to_string(blocked.join("ordo")).unwrap(),
            "other program"
        );

        // Reinstalling replaces the link.
        install_in(&cli, &dirs, false).unwrap();
        let s = uninstall_in(&dirs, false).unwrap();
        assert!(!s.installed);
        assert!(!local.join("ordo").exists());
        assert!(blocked.join("ordo").exists());
    }

    #[test]
    fn missing_cli_is_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(install_in(&tmp.path().join("nope"), &[tmp.path().to_path_buf()], false).is_err());
    }

    #[test]
    fn cli_is_next_to_app() {
        assert_eq!(
            cli_next_to(Path::new("/Applications/Ordo.app/Contents/MacOS/Ordo")),
            PathBuf::from("/Applications/Ordo.app/Contents/MacOS/ordo")
        );
    }
}
