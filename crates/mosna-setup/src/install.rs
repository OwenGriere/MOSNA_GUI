//! The install itself, run on a worker thread while the window shows its
//! progress.
//!
//! It does what `setup.sh` does — a conda environment for the Python analyses,
//! then the Rust interface built with cargo, then a launcher and its shortcuts
//! — but first installs whatever a fresh Windows lacks for that: the Microsoft
//! C++ build tools, which Rust links with, Rust itself, and Miniconda.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;

use crate::conda;
use crate::launcher;
use crate::place::{self, Placement};
use crate::prerequisites::{self, Prerequisite};

/// Deleted from the installed folder at the end: they only serve Linux, or
/// are what this program replaces.
pub const LINUX_FILES: &[&str] = &["setup.sh", "MosnaGUI.sh", "setup_windows.bat"];

/// What winget calls the Microsoft C++ build tools.
pub(crate) const BUILD_TOOLS_PACKAGE: &str = "Microsoft.VisualStudio.2022.BuildTools";

/// What the user chose in the window.
#[derive(Debug, Clone)]
pub struct Choices {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub placement: Placement,
    pub desktop_shortcut: bool,
}

/// What the worker tells the window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A new stage begins.
    Step(String),
    /// A line of output.
    Line(String),
    /// Finished: where MOSNA GUI's folder now is.
    Done(PathBuf),
    Failed(String),
}

/// The log kept on disk, for when the window is gone.
pub fn log_file() -> PathBuf {
    std::env::temp_dir().join("mosna-gui-install.log")
}

/// Sends events to the window and copies them into the log file.
pub struct Reporter {
    sender: Sender<Event>,
    log: Option<std::fs::File>,
}

impl Reporter {
    pub fn new(sender: Sender<Event>) -> Self {
        Self::logging_to(sender, &log_file())
    }

    pub fn logging_to(sender: Sender<Event>, log: &Path) -> Self {
        Self {
            sender,
            log: std::fs::File::create(log).ok(),
        }
    }

    pub(crate) fn send(&mut self, event: Event) {
        if let Some(log) = &mut self.log {
            let _ = match &event {
                Event::Step(text) => writeln!(log, "\n==> {text}"),
                Event::Line(text) => writeln!(log, "{}", crate::console::plain(text)),
                Event::Done(root) => writeln!(log, "\nTerminé ({}).", root.display()),
                Event::Failed(text) => writeln!(log, "\nÉCHEC : {text}"),
            };
        }
        let _ = self.sender.send(event);
    }

    pub fn step(&mut self, text: impl Into<String>) {
        self.send(Event::Step(text.into()));
    }

    pub fn line(&mut self, text: impl Into<String>) {
        self.send(Event::Line(text.into()));
    }
}

/// Run the whole install, reporting as it goes. Never panics the window: the
/// outcome arrives as `Done` or `Failed`.
pub fn run(choices: Choices, sender: Sender<Event>) {
    let mut reporter = Reporter::new(sender);
    match install(&choices, &mut reporter) {
        Ok(root) => reporter.send(Event::Done(root)),
        Err(message) => reporter.send(Event::Failed(message)),
    }
}

