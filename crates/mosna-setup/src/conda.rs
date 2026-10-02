//! The Python half: Miniconda, and the `mosna-GUI` environment the analyses
//! and the figure renderer run in.
//!
//! The same environment `setup.sh` and the former `setup_windows.bat` build:
//! Python 3.11 — `xy`, which draws the figures, requires it — the scientific
//! stack from conda-forge, then `mosna-package` and the `mosna_xy` renderer
//! from this folder.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::install::{self, NoWindow, Reporter};
use crate::prerequisites::{self, Prerequisite};

/// The environment's name, which the launcher activates.
pub const ENV_NAME: &str = "mosna-GUI";
/// The oldest Python `xy` accepts, as conda writes it and as Python compares.
pub const PYTHON_VERSION: &str = "3.11";
const MINIMUM_PYTHON: &str = "(3, 11)";

const MINICONDA_URL: &str =
    "https://repo.anaconda.com/miniconda/Miniconda3-latest-Windows-x86_64.exe";

/// What the environment is built with: the list `setup_windows.bat` installed.
const PACKAGES: &[&str] = &[
    "pyyaml",
    "pandas",
    "pyarrow",
    "scipy=1.13",
    "scikit-learn",
    "networkx",
    "matplotlib",
    "seaborn",
    "scanpy",
    "tqdm",
    "lifelines",
    "ipykernel",
    "ipywidgets",
    "markdown",
    "pip",
];

/// What has to import once the environment is built.
const IMPORTS: &str = "import yaml, pandas, mosna, tysserand, mosna_xy, xy";

fn profile() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").map(PathBuf::from)
}

/// Where Miniconda goes when there is no conda yet.
pub fn default_base() -> Option<PathBuf> {
    profile().map(|profile| profile.join("miniconda3"))
}

/// The conda installation, wherever the usual installers put one.
pub fn base() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(profile) = profile() {
        for name in [
            "miniconda3",
            "anaconda3",
            "Miniconda3",
            "Anaconda3",
            "miniforge3",
        ] {
            candidates.push(profile.join(name));
        }
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let local = PathBuf::from(local);
        candidates.push(local.join("miniconda3"));
        candidates.push(local.join("anaconda3"));
    }
    if let Some(data) = std::env::var_os("ProgramData") {
        let data = PathBuf::from(data);
        candidates.push(data.join("miniconda3"));
        candidates.push(data.join("anaconda3"));
    }
    // And whatever the PATH knows of.
    if let Some(path) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&path) {
            if directory.join("conda.bat").is_file() {
                if let Some(base) = directory.parent() {
                    candidates.push(base.to_path_buf());
                }
            }
        }
    }
    candidates.into_iter().find(|base| is_conda(base))
}

fn is_conda(base: &Path) -> bool {
    conda_exe(base).is_file() && base.join(r"condabin\conda.bat").is_file()
}

/// conda as a program rather than as a batch file, which cmd would re-parse.
fn conda_exe(base: &Path) -> PathBuf {
    base.join("Scripts").join("conda.exe")
}

/// The batch file the launcher calls to activate the environment.
pub fn conda_bat(base: &Path) -> PathBuf {
    base.join(r"condabin\conda.bat")
}

/// A conda command, answering yes to Anaconda's terms of service: asked
/// interactively, the question hangs a window that shows no prompt. Only
/// conda-forge is used, which has none, but a configured default channel may.
fn conda(base: &Path) -> Command {
    let mut command = Command::new(conda_exe(base));
    command
        .env("CONDA_PLUGINS_AUTO_ACCEPT_TOS", "yes")
        .env("CONDA_ALWAYS_YES", "true")
        .env("PYTHONIOENCODING", "utf-8");
    command
}

