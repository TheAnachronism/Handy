use clap::{error::ErrorKind, CommandFactory, Parser, ValueEnum};
use std::ffi::OsString;
use std::path::PathBuf;

use crate::live_insertion::LiveInsertionLookahead;

#[derive(Parser, Debug, Clone, Default)]
#[command(name = "handy", about = "Handy - Speech to Text")]
pub struct CliArgs {
    /// Start with the main window hidden
    #[arg(long)]
    pub start_hidden: bool,

    /// Disable the system tray icon
    #[arg(long)]
    pub no_tray: bool,

    /// Toggle transcription on/off (sent to running instance)
    #[arg(long)]
    pub toggle_transcription: bool,

    /// Toggle transcription with post-processing on/off (sent to running instance)
    #[arg(long)]
    pub toggle_post_process: bool,

    /// Cancel the current operation (sent to running instance)
    #[arg(long)]
    pub cancel: bool,

    /// Lookahead Override for one Plain Dictation start (Live Insertion on only).
    /// Requires --toggle-transcription or --toggle-post-process.
    #[arg(long, value_enum, value_name = "PRESET")]
    pub lookahead: Option<LiveInsertionLookahead>,

    /// Enable Live Insertion for one Plain Dictation start. Does not persist.
    /// Requires --toggle-transcription or --toggle-post-process.
    #[arg(long)]
    pub live_insertion: bool,

    /// Enable debug mode with verbose logging
    #[arg(long)]
    pub debug: bool,

    /// Transcribe this WAV (16 kHz mono) headlessly and exit. Runs the same
    /// batch transcription path as the app — no mic, no VAD, no download
    /// (the model must already be installed).
    #[arg(short = 'f', long, value_name = "WAV")]
    pub transcribe_file: Option<PathBuf>,

    /// Model id to load for --transcribe-file (default: the selected model).
    #[arg(long)]
    pub model: Option<String>,

    /// Hard-select the compute device for --transcribe-file by its registry
    /// index (see --list-devices). Omit to use the persisted accelerator
    /// setting. transcribe-cpp (whisper-family) models only.
    #[arg(long, value_name = "N")]
    pub device_index: Option<usize>,

    /// List the transcribe-cpp compute devices (with indices) and exit.
    #[arg(long)]
    pub list_devices: bool,

    /// List the available models (with ids) and exit. Pass an id to --model.
    /// Honors --json for machine-readable output.
    #[arg(long)]
    pub list_models: bool,

    /// Repeat the transcription N times (best_ms reports the fastest run).
    #[arg(long, value_name = "N")]
    pub repeat: Option<usize>,

    /// Emit --transcribe-file results as JSON.
    #[arg(long)]
    pub json: bool,

    /// Quit the running Handy instance. If no instance is running, exit
    /// without starting the app. Cannot be combined with other actions.
    #[arg(long)]
    pub quit: bool,
}

impl CliArgs {
    pub fn parse_strict() -> Self {
        Self::try_parse_strict_from(std::env::args_os()).unwrap_or_else(|e| e.exit())
    }

    pub fn try_parse_strict_from<I, T>(itr: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let args = Self::try_parse_from(itr)?;
        let has_start = args.toggle_transcription || args.toggle_post_process;
        if args.lookahead.is_some() && !has_start {
            return Err(Self::command().error(
                ErrorKind::MissingRequiredArgument,
                "--lookahead requires --toggle-transcription or --toggle-post-process",
            ));
        }
        if args.live_insertion && !has_start {
            return Err(Self::command().error(
                ErrorKind::MissingRequiredArgument,
                "--live-insertion requires --toggle-transcription or --toggle-post-process",
            ));
        }
        if args.quit
            && (has_start
                || args.cancel
                || args.transcribe_file.is_some()
                || args.list_devices
                || args.list_models
                || args.model.is_some()
                || args.device_index.is_some()
                || args.repeat.is_some()
                || args.json)
        {
            return Err(Self::command().error(
                ErrorKind::ArgumentConflict,
                "--quit cannot be combined with other actions",
            ));
        }
        Ok(args)
    }
}

/// Read `--lookahead` from forwarded single-instance argv (already clap-validated
/// in the sending process).
pub fn lookahead_from_argv<S: AsRef<str>>(
    args: impl IntoIterator<Item = S>,
) -> Option<LiveInsertionLookahead> {
    let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if let Some(rest) = a.strip_prefix("--lookahead=") {
            return LiveInsertionLookahead::from_str(rest, false).ok();
        }
        if a == "--lookahead" {
            return args
                .get(i + 1)
                .and_then(|s| LiveInsertionLookahead::from_str(s, false).ok());
        }
        i += 1;
    }
    None
}

