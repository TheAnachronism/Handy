# CLI `--live-insertion` enables Live Insertion for one Dictation Session start

OS keybinds already start Plain Dictation with `handy --toggle-transcription`. Settings Live Insertion can stay off as the default. A companion `--live-insertion` flag enables Live Insertion for that invocation only, so a second keybind can type Committed Transcript while speaking without changing the Settings toggle.

The override applies only when that invocation **starts** a Dictation Session, only if the model can stream, and only for that session. It is ignored on stop, on Post-Process Dictation, and when the model cannot stream. It does not persist and does not change the Settings toggle. `--live-insertion` without a start flag is a usage error. `--lookahead` still applies only while Live Insertion is active for that start (Settings on, or this flag). In-app hotkeys, tray, and Unix signals keep the Settings flag.