fn capture(command: &mut Command) -> Option<String> {
    let output = command
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .no_window()
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Install Miniconda unless a conda is already there; return its base.
pub fn install_miniconda(root: &Path, report: &mut Reporter) -> Result<PathBuf, String> {
    if let Some(base) = base() {
        report.line(format!("conda : déjà installé ({}).", base.display()));
        return Ok(base);
    }
    report.step("Installation de Miniconda");
    let target = default_base().ok_or("USERPROFILE n'est pas défini.")?;
    let installer = std::env::temp_dir().join("Miniconda3-latest-Windows-x86_64.exe");
    install::download(report, MINICONDA_URL, &installer)?;
    let mut command = Command::new(&installer);
    command.args([
        "/InstallationType=JustMe",
        "/RegisterPython=0",
        "/AddToPath=0",
        "/S",
    ]);
    // NSIS wants the folder last and unquoted, spaces and all.
    command.raw_args(&format!("/D={}", target.display()));
    install::stream(&mut command, report)?;
    if !is_conda(&target) {
        return Err(format!(
            "Miniconda n'a pas pu être installé dans {}. Installez-le depuis \
             https://docs.conda.io/en/latest/miniconda.html, puis relancez INSTALLATION.exe.",
            target.display()
        ));
    }
    prerequisites::record(root, Prerequisite::Miniconda);
    report.line(format!("Miniconda : {}", target.display()));
    Ok(target)
}

/// The environment's folder, if it exists, from `conda env list`.
pub fn environment(base: &Path) -> Option<PathBuf> {
    let listing = capture(conda(base).args(["env", "list"]))?;
    environment_in(&listing)
}

/// The folder of [`ENV_NAME`] in `conda env list` output, whose lines read
/// `name [*] path`.
fn environment_in(listing: &str) -> Option<PathBuf> {
    listing.lines().find_map(|line| {
        let rest = line.trim_start().strip_prefix(ENV_NAME)?;
        if !rest.starts_with(char::is_whitespace) {
            return None;
        }
        let path = rest.trim().trim_start_matches('*').trim();
        (!path.is_empty()).then(|| PathBuf::from(path))
    })
}

fn python_of(environment: &Path) -> PathBuf {
    environment.join("python.exe")
}

/// Whether the environment's Python is recent enough for `xy`.
fn is_recent_enough(python: &Path) -> bool {
    let script = format!("import sys; sys.exit(0 if sys.version_info >= {MINIMUM_PYTHON} else 1)");
    Command::new(python)
        .args(["-c", &script])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .no_window()
        .status()
        .is_ok_and(|status| status.success())
}

/// Build or bring up to date the environment, install MOSNA's Python
/// packages into it, and return its interpreter.
pub fn prepare_environment(
    base: &Path,
    root: &Path,
    report: &mut Reporter,
) -> Result<PathBuf, String> {
    report.step(format!("Environnement conda « {ENV_NAME} »"));
    if let Some(existing) = environment(base) {
        if is_recent_enough(&python_of(&existing)) {
            report.line(format!(
                "Déjà présent ({}) : mis à jour.",
                existing.display()
            ));
        } else {
            // A setup.sh of an earlier version built it on Python 3.10.
            report.line(format!(
                "warning: l'environnement existant n'a pas Python {PYTHON_VERSION} ou plus récent ; il est reconstruit."
            ));
            run(
                conda(base).args(["env", "remove", "-n", ENV_NAME, "-y"]),
                report,
                "L'ancien environnement n'a pas pu être supprimé.",
            )?;
        }
    }
    if environment(base).is_none() {
        report.line(format!("Création avec Python {PYTHON_VERSION}…"));
        run(
            conda(base)
                .args([
                    "create",
                    "-n",
                    ENV_NAME,
                    "-y",
                    "--override-channels",
                    "-c",
                    "conda-forge",
                ])
                .arg(format!("python={PYTHON_VERSION}")),
            report,
            "La création de l'environnement conda a échoué.",
        )?;
    }

    report.step("Paquets scientifiques (conda-forge ; la plus longue étape après la compilation)");
    run(
        conda(base)
            .args([
                "install",
                "-n",
                ENV_NAME,
                "-y",
                "--override-channels",
                "-c",
                "conda-forge",
            ])
            .arg(format!("python={PYTHON_VERSION}"))
            .args(PACKAGES),
        report,
        "L'installation des paquets conda a échoué.",
    )?;

    report.step("mosna-package et le moteur de figures mosna_xy");
    run(
        pip(base).arg(root.join("mosna-package")),
        report,
        "L'installation de mosna-package a échoué.",
    )?;
    // Editable, as setup.sh installs it: a change to a figure is picked up
    // without reinstalling.
    run(
        pip(base).arg("-e").arg(root.join("python")),
        report,
        "L'installation de mosna_xy a échoué.",
    )?;

    report.step("Vérification des modules Python");
    run(
        in_environment(base).args(["python", "-c", IMPORTS]),
        report,
        "Des modules Python manquent après l'installation ; le détail est dans le journal ci-dessus.",
    )?;
    let environment =
        environment(base).ok_or("L'environnement conda est introuvable après sa création.")?;
    let python = python_of(&environment);
    report.line(format!("Python des analyses : {}", python.display()));
    Ok(python)
}

/// A command run inside the activated environment, its output streamed.
fn in_environment(base: &Path) -> Command {
    let mut command = conda(base);
    command.args(["run", "-n", ENV_NAME, "--no-capture-output"]);
    command
}

fn pip(base: &Path) -> Command {
    let mut command = in_environment(base);
    command.args(["python", "-m", "pip", "install", "--no-input"]);
    command
}

fn run(command: &mut Command, report: &mut Reporter, failure: &str) -> Result<(), String> {
    if install::stream(command, report)? {
        Ok(())
    } else {
        Err(failure.into())
    }
}

/// Remove the environment; true when it is gone.
pub fn remove_environment(base: &Path, report: &mut Reporter) -> bool {
    if environment(base).is_none() {
        return true;
    }
    let _ = install::stream(
        conda(base).args(["env", "remove", "-n", ENV_NAME, "-y"]),
        report,
    );
    environment(base).is_none()
}

/// The uninstaller Miniconda (or Anaconda) put at the root of its base.
pub fn uninstaller(base: &Path) -> Option<PathBuf> {
    std::fs::read_dir(base)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("Uninstall-") && name.ends_with(".exe"))
        })
}

