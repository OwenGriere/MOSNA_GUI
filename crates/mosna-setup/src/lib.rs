//! The graphical Windows installer of MOSNA GUI, shipped at the root of the
//! repository as `INSTALLATION.exe` so that installing is a double-click.
//!
//! It asks where to put MOSNA GUI and whether to add a desktop shortcut; then
//! moves the folder there, installs what a fresh Windows lacks (the Microsoft
//! C++ build tools, Rust, Miniconda), builds the `mosna-GUI` conda environment
//! the analyses run in, compiles the interface, writes its launcher and its
//! shortcuts — the same result as `setup.sh` on Linux — and finally deletes the
//! files that only serve Linux.
//!
//! Beside it, `UNINSTALL.exe` (the `mosna-uninstall` binary) removes MOSNA GUI
//! again and, if asked, what was installed to build it.
//!
//! Both executables are committed, since the whole point is not needing a
//! compiler to install. After changing this crate, rebuild them from Linux with:
//!
//! ```text
//! rustup target add x86_64-pc-windows-gnu
//! cargo install cargo-zigbuild        # and zig, e.g. `pip install ziglang`
//! cargo zigbuild --release -p mosna-setup --target x86_64-pc-windows-gnu
//! cp target/x86_64-pc-windows-gnu/release/mosna-setup.exe INSTALLATION.exe
//! cp target/x86_64-pc-windows-gnu/release/mosna-uninstall.exe UNINSTALL.exe
//! ```

pub mod conda;
pub mod console;
pub mod install;
pub mod launcher;
pub mod place;
pub mod prerequisites;
pub mod relaunch;
pub mod theme;
pub mod uninstall;
pub mod uninstall_window;
pub mod window;
