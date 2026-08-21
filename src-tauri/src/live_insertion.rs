//! Live Insertion policy: Committed Deltas, leftover on stop, nothing on Cancel.
//!
//! Pure decision logic. No microphone, STT, overlay, or OS typing.

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

/// Tracks how much Committed Transcript has already been sent (typed or dropped).
pub struct LiveInsertionPolicy {
    live_requested: bool,
    model_streams: bool,
    options: LiveInsertionOptions,
    /// Byte length of Committed Transcript already accounted for.
    acked_len: usize,
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

    /// Process a StreamTextEvent snapshot. Types only new Committed Delta growth.
    pub fn on_stream_text(
        &mut self,
        committed: &str,
        _tentative: &str,
    ) -> Vec<LiveInsertionCommand> {
        if !self.live_active() {
            return Vec::new();
        }
        if committed.len() <= self.acked_len {
            return Vec::new();
        }
        if !committed.is_char_boundary(self.acked_len) {
            return Vec::new();
        }
        let delta = committed[self.acked_len..].to_string();
        if delta.is_empty() {
            return Vec::new();
        }
        self.acked_len = committed.len();
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

/// Shortest non-zero Lookahead on Nemotron Streaming 3.5 GGUF's training menu
/// (`13 6 3 0`). ~240 ms (3 × 80 ms encoder frames). Not 1 (not on this menu)
/// and not 0 (worse WER).
const NEMOTRON_LIVE_ATT_CONTEXT_RIGHT: i32 = 3;
/// Smallest on-menu Unified chunk and right (80 ms encoder frames).
/// parakeet-unified-en menu: L∈{70}, C∈{1,2,7,13}, R∈{0,1,2,3,4,7,13}.
const UNIFIED_LIVE_CHUNK_MS: i32 = 80;
const UNIFIED_LIVE_RIGHT_MS: i32 = 0;

/// When Live Insertion is active, pick a short Lookahead for the loaded family.
pub fn lookahead_for_live_insertion(
    live_insertion_active: bool,
    model_accepts_cache_aware: bool,
    model_accepts_chunked: bool,
) -> LiveLookahead {
    if !live_insertion_active {
        return LiveLookahead::Default;
    }
    if model_accepts_cache_aware {
        return LiveLookahead::CacheAware {
            att_context_right: NEMOTRON_LIVE_ATT_CONTEXT_RIGHT,
        };
    }
    if model_accepts_chunked {
        return LiveLookahead::ChunkedBuffered {
            chunk_ms: UNIFIED_LIVE_CHUNK_MS,
            right_ms: UNIFIED_LIVE_RIGHT_MS,
        };
    }
    LiveLookahead::Default
}

/// Silence Feeding: noise frames go to the stream only while Live Insertion is active.
pub fn should_feed_silence_to_stream(live_insertion_active: bool) -> bool {
    live_insertion_active
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
            lookahead_for_live_insertion(false, true, false),
            LiveLookahead::Default
        );
        assert_eq!(
            lookahead_for_live_insertion(false, false, true),
            LiveLookahead::Default
        );
    }

    #[test]
    fn live_insertion_on_nemotron_uses_shortest_nonzero_menu_lookahead() {
        assert_eq!(
            lookahead_for_live_insertion(true, true, false),
            LiveLookahead::CacheAware {
                att_context_right: 3
            }
        );
    }

    #[test]
    fn live_insertion_on_unified_uses_smallest_on_menu_chunk_and_right() {
        assert_eq!(
            lookahead_for_live_insertion(true, false, true),
            LiveLookahead::ChunkedBuffered {
                chunk_ms: 80,
                right_ms: 0
            }
        );
    }

    #[test]
    fn live_insertion_on_without_family_extension_stays_default() {
        assert_eq!(
            lookahead_for_live_insertion(true, false, false),
            LiveLookahead::Default
        );
    }

    #[test]
    fn silence_feeding_only_when_live_insertion_is_active() {
        assert!(should_feed_silence_to_stream(true));
        assert!(!should_feed_silence_to_stream(false));
    }
}