fn install(choices: &Choices, report: &mut Reporter) -> Result<PathBuf, String> {
    if choices.placement != Placement::Stay {
        report.step(format!(
            "Déplacement de MOSNA GUI vers {}",
            choices.destination.display()
        ));
    }
    let root =
        place::relocate(&choices.source, &choices.destination, choices.placement).map_err(|e| {
            format!(
                "Le déplacement vers {} a échoué : {e}",
                choices.destination.display()
            )
        })?;
    report.line(format!("MOSNA GUI est dans {}", root.display()));

    // The build tools first: they are the one step that asks for an
    // administrator, and the question had better come while the user is still
    // in front of the window rather than half an hour in.
    install_build_tools(&root, report)?;
    let cargo = install_rust(&root, report)?;
    let base = conda::install_miniconda(&root, report)?;
    let python = conda::prepare_environment(&base, &root, report)?;

    report.step("Compilation de l'interface (10 à 20 minutes la première fois)");
    let cargo_bin = cargo.parent().map(Path::to_path_buf).unwrap_or_default();
    let mut build = Command::new(&cargo);
    build
        .args(["build", "--release", "--locked", "--bin", "mosna-gui"])
        .current_dir(&root)
        .env("PATH", prepend_path(&cargo_bin))
        // Cargo's colours, which the window shows; no progress bar, which
        // would only be noise in a log.
        .env("CARGO_TERM_COLOR", "always")
        .env("CARGO_TERM_PROGRESS_WHEN", "never");
    if !stream(&mut build, report)? {
        return Err(
            "La compilation de l'interface a échoué. Le détail est dans le journal ci-dessus."
                .into(),
        );
    }
    let interface = root.join(r"target\release").join(exe("mosna-gui"));
    if !interface.is_file() {
        return Err(format!(
            "La compilation s'est terminée mais {} n'existe pas.",
            interface.display()
        ));
    }

    report.step("Lanceur et raccourcis");
    launcher::install(&root, &base, &python, choices.desktop_shortcut, report)?;

    report.step("Suppression des fichiers propres à Linux");
    for name in LINUX_FILES {
        let file = root.join(name);
        if file.is_file() && std::fs::remove_file(&file).is_ok() {
            report.line(format!("supprimé : {name}"));
        }
    }
    Ok(root)
}

// ---------------------------------------------------------------------------
// Prerequisites
// ---------------------------------------------------------------------------

pub(crate) fn exe(name: &str) -> String {
    format!("{name}{}", std::env::consts::EXE_SUFFIX)
}

pub(crate) fn program_files_x86() -> PathBuf {
    std::env::var_os("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"))
}

/// Asked of vswhere rather than of the PATH: Git for Windows can put a GNU
/// `link.exe` there, which is no linker at all.
pub(crate) fn has_build_tools() -> bool {
    let vswhere = program_files_x86().join(r"Microsoft Visual Studio\Installer\vswhere.exe");
    Command::new(vswhere)
        .args(["-latest", "-products", "*", "-requires"])
        .arg("Microsoft.VisualStudio.Component.VC.Tools.x86.x64")
        .args(["-property", "installationPath"])
        .stderr(Stdio::null())
        .no_window()
        .output()
        .map(|output| output.status.success() && !output.stdout.trim_ascii().is_empty())
        .unwrap_or(false)
}

/// Installed before Rust: rustup-init, finding no linker, would offer the whole
/// of Visual Studio Community instead.
fn install_build_tools(root: &Path, report: &mut Reporter) -> Result<(), String> {
    if has_build_tools() {
        report.line("Outils C++ de Microsoft : déjà installés.");
        return Ok(());
    }
    report.step("Installation des outils C++ de Microsoft (plusieurs Go, la plus longue étape)");
    report.line("Windows va demander une autorisation administrateur.");
    let workload = "--wait --passive --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended";
    // winget reports a reboot-pending install as a failure, so what counts is
    // whether the tools are there afterwards.
    let installed =
        winget(report, BUILD_TOOLS_PACKAGE, &["--override", workload]) || has_build_tools();
    if !installed {
        let installer = std::env::temp_dir().join("vs_BuildTools.exe");
        download(
            report,
            "https://aka.ms/vs/17/release/vs_BuildTools.exe",
            &installer,
        )?;
        // The Visual Studio installer needs elevation, which only the shell
        // asks for: `start` goes through it, a plain CreateProcess does not.
        let mut command = Command::new("cmd");
        command.raw_args(&format!(
            "/c start \"\" /wait \"{}\" {workload}",
            installer.display()
        ));
        stream(&mut command, report)?;
    }
    if has_build_tools() {
        prerequisites::record(root, Prerequisite::BuildTools);
        Ok(())
    } else {
        Err("Les outils C++ de Microsoft n'ont pas pu être installés. Installez « Build Tools for Visual Studio » \
             (charge de travail « Développement Desktop en C++ ») depuis https://visualstudio.microsoft.com/fr/downloads/, \
             puis relancez INSTALLATION.exe."
            .into())
    }
}

