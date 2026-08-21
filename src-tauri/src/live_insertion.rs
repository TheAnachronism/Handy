//! Live Insertion policy: Committed Deltas, leftover on stop, nothing on Cancel.
//!
//! Pure decision logic. No microphone, STT, overlay, or OS typing.

use std::sync::Mutex;

/// Session options that affect stop leftover only, not mid-session Committed Deltas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LiveInsertionOptions {
    pub trailing_space: bool,
    pub auto_submit: bool,
    pub copy_clipboard: bool,
}

/// Commands the policy emits for a Dictation Session. Callers apply them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveInsertionCommand {
    /// Direct-type a Committed Delta during the Dictation Session.
    Type(String),
    /// Direct-type leftover text on a normal stop (finalized Tentative Transcript).
    TypeLeftover { text: String, trailing_space: bool },
    AutoSubmit,
    CopyTranscript(String),
    /// A Direct increment failed; keep the Dictation Session going.
    ErrorContinue,
    /// Live Insertion was requested but the model cannot stream: no live types.
    Inactive,
}

/// Consecutive overlay snapshots that must agree before typing past a frozen
/// library committed prefix (punctuation often stalls `committed` while the
/// next sentence sits in `tentative`).
const DISPLAY_AGREEMENT_N: usize = 3;

/// Tracks how much Committed Transcript has already been sent (typed or dropped).
pub struct LiveInsertionPolicy {
    live_requested: bool,
    model_streams: bool,
    options: LiveInsertionOptions,
    /// Byte length of typable transcript already accounted for.
    acked_len: usize,
    /// Recent `committed + tentative` snapshots for display-prefix agreement.
    display_history: Vec<String>,
}

impl LiveInsertionPolicy {
    pub fn start(
        live_requested: bool,
        model_streams: bool,
        options: LiveInsertionOptions,
    ) -> (Self, Vec<LiveInsertionCommand>) {
        let policy = Self {
            live_requested,
            model_streams,
            options,
            acked_len: 0,
            display_history: Vec::new(),
        };
        let commands = if live_requested && !model_streams {
            vec![LiveInsertionCommand::Inactive]
        } else {
            Vec::new()
        };
        (policy, commands)
    }

    pub fn live_active(&self) -> bool {
        self.live_requested && self.model_streams
    }

    /// Process a StreamTextEvent snapshot. Types library Committed growth, and
    /// if that prefix freezes, types a stable prefix of committed+tentative.
    pub fn on_stream_text(
        &mut self,
        committed: &str,
        tentative: &str,
    ) -> Vec<LiveInsertionCommand> {
        if !self.live_active() {
            return Vec::new();
        }
        let display = format!("{committed}{tentative}");
        self.display_history.push(display.clone());
        if self.display_history.len() > DISPLAY_AGREEMENT_N {
            self.display_history.remove(0);
        }

        let mut typable_end = 0;
        if display.starts_with(committed) {
            typable_end = committed.len();
        }
        let stable_end = stable_display_prefix_len(&self.display_history);
        if stable_end > typable_end {
            typable_end = stable_end;
        }
        if typable_end > display.len() {
            typable_end = display.len();
        }
        if !display.is_char_boundary(typable_end) {
            typable_end = utf8_floor(&display, typable_end);
        }
        if typable_end <= self.acked_len || !display.is_char_boundary(self.acked_len) {
            return Vec::new();
        }
        let delta = display[self.acked_len..typable_end].to_string();
        if delta.is_empty() {
            return Vec::new();
        }
        self.acked_len = typable_end;
        vec![LiveInsertionCommand::Type(delta)]
    }

    /// Normal stop: leftover is untyped finalized text; extras only here, not on deltas.
    pub fn stop(&mut self, finalized_text: &str) -> Vec<LiveInsertionCommand> {
        if !self.live_active() {
            return Vec::new();
        }

        let mut commands = Vec::new();
        let leftover = leftover_after_acked(finalized_text, self.acked_len);
        if !leftover.is_empty() || self.options.trailing_space {
            commands.push(LiveInsertionCommand::TypeLeftover {
                text: leftover,
                trailing_space: self.options.trailing_space,
            });
        }
        self.acked_len = finalized_text.len();

        if self.options.auto_submit {
            commands.push(LiveInsertionCommand::AutoSubmit);
        }
        if self.options.copy_clipboard {
            commands.push(LiveInsertionCommand::CopyTranscript(
                finalized_text.to_string(),
            ));
        }
        commands
    }

