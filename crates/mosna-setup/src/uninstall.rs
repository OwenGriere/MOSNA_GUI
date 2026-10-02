//! The uninstall, run on a worker thread while the window shows its progress.
//!
//! It removes what `INSTALLATION.exe` put on the machine: the shortcuts, the
//! launcher, the `mosna-GUI` conda environment and the build; then, as chosen,
//! the prerequisites installed for it and the MOSNA GUI folder itself.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;

use crate::conda;
use crate::install::{self, Event, NoWindow, Reporter};
use crate::launcher;
use crate::prerequisites::{self, Prerequisite};

/// The log kept on disk, for when the window is gone.
pub fn log_file() -> PathBuf {
    std::env::temp_dir().join("mosna-gui-uninstall.log")
}

/// What the user chose in the window.
#[derive(Debug, Clone)]
pub struct Choices {
    /// The MOSNA GUI folder, when the uninstaller was found in one.
    pub root: Option<PathBuf>,
    /// Delete the folder as a whole, not just what the install wrote into it.
    pub remove_folder: bool,
    pub prerequisites: Vec<Prerequisite>,
}

/// A prerequisite present on the machine, and whether the installer is known
/// to have put it there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Found {
    pub prerequisite: Prerequisite,
    pub installed_by_mosna: bool,
}

/// The prerequisites this uninstaller could remove. The build tools only as
/// the stand-alone product, never a Visual Studio that happens to carry them.
pub fn found(root: Option<&Path>) -> Vec<Found> {
    let recorded = root.map(prerequisites::recorded).unwrap_or_default();
    Prerequisite::ALL
        .into_iter()
        .filter(|prerequisite| is_present(*prerequisite))
        .map(|prerequisite| Found {
            prerequisite,
            installed_by_mosna: recorded.contains(&prerequisite),
        })
        .collect()
}

fn is_present(prerequisite: Prerequisite) -> bool {
    match prerequisite {
        Prerequisite::Miniconda => {
            conda::base().is_some_and(|base| conda::uninstaller(&base).is_some())
        }
        Prerequisite::Rust => rustup().is_some(),
        Prerequisite::BuildTools => build_tools_path().is_some(),
    }
}

/// Run the whole uninstall. Never panics the window: the outcome arrives as
/// `Done` or `Failed`.
pub fn run(choices: Choices, sender: Sender<Event>) {
    let mut report = Reporter::logging_to(sender, &log_file());
    let mut failures = Vec::new();

    report.step("Suppression des raccourcis");
    for link in launcher::shortcut_places() {
        if link.is_file() {
            match std::fs::remove_file(&link) {
                Ok(()) => report.line(format!("supprimé : {}", link.display())),
                Err(e) => {
                    failures.push(format!("{} n'a pas pu être supprimé : {e}", link.display()))
                }
            }
        }
    }

    if let Some(base) = conda::base() {
        report.step(format!(
            "Suppression de l'environnement conda « {} »",
            conda::ENV_NAME
        ));
        if conda::remove_environment(&base, &mut report) {
            report.line("L'environnement est supprimé.");
        } else {
            failures.push(format!(
                "L'environnement conda « {} » n'a pas pu être supprimé. Fermez MOSNA GUI s'il est \
                 ouvert, puis lancez « conda env remove -n {} » dans un terminal Anaconda Prompt.",
                conda::ENV_NAME,
                conda::ENV_NAME
            ));
        }
    }

    if let (Some(root), false) = (&choices.root, choices.remove_folder) {
        report.step("Suppression de la compilation et du lanceur");
        for name in ["target", launcher::LAUNCHER, launcher::ICON] {
            let path = root.join(name);
            let removed = if path.is_dir() {
                std::fs::remove_dir_all(&path)
            } else if path.is_file() {
                std::fs::remove_file(&path)
            } else {
                continue;
            };
            match removed {
                Ok(()) => report.line(format!("supprimé : {}", path.display())),
                Err(e) => failures.push(format!(
                    "{} n'a pas pu être supprimé : {e}. Fermez MOSNA GUI s'il est ouvert.",
                    path.display()
                )),
            }
        }
    }

    for prerequisite in &choices.prerequisites {
        report.step(format!("Désinstallation : {}", prerequisite.label()));
        match remove_prerequisite(*prerequisite, &mut report) {
            Ok(()) => {
                if let Some(root) = &choices.root {
                    prerequisites::forget(root, *prerequisite);
                }
            }
            Err(message) => failures.push(message),
        }
    }

    if let (Some(root), true) = (&choices.root, choices.remove_folder) {
        report.step(format!("Suppression du dossier {}", root.display()));
        match std::fs::remove_dir_all(root) {
            Ok(()) => report.line(format!("supprimé : {}", root.display())),
            Err(e) => failures.push(format!(
                "{} n'a pas pu être supprimé entièrement : {e}. Fermez ce qui l'utilise \
                 (explorateur, terminal, MOSNA GUI), puis supprimez-le à la main.",
                root.display()
            )),
        }
    }

    if failures.is_empty() {
        report.send(Event::Done(choices.root.clone().unwrap_or_default()));
    } else {
        report.send(Event::Failed(failures.join("\n")));
    }
}

