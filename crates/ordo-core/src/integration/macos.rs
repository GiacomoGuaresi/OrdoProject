//! macOS: a Quick Action (Automator "service") in `~/Library/Services`.
//!
//! It appears in Finder's context menu under "Quick Actions" for files and
//! folders and runs a shell script that starts the app with `--sort`.

use std::path::{Path, PathBuf};

use super::{IntegrationItem, SORT_FLAG, menu_label, remove_path, shell_quote, write_file};
use crate::error::Result;
use crate::lang::Lang;

/// File name of the workflow bundle (independent of the language, so that
/// it can always be found and removed).
pub const WORKFLOW_NAME: &str = "Ordo.workflow";

pub fn workflow_path(home: &Path) -> PathBuf {
    home.join("Library/Services").join(WORKFLOW_NAME)
}

pub fn status(home: &Path) -> IntegrationItem {
    let path = workflow_path(home);
    IntegrationItem {
        id: "macos-quick-action".into(),
        name: "Finder".into(),
        available: true,
        installed: path.join("Contents/document.wflow").is_file(),
        location: path.display().to_string(),
    }
}

/// Shell script run by the Quick Action. The app is detached so that
/// Finder's progress indicator stops immediately.
pub fn script(app_exe: &Path) -> String {
    format!(
        "{exe} {SORT_FLAG} \"$@\" >/dev/null 2>&1 &",
        exe = shell_quote(app_exe)
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn info_plist(label: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>NSServices</key>
	<array>
		<dict>
			<key>NSBackgroundColorName</key>
			<string>background</string>
			<key>NSIconName</key>
			<string>NSActionTemplate</string>
			<key>NSMenuItem</key>
			<dict>
				<key>default</key>
				<string>{label}</string>
			</dict>
			<key>NSMessage</key>
			<string>runWorkflowAsService</string>
			<key>NSRequiredContext</key>
			<dict>
				<key>NSApplicationIdentifier</key>
				<string>com.apple.finder</string>
			</dict>
			<key>NSSendFileTypes</key>
			<array>
				<string>public.item</string>
			</array>
		</dict>
	</array>
</dict>
</plist>
"#,
        label = xml_escape(label)
    )
}

fn document_wflow(script: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>AMApplicationBuild</key>
	<string>523</string>
	<key>AMApplicationVersion</key>
	<string>2.10</string>
	<key>AMDocumentVersion</key>
	<string>2</string>
	<key>actions</key>
	<array>
		<dict>
			<key>action</key>
			<dict>
				<key>AMAccepts</key>
				<dict>
					<key>Container</key>
					<string>List</string>
					<key>Optional</key>
					<true/>
					<key>Types</key>
					<array>
						<string>com.apple.cocoa.string</string>
					</array>
				</dict>
				<key>AMActionVersion</key>
				<string>2.0.3</string>
				<key>AMApplication</key>
				<array>
					<string>Automator</string>
				</array>
				<key>AMParameterProperties</key>
				<dict>
					<key>COMMAND_STRING</key>
					<dict/>
					<key>CheckedForUserDefaultShell</key>
					<dict/>
					<key>inputMethod</key>
					<dict/>
					<key>shell</key>
					<dict/>
					<key>source</key>
					<dict/>
				</dict>
				<key>AMProvides</key>
				<dict>
					<key>Container</key>
					<string>List</string>
					<key>Types</key>
					<array>
						<string>com.apple.cocoa.string</string>
					</array>
				</dict>
				<key>ActionBundlePath</key>
				<string>/System/Library/Automator/Run Shell Script.action</string>
				<key>ActionName</key>
				<string>Run Shell Script</string>
				<key>ActionParameters</key>
				<dict>
					<key>COMMAND_STRING</key>
					<string>{script}</string>
					<key>CheckedForUserDefaultShell</key>
					<true/>
					<key>inputMethod</key>
					<integer>1</integer>
					<key>shell</key>
					<string>/bin/bash</string>
					<key>source</key>
					<string></string>
				</dict>
				<key>BundleIdentifier</key>
				<string>com.apple.RunShellScript</string>
				<key>CFBundleVersion</key>
				<string>2.0.3</string>
				<key>CanShowSelectedItemsWhenRun</key>
				<false/>
				<key>CanShowWhenRun</key>
				<true/>
				<key>Category</key>
				<array>
					<string>AMCategoryUtilities</string>
				</array>
				<key>Class Name</key>
				<string>RunShellScriptAction</string>
				<key>InputUUID</key>
				<string>4B6C1E43-6D2E-4E0C-9A0B-0A9C5B2F7E11</string>
				<key>Keywords</key>
				<array>
					<string>Shell</string>
					<string>Script</string>
				</array>
				<key>OutputUUID</key>
				<string>8E1D7C1A-0F55-4C39-8D7B-6B6B2F0E4C22</string>
				<key>UUID</key>
				<string>2F6A9C3B-5E1D-4B8A-9C7E-3D2B1A0F9E33</string>
				<key>UnlocalizedApplications</key>
				<array>
					<string>Automator</string>
				</array>
				<key>arguments</key>
				<dict/>
				<key>isViewVisible</key>
				<integer>1</integer>
				<key>location</key>
				<string>309.000000:253.000000</string>
				<key>nibPath</key>
				<string>/System/Library/Automator/Run Shell Script.action/Contents/Resources/Base.lproj/main.nib</string>
			</dict>
			<key>isViewVisible</key>
			<integer>1</integer>
		</dict>
	</array>
	<key>connectors</key>
	<dict/>
	<key>workflowMetaData</key>
	<dict>
		<key>applicationBundleIDsByPath</key>
		<dict/>
		<key>applicationPaths</key>
		<array/>
		<key>inputTypeIdentifier</key>
		<string>com.apple.Automator.fileSystemObject</string>
		<key>outputTypeIdentifier</key>
		<string>com.apple.Automator.nothing</string>
		<key>presentationMode</key>
		<integer>15</integer>
		<key>processesInput</key>
		<false/>
		<key>serviceInputTypeIdentifier</key>
		<string>com.apple.Automator.fileSystemObject</string>
		<key>serviceOutputTypeIdentifier</key>
		<string>com.apple.Automator.nothing</string>
		<key>serviceProcessesInput</key>
		<false/>
		<key>systemImageName</key>
		<string>NSActionTemplate</string>
		<key>useAutomaticInputType</key>
		<false/>
		<key>workflowTypeIdentifier</key>
		<string>com.apple.Automator.servicesMenu</string>
	</dict>
</dict>
</plist>
"#,
        script = xml_escape(script)
    )
}