pub(crate) fn cargo_path() -> Option<PathBuf> {
    let home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("USERPROFILE").map(|profile| PathBuf::from(profile).join(".cargo"))
        })?;
    let cargo = home.join("bin").join(exe("cargo"));
    cargo.is_file().then_some(cargo)
}

fn install_rust(root: &Path, report: &mut Reporter) -> Result<PathBuf, String> {
    if let Some(cargo) = cargo_path() {
        report.line("Rust : déjà installé.");
        return Ok(cargo);
    }
    report.step("Installation de Rust");
    let rustup = std::env::temp_dir().join("rustup-init.exe");
    download(report, "https://win.rustup.rs/x86_64", &rustup)?;
    let mut command = Command::new(&rustup);
    command.args([
        "-y",
        "--default-toolchain",
        "stable",
        "--profile",
        "default",
    ]);
    stream(&mut command, report)?;
    let cargo = cargo_path().ok_or_else(|| {
        "L'installation de Rust a échoué. Installez-le depuis https://rustup.rs, puis relancez INSTALLATION.exe.".to_string()
    })?;
    prerequisites::record(root, Prerequisite::Rust);
    Ok(cargo)
}

/// Install a package with winget. False when winget is missing or failed, so
/// the caller can fall back to the vendor's own installer.
fn winget(report: &mut Reporter, id: &str, extra: &[&str]) -> bool {
    let mut command = Command::new("winget");
    command
        .args(["install", "--id", id, "-e", "--silent"])
        .args(["--accept-package-agreements", "--accept-source-agreements"])
        .args(extra);
    run_winget(&mut command, report)
}

/// Remove a package with winget; false as for [`winget`].
pub(crate) fn winget_uninstall(report: &mut Reporter, id: &str) -> bool {
    let mut command = Command::new("winget");
    command
        .args(["uninstall", "--id", id, "-e", "--silent"])
        .arg("--accept-source-agreements");
    run_winget(&mut command, report)
}

fn run_winget(command: &mut Command, report: &mut Reporter) -> bool {
    match stream(command, report) {
        Ok(success) => success,
        Err(_) => {
            report.line("winget n'est pas disponible ; méthode directe.");
            false
        }
    }
}

/// Download with the `curl.exe` Windows has shipped since 2018.
pub(crate) fn download(report: &mut Reporter, url: &str, file: &Path) -> Result<(), String> {
    report.line(format!("Téléchargement de {url}"));
    let mut command = Command::new("curl.exe");
    command
        .args(["-fsSL", "--retry", "3", "-o"])
        .arg(file)
        .arg(url);
    if stream(&mut command, report)? && file.is_file() {
        Ok(())
    } else {
        Err(format!(
            "Le téléchargement de {url} a échoué. Vérifiez la connexion à Internet."
        ))
    }
}

/// The user's real desktop, from the registry: OneDrive redirects it.
pub(crate) fn desktop_folder() -> Option<PathBuf> {
    let output = Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders",
            "/v",
            "Desktop",
        ])
        .stderr(Stdio::null())
        .no_window()
        .output()
        .ok()?;
    registry_value(&String::from_utf8_lossy(&output.stdout)).map(PathBuf::from)
}

/// The value in `reg query` output: `    Desktop    REG_SZ    C:\Users\...`.
fn registry_value(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let (_, rest) = line.split_once("REG_")?;
        let (_, value) = rest.split_once(char::is_whitespace)?;
        let value = value.trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn prepend_path(directory: &Path) -> std::ffi::OsString {
    let mut paths = vec![directory.to_path_buf()];
    if let Some(path) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&path));
    }
    std::env::join_paths(paths).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Running a program, its output into the window
