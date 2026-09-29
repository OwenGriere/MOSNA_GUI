//! Launching an analysis and reading its output.
//!
//! The analyses are Python: each step is `python -m package.<module>`, with
//! the two flags those modules declare — so a step can be run from a terminal
//! exactly as the interface runs it.

use std::path::Path;

/// The buttons of the action bar.
///
/// Three analyses, and one thing to do with the directory they filled: take
/// the intermediates back out of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Tysserand,
    Assortativity,
    NicheAnalysis,
    ClearTemporary,
}

impl Step {
    /// Every step, in the order the buttons appear.
    pub fn all() -> [Step; 4] {
        [
            Step::Tysserand,
            Step::Assortativity,
            Step::NicheAnalysis,
            Step::ClearTemporary,
        ]
    }

    /// The action bar, row by row.
    ///
    /// An analysis takes the full width: it is the thing the user came to do,
    /// and its caption is long. Clearing has a row of its own under them,
    /// separated by the space the panel leaves between rows, so the
    /// destructive button is not one of a stack that all start work.
    ///
    /// Written here rather than in the panel so the arrangement can be stated
    /// as a test instead of discovered by looking.
    pub fn rows() -> Vec<Vec<Step>> {
        vec![
            vec![Step::Tysserand],
            vec![Step::Assortativity],
            vec![Step::NicheAnalysis],
            vec![Step::ClearTemporary],
        ]
    }

    /// The button's caption.
    pub fn label(self) -> &'static str {
        match self {
            Step::Tysserand => "Step 1 — Tysserand",
            Step::Assortativity => "Step 2 — Assortativity",
            Step::NicheAnalysis => "Step 3 — Niche Analysis",
            Step::ClearTemporary => "Clear temporary data",
        }
    }

    /// What the button does, spelled out for the tooltip.
    ///
    /// The last one needs it most: it deletes, and a caption of three words is
    /// not enough to say what.
    pub fn hint(self) -> &'static str {
        match self {
            Step::Tysserand => "Reconstruct a spatial network for every sample.",
            Step::Assortativity => "Measure which cell types sit next to which.",
            Step::NicheAnalysis => "Group neighbourhoods into spatial niches.",
            Step::ClearTemporary => {
                "Delete the temp folder and the intermediate networks in it.  \
                 The figures and the tables are kept."
            }
        }
    }

    /// The Python module this step runs, as `python -m` spells it.
    pub fn module(self) -> &'static str {
        match self {
            Step::Tysserand => "package.tysserand_network",
            Step::Assortativity => "package.assortativity",
            Step::NicheAnalysis => "package.niche_analysis",
            Step::ClearTemporary => "package.clear_temporary",
        }
    }

    /// Whether the step reads `configuration.yaml`.
    ///
    /// Clearing does not: `clear_temporary.py`'s parser declares only
    /// `--working_dir`, and passing a flag it does not know would make the
    /// button fail with a usage message.
    pub fn takes_config(self) -> bool {
        !matches!(self, Step::ClearTemporary)
    }

    /// The full argument list for the interpreter, flags included.
    pub fn arguments(self, config_path: &Path, working_dir: &Path) -> Vec<String> {
        let mut arguments = vec!["-m".to_string(), self.module().to_string()];
        if self.takes_config() {
            arguments.push("--file".to_string());
            arguments.push(config_path.to_string_lossy().into_owned());
        }
        arguments.push("--working_dir".to_string());
        arguments.push(working_dir.to_string_lossy().into_owned());
        arguments
    }
}

/// What a line of the process's stdout carries.
#[derive(Debug, Clone, PartialEq)]
pub enum OutputLine {
    /// A status message for the status bar.
    Info(String),
    /// A position in the current step.
    Progress {
        current: usize,
        total: usize,
        description: String,
    },
    /// Anything else: shown in the log, nothing more.
    Plain,
}

/// Parse one line of the analysis process's stdout.
///
/// The protocol is `package/utils/emit_qt_progress.py`'s, unchanged:
///
/// ```text
/// [QT_INFO] <message>
/// [QT_PROGRESS] current=<n> total=<n> desc=<text>
/// ```
///
/// A malformed progress line is treated as plain output rather than guessed at:
/// a wrong `total` would leave the progress bar stuck.
pub fn parse_output_line(line: &str) -> OutputLine {
    if let Some(message) = line.strip_prefix("[QT_INFO]") {
        return OutputLine::Info(message.trim().to_string());
    }

    if let Some(payload) = line.strip_prefix("[QT_PROGRESS]") {
        let current = field(payload, "current=").and_then(|v| v.parse().ok());
        let total = field(payload, "total=").and_then(|v| v.parse().ok());
        if let (Some(current), Some(total)) = (current, total) {
            let description = payload
                .find("desc=")
                .map(|at| payload[at + "desc=".len()..].trim().to_string())
                .unwrap_or_default();
            return OutputLine::Progress {
                current,
                total,
                description,
            };
        }
    }

    OutputLine::Plain
}