/// Read `--live-insertion` from forwarded single-instance argv (already clap-validated
/// in the sending process).
pub fn live_insertion_from_argv<S: AsRef<str>>(args: impl IntoIterator<Item = S>) -> bool {
    args.into_iter().any(|s| s.as_ref() == "--live-insertion")
}


#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<CliArgs, clap::Error> {
        let mut argv = vec!["handy"];
        argv.extend_from_slice(args);
        CliArgs::try_parse_strict_from(&argv)
    }

    #[test]
    fn lookahead_without_start_flag_is_usage_error() {
        assert!(parse(&["--lookahead", "balanced"]).is_err());
    }

    #[test]
    fn invalid_lookahead_is_usage_error() {
        assert!(parse(&["--toggle-transcription", "--lookahead", "snappy"]).is_err());
        assert!(parse(&["--toggle-transcription", "--lookahead", "1"]).is_err());
    }

    #[test]
    fn toggle_transcription_accepts_each_preset() {
        for (name, expected) in [
            ("fastest", LiveInsertionLookahead::Fastest),
            ("fast", LiveInsertionLookahead::Fast),
            ("balanced", LiveInsertionLookahead::Balanced),
            ("accurate", LiveInsertionLookahead::Accurate),
        ] {
            let args = parse(&["--toggle-transcription", "--lookahead", name]).unwrap();
            assert_eq!(args.lookahead, Some(expected), "{name}");
        }
    }

    #[test]
    fn toggle_post_process_accepts_lookahead_without_error() {
        let args = parse(&["--toggle-post-process", "--lookahead", "fast"]).unwrap();
        assert_eq!(args.lookahead, Some(LiveInsertionLookahead::Fast));
    }

    #[test]
    fn toggle_without_lookahead_leaves_override_absent() {
        let args = parse(&["--toggle-transcription"]).unwrap();
        assert_eq!(args.lookahead, None);
    }

    #[test]
    fn lookahead_from_forwarded_argv() {
        assert_eq!(
            lookahead_from_argv(["handy", "--toggle-transcription", "--lookahead", "balanced"]),
            Some(LiveInsertionLookahead::Balanced)
        );
        assert_eq!(
            lookahead_from_argv(["handy", "--toggle-transcription", "--lookahead=accurate"]),
            Some(LiveInsertionLookahead::Accurate)
        );
        assert_eq!(
            lookahead_from_argv(["handy", "--toggle-transcription"]),
            None
        );
    }

    #[test]
    fn live_insertion_without_start_flag_is_usage_error() {
        assert!(parse(&["--live-insertion"]).is_err());
    }

    #[test]
    fn toggle_transcription_accepts_live_insertion() {
        let args = parse(&["--toggle-transcription", "--live-insertion"]).unwrap();
        assert!(args.live_insertion);
        assert_eq!(args.lookahead, None);
    }

    #[test]
    fn toggle_post_process_accepts_live_insertion_without_error() {
        let args = parse(&["--toggle-post-process", "--live-insertion"]).unwrap();
        assert!(args.live_insertion);
    }

    #[test]
    fn toggle_without_live_insertion_leaves_flag_off() {
        let args = parse(&["--toggle-transcription"]).unwrap();
        assert!(!args.live_insertion);
    }

    #[test]
    fn live_insertion_and_lookahead_together() {
        let args = parse(&[
            "--toggle-transcription",
            "--live-insertion",
            "--lookahead",
            "balanced",
        ])
        .unwrap();
        assert!(args.live_insertion);
        assert_eq!(args.lookahead, Some(LiveInsertionLookahead::Balanced));
    }

    #[test]
    fn live_insertion_from_forwarded_argv() {
        assert!(live_insertion_from_argv([
            "handy",
            "--toggle-transcription",
            "--live-insertion",
        ]));
        assert!(live_insertion_from_argv([
            "handy",
            "--toggle-transcription",
            "--live-insertion",
            "--lookahead",
            "fast",
        ]));
        assert!(!live_insertion_from_argv(["handy", "--toggle-transcription"]));
        assert!(!live_insertion_from_argv([
            "handy",
            "--toggle-transcription",
            "--lookahead",
            "fast",
        ]));
    }

    #[test]
    fn quit_alone_is_accepted() {
        let args = parse(&["--quit"]).unwrap();
        assert!(args.quit);
    }

    #[test]
    fn quit_conflicts_with_actions() {
        assert!(parse(&["--quit", "--toggle-transcription"]).is_err());
        assert!(parse(&["--quit", "--cancel"]).is_err());
        assert!(parse(&["--quit", "--transcribe-file", "a.wav"]).is_err());
        assert!(parse(&["--quit", "--list-models"]).is_err());
        assert!(parse(&["--quit", "--model", "nemotron"]).is_err());
    }
}
