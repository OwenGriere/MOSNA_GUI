//! How MOSNA GUI is started once installed: `MosnaGUI.bat`, and the shortcuts
//! pointing at it.
//!
//! The interface finds its Python sources from its own location, but not the
//! interpreter the analyses need: that is the conda environment's, and conda
//! wants it *activated* — on Windows activation is what puts the environment's
//! DLL folders on the PATH. So the shortcuts start a launcher that activates
//! it, as `MosnaGUI.sh` does on Linux, rather than the interface itself. It
//! runs minimised: the console behind it has nothing to say.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::conda;
use crate::install::{self, NoWindow, Reporter};

/// The launcher, at the root of the folder.
pub const LAUNCHER: &str = "MosnaGUI.bat";
/// The icon the shortcuts show — a real `.ico`, written from the logo, which
/// is a PNG whatever its extension says, and which Windows will not show.
pub const ICON: &str = "MosnaGUI.ico";
/// The shortcuts' name, the one `setup_windows.bat` gave its own.
pub const SHORTCUT: &str = "MOSNA GUI.lnk";

/// The logo, built into the program.
const LOGO: &[u8] = include_bytes!("../../../assets/logo.ico");

/// Write the launcher and the icon, then the shortcuts.
pub fn install(
    root: &Path,
    base: &Path,
    python: &Path,
    desktop_shortcut: bool,
    report: &mut Reporter,
) -> Result<(), String> {
    let launcher = root.join(LAUNCHER);
    std::fs::write(&launcher, batch(root, base, python))
        .map_err(|e| format!("{} n'a pas pu être écrit : {e}", launcher.display()))?;
    report.line(format!("Lanceur : {}", launcher.display()));

    let icon = root.join(ICON);
    if let Err(e) = write_icon(&icon) {
        report.line(format!(
            "warning: l'icône n'a pas pu être écrite ({e}) ; raccourcis sans icône."
        ));
    }

    let mut places = vec![start_menu()];
    if desktop_shortcut {
        places.push(install::desktop_folder());
    }
    for directory in places.into_iter().flatten() {
        let link = directory.join(SHORTCUT);
        if write_shortcut(&link, &launcher, root, &icon) {
            report.line(format!("Raccourci : {}", link.display()));
        } else {
            return Err(format!(
                "Le raccourci {} n'a pas pu être créé.",
                link.display()
            ));
        }
    }
    Ok(())
}

/// The launcher's text.
///
/// `chcp 65001` first, so the paths below — written as UTF-8 — survive an
/// accented user name; then the environment, then the interface, told where
/// its sources and its interpreter are rather than left to guess.
pub fn batch(root: &Path, base: &Path, python: &Path) -> String {
    let interface = root.join(r"target\release\mosna-gui.exe");
    [
        "@echo off".to_string(),
        "rem Written by INSTALLATION.exe; written again by each install.".to_string(),
        "chcp 65001 >nul".to_string(),
        format!(
            "call \"{}\" activate \"{}\"",
            conda::conda_bat(base).display(),
            conda::ENV_NAME
        ),
        format!("set \"MOSNA_GUI_ROOT={}\"", root.display()),
        format!("set \"MOSNA_PYTHON={}\"", python.display()),
        format!("cd /d \"{}\"", root.display()),
        // Run in this console, not `start`ed: the interface is a console
        // program, and a `start` would open it a console of its own, in plain
        // view, where this one is minimised.
        format!("\"{}\" %*", interface.display()),
        String::new(),
    ]
    .join("\r\n")
}