// ---------------------------------------------------------------------------
// The prerequisites
// ---------------------------------------------------------------------------

fn remove_prerequisite(prerequisite: Prerequisite, report: &mut Reporter) -> Result<(), String> {
    match prerequisite {
        Prerequisite::Miniconda => match conda::base() {
            Some(base) => conda::remove_miniconda(&base, report),
            None => Ok(()),
        },
        Prerequisite::Rust => remove_rust(report),
        Prerequisite::BuildTools => remove_build_tools(report),
    }
}

fn rustup() -> Option<PathBuf> {
    let rustup = install::cargo_path()?.with_file_name(install::exe("rustup"));
    rustup.is_file().then_some(rustup)
}

fn remove_rust(report: &mut Reporter) -> Result<(), String> {
    let Some(rustup) = rustup() else {
        return Ok(());
    };
    let mut command = Command::new(&rustup);
    command.args(["self", "uninstall", "-y"]);
    install::stream(&mut command, report)?;
    if rustup.is_file() {
        Err(
            "Rust n'a pas pu être désinstallé. Fermez tout terminal ou éditeur qui l'utilise, \
             puis lancez « rustup self uninstall » dans un terminal."
                .into(),
        )
    } else {
        Ok(())
    }
}

/// Where the stand-alone build tools are installed, if they are.
fn build_tools_path() -> Option<PathBuf> {
    let vswhere =
        install::program_files_x86().join(r"Microsoft Visual Studio\Installer\vswhere.exe");
    let output = Command::new(vswhere)
        .args(["-products", "Microsoft.VisualStudio.Product.BuildTools"])
        .args(["-property", "installationPath"])
        .stderr(Stdio::null())
        .no_window()
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let path = text.lines().next()?.trim();
    (output.status.success() && !path.is_empty()).then(|| PathBuf::from(path))
}

fn remove_build_tools(report: &mut Reporter) -> Result<(), String> {
    report.line("Windows va demander une autorisation administrateur.");
    if !install::winget_uninstall(report, install::BUILD_TOOLS_PACKAGE)
        || build_tools_path().is_some()
    {
        if let Some(path) = build_tools_path() {
            let setup =
                install::program_files_x86().join(r"Microsoft Visual Studio\Installer\setup.exe");
            // As for installing, elevation is only asked for through the shell.
            let mut command = Command::new("cmd");
            command.raw_args(&format!(
                "/c start \"\" /wait \"{}\" uninstall --installPath \"{}\" --passive --norestart",
                setup.display(),
                path.display()
            ));
            install::stream(&mut command, report)?;
        }
    }
    if build_tools_path().is_none() {
        report.line(
            "Le « Visual Studio Installer » lui-même reste : il se désinstalle depuis \
             Paramètres → Applications s'il ne sert plus.",
        );
        Ok(())
    } else {
        Err(
            "Les outils C++ de Microsoft n'ont pas pu être désinstallés. Désinstallez « Build Tools \
             for Visual Studio » depuis Paramètres → Applications."
                .into(),
        )
    }
}
