//! The two committed Windows programs, and how they fit beside `setup.sh`.
//!
//! They are built from this crate but committed as binaries, so nothing
//! rebuilds them by itself: what can be checked is that they are there, that
//! they are window programs carrying the manifest that keeps Windows from
//! running them as an administrator, and that `setup.sh` deletes exactly them.

use std::path::{Path, PathBuf};

const PROGRAMS: [&str; 2] = ["INSTALLATION.exe", "UNINSTALL.exe"];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

#[test]
fn both_programs_are_windows_programs_with_a_window() {
    for name in PROGRAMS {
        let bytes =
            std::fs::read(root().join(name)).unwrap_or_else(|_| panic!("{name} is missing"));
        assert_eq!(&bytes[..2], b"MZ", "{name} is not a Windows executable");
        let pe = u32::from_le_bytes(bytes[0x3C..0x40].try_into().unwrap()) as usize;
        assert_eq!(&bytes[pe..pe + 4], b"PE\0\0");
        // IMAGE_SUBSYSTEM_WINDOWS_GUI, in the PE32+ optional header.
        let subsystem = u16::from_le_bytes(bytes[pe + 24 + 68..pe + 24 + 70].try_into().unwrap());
        assert_eq!(subsystem, 2, "{name} would open a console window");
    }
}

/// Without a manifest, Windows takes a program named INSTALLATION for a legacy
/// installer and runs it as an administrator — for a user who is not one, an
/// administrator's account, so MOSNA GUI would land in someone else's profile.
#[test]
fn both_programs_run_as_the_user() {
    let manifest = b"requestedExecutionLevel level=\"asInvoker\"";
    for name in PROGRAMS {
        let bytes = std::fs::read(root().join(name)).unwrap();
        assert!(
            bytes
                .windows(manifest.len())
                .any(|window| window == manifest),
            "{name} carries no asInvoker manifest; rebuild it (see crates/mosna-setup/src/lib.rs)"
        );
    }
}

/// setup.sh deletes the Windows programs once it has installed; a name that
/// no longer exists means the list has drifted.
#[test]
fn setup_sh_deletes_exactly_the_windows_programs() {
    let text = std::fs::read_to_string(root().join("setup.sh")).unwrap();
    let line = text
        .lines()
        .find(|line| {
            line.trim_start()
                .starts_with("for name in INSTALLATION.exe")
        })
        .expect("setup.sh no longer deletes the Windows programs");
    let names: Vec<&str> = line.trim_start()["for name in ".len()..]
        .trim_end_matches("; do")
        .split_whitespace()
        .collect();
    assert_eq!(names, PROGRAMS);
}

/// The batch installer the programs replace is gone, and nothing still sends
/// the reader to it.
#[test]
fn the_former_batch_installer_is_gone() {
    assert!(!root().join("setup_windows.bat").exists());
    let readme = std::fs::read_to_string(root().join("README.md")).unwrap();
    assert!(!readme.contains("setup_windows.bat"));
}