    /// Cancel: do not finalize leftover Tentative Transcript.
    pub fn cancel(&mut self) -> Vec<LiveInsertionCommand> {
        self.acked_len = 0;
        self.display_history.clear();
        Vec::new()
    }

    /// Drop a failed Direct increment; do not retry that Committed Delta.
    pub fn direct_failed(&mut self) -> Vec<LiveInsertionCommand> {
        if !self.live_active() {
            return Vec::new();
        }
        vec![LiveInsertionCommand::ErrorContinue]
    }
}

fn leftover_after_acked(finalized_text: &str, acked_len: usize) -> String {
    if acked_len >= finalized_text.len() || !finalized_text.is_char_boundary(acked_len) {
        String::new()
    } else {
        finalized_text[acked_len..].to_string()
    }
}

fn utf8_floor(s: &str, mut pos: usize) -> usize {
    if pos >= s.len() {
        return s.len();
    }
    while pos > 0 && !s.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

fn common_prefix_len(a: &str, b: &str) -> usize {
    let n = a.len().min(b.len());
    let mut i = 0;
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    while i < n && ab[i] == bb[i] {
        i += 1;
    }
    utf8_floor(a, i)
}

fn stable_display_prefix_len(history: &[String]) -> usize {
    if history.len() < DISPLAY_AGREEMENT_N {
        return 0;
    }
    let mut prefix_n = history[0].len();
    for text in history.iter().skip(1) {
        prefix_n = prefix_n.min(common_prefix_len(&history[0], text));
    }
    utf8_floor(&history[0], prefix_n)
}

/// Lookahead the stream worker should request. `Default` is the model's
/// accuracy-first StreamOptions (no family extension).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveLookahead {
    Default,
    /// Cache-aware (Nemotron): `att_context_right` in encoder frames.
    CacheAware { att_context_right: i32 },
    /// Chunked Unified: milliseconds; left stays the model default (large).
    ChunkedBuffered { chunk_ms: i32, right_ms: i32 },
}

/// User-facing Lookahead preset. Maps onto each family's training menu.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    serde::Serialize,
    serde::Deserialize,
    specta::Type,
    clap::ValueEnum,
)]
#[serde(rename_all = "snake_case")]
#[value(rename_all = "snake_case")]
pub enum LiveInsertionLookahead {
    /// ~0 ms (Nemotron right=0). Fastest typing; weakest punctuation/accuracy.
    Fastest,
    /// ~240 ms (Nemotron right=3). Default: shortest non-zero on the GGUF menu.
    #[default]
    Fast,
    /// ~480 ms (Nemotron right=6). Later commits; usually better punctuation.
    Balanced,
    /// ~1040 ms (Nemotron right=13). Closest to batch quality.
    Accurate,
}

impl LiveInsertionLookahead {
    /// Nemotron Streaming 3.5 GGUF menu: 13 6 3 0.
    pub fn nemotron_att_context_right(self) -> i32 {
        match self {
            Self::Fastest => 0,
            Self::Fast => 3,
            Self::Balanced => 6,
            Self::Accurate => 13,
        }
    }

    /// Unified EN: C in {1,2,7,13}, R in {0,1,2,3,4,7,13} (80 ms frames).
    pub fn unified_chunk_and_right_ms(self) -> (i32, i32) {
        match self {
            Self::Fastest => (80, 0),
            Self::Fast => (80, 80),
            Self::Balanced => (160, 160),
            Self::Accurate => (1040, 1040),
        }
    }
}

/// When Live Insertion is active, pick Lookahead for the loaded family and preset.
pub fn lookahead_for_live_insertion(
    live_insertion_active: bool,
    model_accepts_cache_aware: bool,
    model_accepts_chunked: bool,
    preset: LiveInsertionLookahead,
) -> LiveLookahead {
    if !live_insertion_active {
        return LiveLookahead::Default;
    }
    if model_accepts_cache_aware {
        return LiveLookahead::CacheAware {
            att_context_right: preset.nemotron_att_context_right(),
        };
    }
    if model_accepts_chunked {
        let (chunk_ms, right_ms) = preset.unified_chunk_and_right_ms();
        return LiveLookahead::ChunkedBuffered { chunk_ms, right_ms };
    }
    LiveLookahead::Default
}