/// Writes the Quick Action.
pub fn install(home: &Path, app_exe: &Path, lang: Lang) -> Result<()> {
    let path = workflow_path(home);
    remove_path(&path)?;
    write_file(
        &path.join("Contents/Info.plist"),
        &info_plist(menu_label(lang)),
        false,
    )?;
    write_file(
        &path.join("Contents/document.wflow"),
        &document_wflow(&script(app_exe)),
        false,
    )?;
    refresh_services();
    Ok(())
}

pub fn uninstall(home: &Path) -> Result<()> {
    remove_path(&workflow_path(home))?;
    refresh_services();
    Ok(())
}

/// Asks macOS to rebuild the services menu so the change is visible at once.
fn refresh_services() {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("/System/Library/CoreServices/pbs")
            .arg("-update")
            .status();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_quotes_and_detaches() {
        let s = script(Path::new("/Applications/Ordo.app/Contents/MacOS/Ordo"));
        assert_eq!(
            s,
            "'/Applications/Ordo.app/Contents/MacOS/Ordo' --sort \"$@\" >/dev/null 2>&1 &"
        );
    }

    #[test]
    fn install_and_uninstall_in_home() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        assert!(!status(home).installed);
        install(home, Path::new("/Apps/Ordo & co/Ordo"), Lang::It).unwrap();
        assert!(status(home).installed);
        let info =
            std::fs::read_to_string(workflow_path(home).join("Contents/Info.plist")).unwrap();
        assert!(info.contains("<string>Ordina con Ordo</string>"));
        let doc =
            std::fs::read_to_string(workflow_path(home).join("Contents/document.wflow")).unwrap();
        assert!(doc.contains("&apos;") || doc.contains("'/Apps/Ordo &amp; co/Ordo' --sort"));
        uninstall(home).unwrap();
        assert!(!status(home).installed);
        assert!(!workflow_path(home).exists());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn generated_plists_are_valid() {
        let tmp = tempfile::tempdir().unwrap();
        install(tmp.path(), Path::new("/usr/bin/true"), Lang::En).unwrap();
        for f in ["Info.plist", "document.wflow"] {
            let ok = std::process::Command::new("plutil")
                .arg("-lint")
                .arg(workflow_path(tmp.path()).join("Contents").join(f))
                .status()
                .unwrap()
                .success();
            assert!(ok, "{f} is not a valid plist");
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
mod automator_tests {
    use super::*;

    /// Runs the generated workflow with Automator, if `ORDO_TEST_AUTOMATOR` is set.
    #[test]
    fn workflow_runs_in_automator() {
        let Some(dir) = std::env::var_os("ORDO_TEST_AUTOMATOR") else {
            return;
        };
        let dir = PathBuf::from(dir);
        let fake = dir.join("fake app");
        install(&dir, &fake, Lang::En).unwrap();
        let file = dir.join("input file.txt");
        std::fs::write(&file, "").unwrap();
        let status = std::process::Command::new("automator")
            .arg("-i")
            .arg(&file)
            .arg(workflow_path(&dir))
            .status()
            .unwrap();
        assert!(status.success());
        std::thread::sleep(std::time::Duration::from_millis(500));
        let args = std::fs::read_to_string(dir.join("args.txt")).unwrap();
        assert_eq!(args, format!("--sort\n{}\n", file.display()));
    }
}