// ---------------------------------------------------------------------------

/// Hide the console window a program would otherwise open.
pub(crate) trait NoWindow {
    fn no_window(&mut self) -> &mut Self;
    fn raw_args(&mut self, arguments: &str) -> &mut Self;
}

impl NoWindow for Command {
    #[cfg(windows)]
    fn no_window(&mut self) -> &mut Self {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        self.creation_flags(CREATE_NO_WINDOW)
    }

    #[cfg(not(windows))]
    fn no_window(&mut self) -> &mut Self {
        self
    }

    /// Arguments passed as they are written, for `cmd`, which quotes by rules
    /// of its own.
    #[cfg(windows)]
    fn raw_args(&mut self, arguments: &str) -> &mut Self {
        use std::os::windows::process::CommandExt;
        self.raw_arg(arguments)
    }

    #[cfg(not(windows))]
    fn raw_args(&mut self, arguments: &str) -> &mut Self {
        self.args(arguments.split_whitespace())
    }
}

/// Run a program, forwarding each line it prints; whether it succeeded, or an
/// error when it could not be started at all.
pub(crate) fn stream(command: &mut Command, report: &mut Reporter) -> Result<bool, String> {
    let name = command.get_program().to_string_lossy().into_owned();
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .no_window()
        .spawn()
        .map_err(|e| format!("Impossible de lancer {name} : {e}"))?;

    let (lines, received) = std::sync::mpsc::channel::<String>();
    let readers: Vec<_> = [
        child
            .stdout
            .take()
            .map(|out| Box::new(out) as Box<dyn Read + Send>),
        child
            .stderr
            .take()
            .map(|err| Box::new(err) as Box<dyn Read + Send>),
    ]
    .into_iter()
    .flatten()
    .map(|pipe| {
        let lines = lines.clone();
        std::thread::spawn(move || {
            // Split on bytes: a Windows program's output is rarely UTF-8.
            for line in BufReader::new(pipe).split(b'\n').map_while(Result::ok) {
                let text = String::from_utf8_lossy(&line).trim_end().to_string();
                if lines.send(text).is_err() {
                    break;
                }
            }
        })
    })
    .collect();
    drop(lines);

    for line in received {
        report.line(line);
    }
    for reader in readers {
        let _ = reader.join();
    }
    let status = child.wait().map_err(|e| format!("{name} : {e}"))?;
    Ok(status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_desktop_is_read_from_reg_output() {
        let output = "\r\nHKEY_CURRENT_USER\\Software\\...\\Shell Folders\r\n    Desktop    REG_SZ    C:\\Users\\Me\\OneDrive\\Bureau\r\n\r\n";
        assert_eq!(
            registry_value(output).as_deref(),
            Some(r"C:\Users\Me\OneDrive\Bureau")
        );
        assert_eq!(registry_value("ERROR: not found"), None);
    }

    #[cfg(unix)]
    #[test]
    fn output_is_forwarded_line_by_line() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut reporter = Reporter { sender, log: None };
        let mut command = Command::new("sh");
        command.args(["-c", "echo one; echo two >&2; exit 3"]);
        assert_eq!(stream(&mut command, &mut reporter), Ok(false));
        let lines: Vec<Event> = receiver.try_iter().collect();
        assert!(lines.contains(&Event::Line("one".into())));
        assert!(lines.contains(&Event::Line("two".into())));
    }

    #[test]
    fn a_missing_program_is_an_error_not_a_panic() {
        let (sender, _receiver) = std::sync::mpsc::channel();
        let mut reporter = Reporter { sender, log: None };
        assert!(stream(&mut Command::new("no-such-program-anywhere"), &mut reporter).is_err());
    }
}