/// Lookahead for one Plain Dictation start: CLI override wins when Live Insertion
/// is on; otherwise Settings. Live Insertion off or no family extension → Default.
pub fn lookahead_for_plain_dictation_start(
    live_insertion_active: bool,
    model_accepts_cache_aware: bool,
    model_accepts_chunked: bool,
    settings_preset: LiveInsertionLookahead,
    override_preset: Option<LiveInsertionLookahead>,
) -> LiveLookahead {
    let preset = override_preset.unwrap_or(settings_preset);
    lookahead_for_live_insertion(
        live_insertion_active,
        model_accepts_cache_aware,
        model_accepts_chunked,
        preset,
    )
}

/// Silence Feeding: noise frames go to the stream only while Live Insertion is active.
pub fn should_feed_silence_to_stream(live_insertion_active: bool) -> bool {
    live_insertion_active
}

/// CLI overrides for one Plain Dictation **start**. Do not persist.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SessionStartOverrides {
    pub lookahead: Option<LiveInsertionLookahead>,
    pub live_insertion: bool,
}

/// Whether Live Insertion should run for this Plain Dictation start.
/// Settings on or CLI `--live-insertion`; never for Post-Process. Streaming is
/// required for live typing; callers still start the policy when requested so
/// a non-streaming model can emit Inactive.
pub fn live_insertion_requested_for_plain_dictation_start(
    post_process: bool,
    settings_live_insertion: bool,
    cli_live_insertion: bool,
) -> bool {
    !post_process && (settings_live_insertion || cli_live_insertion)
}

/// One-shot CLI overrides for the next Plain Dictation **start**.
#[derive(Default)]
pub struct SessionLookaheadOverride {
    inner: Mutex<SessionStartOverrides>,
}

impl SessionLookaheadOverride {
    pub fn store(&self, value: SessionStartOverrides) {
        *self.inner.lock().unwrap_or_else(|e| e.into_inner()) = value;
    }

