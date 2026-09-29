//! Finding the Python project the interface drives.
//!
//! The analyses are `python -m package.<module>`, so the interpreter has to be
//! started from — or at least pointed at — the directory holding `package/`.
//! Run from a checkout that is the repository root; installed it is wherever
//! the sources were put, which is why the lookup is here rather than assumed
//! at the call site.

use std::path::{Path, PathBuf};

use crate::Environment;

/// The package the analyses live in.
pub const PACKAGE_DIR: &str = "package";

/// The file that proves a directory really is the project root, rather than
/// some other directory that happens to contain a `package` folder.
pub const PACKAGE_MARKER: &str = "__init__.py";

/// How far up from a starting point the root is looked for.
///
/// `cargo test` runs each crate's tests from that crate's own directory, two
/// levels below the workspace root; four leaves room for a deeper layout
/// without turning the search into a walk to the file system root.
const SEARCH_DEPTH: usize = 5;

/// Whether `directory` is the root of the Python project.
pub fn is_project_root(directory: &Path) -> bool {
    directory.join(PACKAGE_DIR).join(PACKAGE_MARKER).is_file()
}

/// The nearest ancestor of `start` (itself included) that is a project root.
pub fn search_upwards(start: &Path) -> Option<PathBuf> {
    let mut directory = start.to_path_buf();
    for _ in 0..=SEARCH_DEPTH {
        if is_project_root(&directory) {
            return Some(directory);
        }
        directory = directory.parent()?.to_path_buf();
    }
    None
}

/// Locate the project root, in order of precedence:
///
/// 1. `MOSNA_GUI_ROOT`, taken verbatim, so a packaged interface can be pointed
///    at a checkout without reinstalling — and so a typo surfaces as
///    "no module named package" rather than being silently swapped for
///    another copy of the sources;
/// 2. above the running executable, which is where `cargo build` and the
///    installer both put it (`target/release/mosna-gui`);
/// 3. above the current directory, for an interface started from the sources;
/// 4. the current directory, so the failure names somewhere real.
pub fn resolve(environment: &Environment) -> Option<PathBuf> {
    if let Some(explicit) = &environment.mosna_gui_root {
        return Some(explicit.clone());
    }

    environment
        .exe_dir
        .as_deref()
        .and_then(search_upwards)
        .or_else(|| environment.current_dir.as_deref().and_then(search_upwards))
        .or_else(|| environment.current_dir.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A directory with `package/__init__.py` in it, and two levels of
    /// nesting under it to search up from.
    fn checkout() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(PACKAGE_DIR)).unwrap();
        std::fs::write(dir.path().join(PACKAGE_DIR).join(PACKAGE_MARKER), "").unwrap();
        std::fs::create_dir_all(dir.path().join("target").join("release")).unwrap();
        dir
    }

    #[test]
    fn the_root_is_found_from_the_binary_that_was_built_in_it() {
        let dir = checkout();
        let environment = Environment {
            exe_dir: Some(dir.path().join("target").join("release")),
            ..Default::default()
        };
        assert_eq!(resolve(&environment).as_deref(), Some(dir.path()));
    }

    #[test]
    fn an_override_is_taken_verbatim_even_when_absent() {
        let environment = Environment {
            mosna_gui_root: Some(PathBuf::from("/nowhere/MOSNA_GUI")),
            exe_dir: Some(checkout().path().join("target").join("release")),
            ..Default::default()
        };
        assert_eq!(
            resolve(&environment),
            Some(PathBuf::from("/nowhere/MOSNA_GUI"))
        );
    }

    #[test]
    fn a_directory_without_the_package_is_not_a_root() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(PACKAGE_DIR)).unwrap();
        assert!(!is_project_root(dir.path()));
    }
}
