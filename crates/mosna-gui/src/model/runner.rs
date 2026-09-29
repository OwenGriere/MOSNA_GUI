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

/// What to tell the user a failed run failed of.
///
/// # Why the last line is not good enough
///
/// It was the rule, and it works whenever a Python traceback is the last
/// thing written: the final line of a traceback *is* the exception. But the
/// analyses run `dask`, whose workers log their own shutdown after the
/// exception has already propagated — pages of `distributed.worker - ERROR`
/// and tornado frames. The dialog then reported `^^^^^^^^^^^` , or a line of
/// somebody else's stack, as the cause of the run failing.
///
/// So the exception line is looked for first, and the old rule is kept as the
/// fallback for a failure that is not a Python traceback at all — a module
/// that could not be imported, an interpreter that is not there.
pub fn failure_reason<'a>(lines: impl DoubleEndedIterator<Item = &'a str>) -> String {
    let candidates: Vec<&str> = lines
        .filter(|line| {
            !line.contains("[QT_PROGRESS]")
                && !line.contains("[QT_INFO]")
                && !line.trim().is_empty()
        })
        .collect();

    candidates
        .iter()
        .rev()
        .find(|line| is_exception(line))
        .or_else(|| candidates.last())
        .map(|line| line.trim().to_string())
        .unwrap_or_else(|| "Unknown error.".to_string())
}

/// Whether a line is the `SomeError: what happened` that ends a traceback.
///
/// Matched on shape rather than against a list of exception names: the
/// analyses raise from `pandas`, `numpy`, `mosna` and `tysserand`, and a list
/// would be a thing to keep up to date for no gain. A dotted path is allowed
/// because that is how a traceback names an exception from another module —
/// `tornado.iostream.StreamClosedError`.
fn is_exception(line: &str) -> bool {
    // Indented lines are stack frames, not the exception.
    if line.starts_with(char::is_whitespace) {
        return false;
    }
    let Some((name, message)) = line.split_once(": ") else {
        return false;
    };
    if message.trim().is_empty() {
        return false;
    }
    let mut parts = name.split('.').peekable();
    let mut segments = 0;
    while let Some(part) = parts.next() {
        segments += 1;
        let first = part.chars().next();
        let identifier = !part.is_empty()
            && first.is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !identifier {
            return false;
        }
        // Only the last segment carries the exception's own name.
        if parts.peek().is_none() {
            return part.ends_with("Error") || part.ends_with("Exception");
        }
    }
    segments == 0
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

    // -----------------------------------------------------------------------
    // What a failed run is reported to have failed of
    // -----------------------------------------------------------------------

    /// The exception, not whatever dask happened to log on its way out.
    #[test]
    fn the_reason_is_the_exception_not_the_noise_after_it() {
        let log = [
            "[QT_INFO] Niches found",
            "Traceback (most recent call last):",
            "  File \"niche_analysis.py\", line 186, in main",
            "    c_map = generate_cmap(net_dir, 'niches', 'parquet')",
            "KeyError: 'niches'",
            "2026-09-29 16:38:45,345 - distributed.worker - ERROR - Failed to communicate",
            "Traceback (most recent call last):",
            "  File \"distributed/comm/tcp.py\", line 226, in read",
            "                                ^^^^^^^^^^^^^^^^^^^^^",
        ];
        assert_eq!(failure_reason(log.into_iter()), "KeyError: 'niches'");
    }

    /// A dotted name is still an exception: that is how a traceback spells one
    /// raised in another module.
    #[test]
    fn a_dotted_exception_name_is_recognised() {
        let log = ["tornado.iostream.StreamClosedError: Stream is closed"];
        assert_eq!(
            failure_reason(log.into_iter()),
            "tornado.iostream.StreamClosedError: Stream is closed"
        );
    }

    /// A failure that is not a traceback still says something useful.
    #[test]
    fn a_failure_without_a_traceback_falls_back_to_the_last_line() {
        let log = [
            "[QT_INFO] starting",
            "/usr/bin/python: No module named package.tysserand_network",
        ];
        assert_eq!(
            failure_reason(log.into_iter()),
            "/usr/bin/python: No module named package.tysserand_network"
        );
    }

    /// Progress and status lines are the protocol, not an explanation.
    #[test]
    fn the_protocol_is_never_offered_as_a_reason() {
        let log = [
            "[QT_PROGRESS] current=1 total=2 desc=working",
            "[QT_INFO] almost there",
            "   ",
        ];
        assert_eq!(failure_reason(log.into_iter()), "Unknown error.");
    }

    /// A stack frame mentioning a colon is not the exception.
    #[test]
    fn an_indented_frame_is_not_mistaken_for_the_exception() {
        let log = [
            "ValueError: the matrix holds no usable value",
            "  File \"x.py\", line 3: in main",
        ];
        assert_eq!(
            failure_reason(log.into_iter()),
            "ValueError: the matrix holds no usable value"
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