    pub fn take(&self) -> SessionStartOverrides {
        std::mem::take(&mut *self.inner.lock().unwrap_or_else(|e| e.into_inner()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    enum Step {
        Stream {
            committed: &'static str,
            tentative: &'static str,
        },
        Stop(&'static str),
        Cancel,
        DirectFailed,
    }

    struct Case {
        name: &'static str,
        live_requested: bool,
        model_streams: bool,
        options: LiveInsertionOptions,
        steps: &'static [Step],
        expected: Vec<LiveInsertionCommand>,
    }

    fn collect(
        live_requested: bool,
        model_streams: bool,
        options: LiveInsertionOptions,
        steps: &[Step],
    ) -> Vec<LiveInsertionCommand> {
        let (mut policy, mut out) =
            LiveInsertionPolicy::start(live_requested, model_streams, options);
        for step in steps {
            let mut cmds = match step {
                Step::Stream {
                    committed,
                    tentative,
                } => policy.on_stream_text(committed, tentative),
                Step::Stop(finalized) => policy.stop(finalized),
                Step::Cancel => policy.cancel(),
                Step::DirectFailed => policy.direct_failed(),
            };
            out.append(&mut cmds);
        }
        out
    }

    fn type_cmd(s: &str) -> LiveInsertionCommand {
        LiveInsertionCommand::Type(s.to_string())
    }

    fn leftover(s: &str, trailing_space: bool) -> LiveInsertionCommand {
        LiveInsertionCommand::TypeLeftover {
            text: s.to_string(),
            trailing_space,
        }
    }

    fn copy_cmd(s: &str) -> LiveInsertionCommand {
        LiveInsertionCommand::CopyTranscript(s.to_string())
    }

    #[test]
    fn live_insertion_policy_table() {
        let extras = LiveInsertionOptions {
            trailing_space: true,
            auto_submit: true,
            copy_clipboard: true,
        };
        let no_extras = LiveInsertionOptions::default();

        let cases = vec![
            Case {
                name: "two growing Committed Transcript snapshots emit two Committed Deltas and never re-type the prefix",
                live_requested: true,
                model_streams: true,
                options: no_extras,
                steps: &[
                    Step::Stream {
                        committed: "hello",
                        tentative: "",
                    },
                    Step::Stream {
                        committed: "hello world",
                        tentative: "",
                    },
                ],
                expected: vec![type_cmd("hello"), type_cmd(" world")],
            },
            Case {
                name: "Tentative Transcript growth alone emits no type command during a Dictation Session",
                live_requested: true,
                model_streams: true,
                options: no_extras,
                steps: &[
                    Step::Stream {
                        committed: "",
                        tentative: "hello",
                    },
                    Step::Stream {
                        committed: "",
                        tentative: "hello there",
                    },
                ],
                expected: vec![],
            },
            Case {
                name: "stop after remaining Tentative Transcript emits leftover equal to untyped finalized text",
                live_requested: true,
                model_streams: true,
                options: no_extras,
                steps: &[
                    Step::Stream {
                        committed: "hello",
                        tentative: " there",
                    },
                    Step::Stop("hello there"),
                ],
                expected: vec![type_cmd("hello"), leftover(" there", false)],
            },
            Case {
                name: "stop after all Committed Transcript emits no leftover",
                live_requested: true,
                model_streams: true,
                options: no_extras,
                steps: &[
                    Step::Stream {
                        committed: "hello",
                        tentative: "",
                    },
                    Step::Stop("hello"),
                ],
                expected: vec![type_cmd("hello")],
            },
            Case {
                name: "stop after all Committed Transcript still types trailing space when that setting is on",
                live_requested: true,
                model_streams: true,
                options: extras,
                steps: &[
                    Step::Stream {
                        committed: "hello",
                        tentative: "",
                    },
                    Step::Stop("hello"),
                ],
                expected: vec![
                    type_cmd("hello"),
                    leftover("", true),
                    LiveInsertionCommand::AutoSubmit,
                    copy_cmd("hello"),
                ],
            },
            Case {
                name: "Cancel of a Dictation Session emits no leftover type command",
                live_requested: true,
                model_streams: true,
                options: extras,
                steps: &[
                    Step::Stream {
                        committed: "hello",
                        tentative: " there",
                    },
                    Step::Cancel,
                ],
                expected: vec![type_cmd("hello")],
            },
            Case {
                name: "after Direct failure the next commit types only new Committed Delta growth",
                live_requested: true,
                model_streams: true,
                options: no_extras,
                steps: &[
                    Step::Stream {
                        committed: "hello",
                        tentative: "",
                    },
                    Step::DirectFailed,
                    Step::Stream {
                        committed: "hello world",
                        tentative: "",
                    },
                ],
                expected: vec![
                    type_cmd("hello"),
                    LiveInsertionCommand::ErrorContinue,
                    type_cmd(" world"),
                ],
            },
            Case {
                name: "auto-submit, trailing space, and copy-full-transcript occur only on stop leftover, not on mid-session Committed Deltas",
                live_requested: true,
                model_streams: true,
                options: extras,
                steps: &[
                    Step::Stream {
                        committed: "hello",
                        tentative: " there",
                    },
                    Step::Stream {
                        committed: "hello",
                        tentative: " there friend",
                    },
                    Step::Stop("hello there friend"),
                ],
                expected: vec![
                    type_cmd("hello"),
                    leftover(" there friend", true),
                    LiveInsertionCommand::AutoSubmit,
                    copy_cmd("hello there friend"),
                ],
            },
            Case {
                name: "when Committed Transcript freezes after a period, a stable display prefix still types the next sentence",
                live_requested: true,
                model_streams: true,
                options: no_extras,
                steps: &[
                    Step::Stream {
                        committed: "Hello.",
                        tentative: " Next",
                    },
                    Step::Stream {
                        committed: "Hello.",
                        tentative: " Next sentence",
                    },
                    Step::Stream {
                        committed: "Hello.",
                        tentative: " Next sentence here",
                    },
                ],
                expected: vec![type_cmd("Hello."), type_cmd(" Next")],
            },
            Case {
                name: "Live Insertion requested but model cannot stream yields Inactive and no live type commands",
                live_requested: true,
                model_streams: false,
                options: extras,
                steps: &[
                    Step::Stream {
                        committed: "hello",
                        tentative: " there",
                    },
                    Step::Stop("hello there"),
                ],
                expected: vec![LiveInsertionCommand::Inactive],
            },
        ];

        for c in cases {
            let got = collect(c.live_requested, c.model_streams, c.options, c.steps);
            assert_eq!(got, c.expected, "{}", c.name);
        }
    }

    #[test]
    fn live_insertion_off_keeps_accuracy_first_lookahead() {
        assert_eq!(
            lookahead_for_live_insertion(false, true, false, LiveInsertionLookahead::Balanced),
            LiveLookahead::Default
        );
        assert_eq!(
            lookahead_for_live_insertion(false, false, true, LiveInsertionLookahead::Accurate),
            LiveLookahead::Default
        );
    }

    #[test]
    fn live_insertion_on_nemotron_maps_menu_presets() {
        assert_eq!(
            lookahead_for_live_insertion(true, true, false, LiveInsertionLookahead::Fast),
            LiveLookahead::CacheAware {
                att_context_right: 3
            }
        );
        assert_eq!(
            lookahead_for_live_insertion(true, true, false, LiveInsertionLookahead::Balanced),
            LiveLookahead::CacheAware {
                att_context_right: 6
            }
        );
        assert_eq!(
            lookahead_for_live_insertion(true, true, false, LiveInsertionLookahead::Fastest),
            LiveLookahead::CacheAware {
                att_context_right: 0
            }
        );
        assert_eq!(
            lookahead_for_live_insertion(true, true, false, LiveInsertionLookahead::Accurate),
            LiveLookahead::CacheAware {
                att_context_right: 13
            }
        );
    }

    #[test]
    fn live_insertion_on_unified_maps_on_menu_chunk_and_right() {
        assert_eq!(
            lookahead_for_live_insertion(true, false, true, LiveInsertionLookahead::Fastest),
            LiveLookahead::ChunkedBuffered {
                chunk_ms: 80,
                right_ms: 0
            }
        );
        assert_eq!(
            lookahead_for_live_insertion(true, false, true, LiveInsertionLookahead::Fast),
            LiveLookahead::ChunkedBuffered {
                chunk_ms: 80,
                right_ms: 80
            }
        );
        assert_eq!(
            lookahead_for_live_insertion(true, false, true, LiveInsertionLookahead::Balanced),
            LiveLookahead::ChunkedBuffered {
                chunk_ms: 160,
                right_ms: 160
            }
        );
        assert_eq!(
            lookahead_for_live_insertion(true, false, true, LiveInsertionLookahead::Accurate),
            LiveLookahead::ChunkedBuffered {
                chunk_ms: 1040,
                right_ms: 1040
            }
        );
    }

    #[test]
    fn live_insertion_on_without_family_extension_stays_default() {
        assert_eq!(
            lookahead_for_live_insertion(true, false, false, LiveInsertionLookahead::Fast),
            LiveLookahead::Default
        );
    }

    #[test]
    fn silence_feeding_only_when_live_insertion_is_active() {
        assert!(should_feed_silence_to_stream(true));
        assert!(!should_feed_silence_to_stream(false));
    }

    #[test]
    fn lookahead_override_is_ignored_when_live_insertion_is_off() {
        assert_eq!(
            lookahead_for_plain_dictation_start(
                false,
                true,
                false,
                LiveInsertionLookahead::Fast,
                Some(LiveInsertionLookahead::Balanced),
            ),
            LiveLookahead::Default
        );
    }

    #[test]
    fn lookahead_override_wins_on_nemotron_when_live_insertion_is_on() {
        assert_eq!(
            lookahead_for_plain_dictation_start(
                true,
                true,
                false,
                LiveInsertionLookahead::Fast,
                Some(LiveInsertionLookahead::Balanced),
            ),
            LiveLookahead::CacheAware {
                att_context_right: 6
            }
        );
    }

    #[test]
    fn settings_lookahead_used_when_override_absent() {
        assert_eq!(
            lookahead_for_plain_dictation_start(
                true,
                true,
                false,
                LiveInsertionLookahead::Fast,
                None,
            ),
            LiveLookahead::CacheAware {
                att_context_right: 3
            }
        );
    }

    #[test]
    fn cli_live_insertion_requests_live_when_settings_off() {
        assert!(live_insertion_requested_for_plain_dictation_start(
            false, false, true
        ));
        assert!(live_insertion_requested_for_plain_dictation_start(
            false, true, false
        ));
        assert!(!live_insertion_requested_for_plain_dictation_start(
            false, false, false
        ));
        assert!(!live_insertion_requested_for_plain_dictation_start(
            true, true, true
        ));
    }

    #[test]
    fn lookahead_override_without_family_extension_stays_default() {
        assert_eq!(
            lookahead_for_plain_dictation_start(
                true,
                false,
                false,
                LiveInsertionLookahead::Fast,
                Some(LiveInsertionLookahead::Accurate),
            ),
            LiveLookahead::Default
        );
    }
}