/// The whitespace-delimited value following `name` in `payload`.
fn field<'a>(payload: &'a str, name: &str) -> Option<&'a str> {
    let at = payload.find(name)? + name.len();
    let rest = &payload[at..];
    Some(rest.split_whitespace().next().unwrap_or(rest))
}

/// Render a duration the way the status line reads it.
pub fn format_duration(seconds: f64) -> String {
    let total = seconds.round() as i64;
    let (hours, remainder) = (total / 3600, total % 3600);
    let (minutes, secs) = (remainder / 60, remainder % 60);

    if hours > 0 {
        format!("{hours} h {minutes} min {secs} s")
    } else if minutes > 0 {
        format!("{minutes} min {secs} s")
    } else {
        format!("{seconds:.2} s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clear_step_takes_no_configuration() {
        let arguments = Step::ClearTemporary.arguments(Path::new("/cfg.yaml"), Path::new("/work"));
        assert_eq!(
            arguments,
            vec!["-m", "package.clear_temporary", "--working_dir", "/work"]
        );
    }

    /// The action bar: the three analyses each on their own line, and the one
    /// operation on a finished directory under them.
    #[test]
    fn the_last_row_holds_the_operation_on_a_finished_directory() {
        let rows = Step::rows();

        assert_eq!(rows.len(), 4, "the bar is not four rows: {rows:?}");
        for row in &rows {
            assert_eq!(row.len(), 1, "a button shares its row");
        }
        assert_eq!(rows[3], vec![Step::ClearTemporary]);
    }

    /// Every step is reachable, and none is offered twice — the rows are the
    /// only thing the panel draws from.
    #[test]
    fn every_step_appears_in_the_rows_exactly_once() {
        let mut drawn: Vec<Step> = Step::rows().into_iter().flatten().collect();
        let mut expected = Step::all().to_vec();

        drawn.sort_by_key(|step| step.module());
        expected.sort_by_key(|step| step.module());
        assert_eq!(drawn, expected);
    }

    /// The module name is the contract with the Python package; a typo here is
    /// a button that reports `No module named …` when it is pressed.
    #[test]
    fn every_step_names_a_module_of_the_package() {
        for step in Step::all() {
            assert!(
                step.module().starts_with("package."),
                "{} runs {}",
                step.label(),
                step.module()
            );
        }
    }

    #[test]
    fn an_analysis_step_passes_both_flags() {
        let arguments = Step::Tysserand.arguments(Path::new("/cfg.yaml"), Path::new("/work"));
        assert_eq!(
            arguments,
            vec![
                "-m",
                "package.tysserand_network",
                "--file",
                "/cfg.yaml",
                "--working_dir",
                "/work"
            ]
        );
    }

    #[test]
    fn progress_lines_carry_their_description() {
        let parsed = parse_output_line("[QT_PROGRESS] current=7 total=9 desc=[PROCESS] Doing it");
        assert_eq!(
            parsed,
            OutputLine::Progress {
                current: 7,
                total: 9,
                description: "[PROCESS] Doing it".to_string(),
            }
        );
    }

    #[test]
    fn a_progress_line_without_a_description_still_parses() {
        assert_eq!(
            parse_output_line("[QT_PROGRESS] current=1 total=2"),
            OutputLine::Progress {
                current: 1,
                total: 2,
                description: String::new(),
            }
        );
    }

    #[test]
    fn a_malformed_progress_line_is_not_guessed_at() {
        assert_eq!(
            parse_output_line("[QT_PROGRESS] current=x total=2"),
            OutputLine::Plain
        );
        assert_eq!(parse_output_line("[QT_PROGRESS]"), OutputLine::Plain);
    }

    #[test]
    fn info_lines_are_trimmed() {
        assert_eq!(
            parse_output_line("[QT_INFO]   spaced   "),
            OutputLine::Info("spaced".to_string())
        );
    }

    #[test]
    fn durations_match_the_python_format() {
        assert_eq!(format_duration(0.5), "0.50 s");
        assert_eq!(format_duration(59.0), "59.00 s");
        assert_eq!(format_duration(60.0), "1 min 0 s");
        assert_eq!(format_duration(3600.0), "1 h 0 min 0 s");
    }

    #[test]
    fn the_buttons_appear_in_workflow_order() {
        let labels: Vec<&str> = Step::all().iter().map(|s| s.label()).collect();
        assert_eq!(labels[0], "Step 1 — Tysserand");
        assert_eq!(labels[2], "Step 3 — Niche Analysis");
        assert_eq!(
            *labels.last().unwrap(),
            "Clear temporary data",
            "clearing is still the last thing offered"
        );
    }
}
