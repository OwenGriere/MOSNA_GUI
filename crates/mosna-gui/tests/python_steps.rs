//! The seam between the two halves: the interface really does start the
//! Python modules of `package/`.
//!
//! Every other test here checks the *arguments* a step would build. That is
//! the cheap half of the contract and not the half that breaks: what breaks is
//! the interpreter not being found, the project root not being found, or
//! `package` not being importable from the working directory the analyses run
//! in — and none of those show up until a button is pressed on a real machine.
//!
//! `clear-temporary` is the step used, because it is the only one that needs
//! nothing from the scientific stack: `clear_temporary.py` imports `pathlib`,
//! `shutil` and `argparse`. The other three are the same code path with a
//! different module name.

use std::path::Path;
use std::time::{Duration, Instant};

use mosna_gui::model::runner::Step;
use mosna_gui::MosnaApp;

/// Whether an interpreter can be started at all.
///
/// Reported as a skip rather than a failure: the interface is built on
/// machines that have no Python, and a compile that fails there would be worse
/// than a test that says so.
fn have_python() -> bool {
    let environment = mosna_paths::Environment::detect();
    std::process::Command::new(mosna_paths::python::resolve(&environment))
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

fn app(working_dir: &Path) -> MosnaApp {
    let path = working_dir.join("configuration.yaml");
    std::fs::write(
        &path,
        "Tysserand:\n  Nodes directory: /data\n  Patient column name: patient\n  \
         Extension: parquet\n  Min neighbors: 3\n",
    )
    .unwrap();
    let mut app = MosnaApp::new(path);
    app.browser.working_dir = Some(working_dir.to_path_buf());
    app
}

/// Run the step to completion, or give up after `limit`.
fn run_to_completion(app: &mut MosnaApp, limit: Duration) -> bool {
    let started = Instant::now();
    while app.run.is_some() {
        if started.elapsed() > limit {
            return false;
        }
        std::thread::sleep(Duration::from_millis(20));
        app.poll_run();
    }
    true
}

/// The project root has to be found from wherever the tests run, or every
/// button reports `No module named package`.
#[test]
fn the_python_package_is_found_from_the_test_runner() {
    let environment = mosna_paths::Environment::detect();
    let root = mosna_paths::project::resolve(&environment).expect("a project root");
    assert!(
        mosna_paths::project::is_project_root(&root),
        "{} has no package/__init__.py",
        root.display()
    );
}

/// And pressing the button really starts it, in the working directory, and
/// really removes what it was asked to remove.
#[test]
fn clearing_runs_the_python_module_and_deletes_the_temp_folder() {
    if !have_python() {
        eprintln!("skipping: no interpreter");
        return;
    }

    let directory = tempfile::tempdir().unwrap();
    let temp = directory.path().join("temp");
    std::fs::create_dir_all(temp.join("net_dir_mosna")).unwrap();
    std::fs::write(temp.join("net_dir_mosna").join("nodes_x.parquet"), b"").unwrap();

    let mut app = app(directory.path());
    app.start(Step::ClearTemporary);

    assert!(
        app.run.is_some(),
        "the step did not start: {:?}",
        app.notice
    );
    assert!(
        run_to_completion(&mut app, Duration::from_secs(60)),
        "the step never finished"
    );

    assert!(!app.last_run_failed, "the step failed: {:?}", app.log);
    assert!(!temp.exists(), "temp/ is still there");
    assert!(app.status.contains("completed"), "{}", app.status);
}

/// The log carries what the module printed, which is what makes a failure
/// readable in the Viewer rather than only in a terminal nobody sees.
#[test]
fn the_modules_output_reaches_the_log() {
    if !have_python() {
        eprintln!("skipping: no interpreter");
        return;
    }

    let directory = tempfile::tempdir().unwrap();
    let mut app = app(directory.path());
    app.start(Step::ClearTemporary);
    assert!(run_to_completion(&mut app, Duration::from_secs(60)));

    let log: Vec<&str> = app.log.iter().map(|(_, line)| line.as_str()).collect();
    assert!(
        log.iter().any(|line| line.contains("temporary folder")),
        "nothing the module printed reached the log: {log:?}"
    );
}
