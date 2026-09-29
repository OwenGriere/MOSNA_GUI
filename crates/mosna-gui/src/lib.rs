//! Graphical interface of MOSNA.
//!
//! Three panels in a splitter — the data on the left, the figures in the
//! middle, the settings on the right — one widget per configuration key, and a
//! button per step of the workflow.
//!
//! The crate is split so that everything except the drawing is testable:
//!
//! * [`model`] holds the logic — which widget a key gets, how its value reads
//!   back, how the form is laid out, how a log line is classified, how a
//!   progress line is parsed, which figures belong to which patient.
//! * [`panels`] draws it.
//! * [`docs`] is the manual, as data the interface draws itself.
//! * [`theme`] is the palette.
//!
//! # Where the science is
//!
//! Not here. Each step runs `python -m package.<module>` as a sub-process and
//! this reads its `[QT_INFO]` / `[QT_PROGRESS]` output, so the analyses are
//! exactly the Python ones and can still be run from a terminal without the
//! interface at all.

pub mod app;
pub mod colormap;
pub mod docs;
pub mod icon;
pub mod model;
pub mod panels;
pub mod theme;

pub use app::MosnaApp;