/// The logo as a multi-size icon.
fn write_icon(path: &Path) -> Result<(), String> {
    use image::codecs::ico::{IcoEncoder, IcoFrame};
    let logo = image::load_from_memory(LOGO).map_err(|e| e.to_string())?;
    let frames = [256u32, 64, 48, 32, 16]
        .into_iter()
        .map(|size| {
            let pixels = logo
                .resize_exact(size, size, image::imageops::FilterType::Lanczos3)
                .into_rgba8();
            IcoFrame::as_png(pixels.as_raw(), size, size, image::ExtendedColorType::Rgba8)
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    IcoEncoder::new(file)
        .encode_images(&frames)
        .map_err(|e| e.to_string())
}

/// The user's Start Menu programs folder.
pub fn start_menu() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .map(|roaming| PathBuf::from(roaming).join(r"Microsoft\Windows\Start Menu\Programs"))
}

/// Every place a shortcut may have been put, by this installer or by
/// `setup_windows.bat` before it, which wrote to the profile's own Desktop.
pub fn shortcut_places() -> Vec<PathBuf> {
    let mut places: Vec<PathBuf> = [start_menu(), install::desktop_folder()]
        .into_iter()
        .flatten()
        .collect();
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        places.push(PathBuf::from(profile).join("Desktop"));
    }
    places
        .into_iter()
        .map(|place| place.join(SHORTCUT))
        .collect()
}

/// Have Windows write the shortcut, through `WScript.Shell`. The paths travel
/// in environment variables, so no quoting rule can mangle them.
fn write_shortcut(link: &Path, launcher: &Path, root: &Path, icon: &Path) -> bool {
    const SCRIPT: &str =
        "$link = (New-Object -ComObject WScript.Shell).CreateShortcut($env:MOSNA_LINK); \
        $link.TargetPath = $env:MOSNA_LINK_TARGET; \
        $link.WorkingDirectory = $env:MOSNA_LINK_DIRECTORY; \
        $link.Description = 'MOSNA GUI'; \
        $link.WindowStyle = 7; \
        if (Test-Path $env:MOSNA_LINK_ICON) { $link.IconLocation = $env:MOSNA_LINK_ICON + ',0' }; \
        $link.Save()";
    if let Some(parent) = link.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::remove_file(link);
    Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .env("MOSNA_LINK", link)
        .env("MOSNA_LINK_TARGET", launcher)
        .env("MOSNA_LINK_DIRECTORY", root)
        .env("MOSNA_LINK_ICON", icon)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .no_window()
        .status()
        .is_ok_and(|status| status.success())
        && link.is_file()
}

/// Start MOSNA GUI as the shortcuts do, its console minimised.
pub fn launch(root: &Path) {
    let launcher = root.join(LAUNCHER);
    let _ = Command::new("cmd")
        .raw_args(&format!("/c start \"\" /min \"{}\"", launcher.display()))
        .no_window()
        .spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_launcher_activates_the_environment_then_starts_the_interface() {
        let text = batch(
            Path::new(r"C:\Users\Me\MOSNA_GUI"),
            Path::new(r"C:\Users\Me\miniconda3"),
            Path::new(r"C:\Users\Me\miniconda3\envs\mosna-GUI\python.exe"),
        );
        let lines: Vec<&str> = text.lines().collect();
        let activate = lines
            .iter()
            .position(|l| l.contains("activate \"mosna-GUI\""))
            .unwrap();
        let start = lines
            .iter()
            .position(|l| l.contains("mosna-gui.exe"))
            .unwrap();
        assert!(
            activate < start,
            "the interface starts before the environment is active"
        );
        assert!(
            text.contains(r#"set "MOSNA_PYTHON=C:\Users\Me\miniconda3\envs\mosna-GUI\python.exe""#)
        );
        assert!(text.contains(r#"set "MOSNA_GUI_ROOT=C:\Users\Me\MOSNA_GUI""#));
        assert!(text.contains("\r\n"), "a batch file wants CRLF");
    }

    #[test]
    fn the_icon_is_a_real_icon_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(ICON);
        write_icon(&path).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        // ICONDIR: reserved 0, type 1 (icon), then the number of images.
        assert_eq!(&bytes[..4], &[0, 0, 1, 0]);
        assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), 5);
    }
}