/// Remove the conda installation at `base`.
pub fn remove_miniconda(base: &Path, report: &mut Reporter) -> Result<(), String> {
    if let Some(uninstaller) = uninstaller(base) {
        let mut command = Command::new(&uninstaller);
        command.arg("/S");
        // Without `_?=`, an NSIS uninstaller copies itself away and returns at
        // once; with it, it runs in place and is waited for — and leaves
        // itself behind, removed just below.
        command.raw_args(&format!("_?={}", base.display()));
        install::stream(&mut command, report)?;
    }
    let _ = std::fs::remove_dir_all(base);
    if is_conda(base) {
        Err(format!(
            "conda n'a pas pu être désinstallé de {}. Désinstallez-le depuis Paramètres → Applications.",
            base.display()
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_environment_is_found_in_the_listing() {
        let listing =
            "# conda environments:\n#\nbase                 * C:\\Users\\Me\\miniconda3\n\
                       mosna-GUI              C:\\Users\\Me\\miniconda3\\envs\\mosna-GUI\n";
        assert_eq!(
            environment_in(listing),
            Some(PathBuf::from(r"C:\Users\Me\miniconda3\envs\mosna-GUI"))
        );
    }

    #[test]
    fn the_active_marker_and_spaces_are_handled() {
        let listing = "mosna-GUI  *  C:\\Users\\Jo Doe\\miniconda3\\envs\\mosna-GUI\n";
        assert_eq!(
            environment_in(listing),
            Some(PathBuf::from(r"C:\Users\Jo Doe\miniconda3\envs\mosna-GUI"))
        );
    }

    #[test]
    fn a_longer_name_is_not_taken_for_it() {
        assert_eq!(environment_in("mosna-GUI-old   C:\\envs\\old\n"), None);
        assert_eq!(environment_in("base   C:\\miniconda3\n"), None);
    }

    #[test]
    fn the_uninstaller_is_found_by_its_name() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(uninstaller(dir.path()), None);
        std::fs::write(dir.path().join("Uninstall-Miniconda3.exe"), "MZ").unwrap();
        assert_eq!(
            uninstaller(dir.path()),
            Some(dir.path().join("Uninstall-Miniconda3.exe"))
        );
    }
}
