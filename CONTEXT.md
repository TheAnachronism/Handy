# Handy

Handy is a local speech-to-text dictation app: the user starts a Dictation Session, speech is transcribed, and text is inserted into the currently focused application.

## Language

**Dictation Session**:
A period of microphone capture bounded by starting and stopping dictation (hotkey, CLI, signal, or tray).
_Avoid_: recording, transcription job, utterance (unless referring to a single spoken span)

**Batch Insertion**:
Text is assembled once after the Dictation Session ends, then inserted into the focused application.
_Avoid_: delayed paste, end-of-stream write

**Live Insertion**:
Committed Transcript is inserted into the focused application while the Dictation Session is still open.
_Avoid_: active streaming, streaming paste, real-time typing (unless describing keystroke injection)

**Committed Transcript**:
Speech-to-text output the model will not revise later in this session.
_Avoid_: final text, stable partial (except in overlay UI copy)

**Committed Delta**:
The slice of Committed Transcript that has not yet been sent to Live Insertion. Each delta is typed as soon as the model emits it.
_Avoid_: partial commit, stream chunk

**Tentative Transcript**:
Speech-to-text output that may still change before it becomes Committed Transcript. It is overlay-only until it is committed or the session ends and remaining tentative text is finalized.
_Avoid_: partial, hypothesis, streaming text (too broad)

**Plain Dictation**:
Dictation started with the transcribe hotkey, without the post-process pipeline.
_Avoid_: normal dictation, default transcribe

**Post-Process Dictation**:
Dictation started with the post-process hotkey; OpenCC and/or an LLM run on the full transcript after the session ends, then a single insertion.
_Avoid_: enhanced transcribe, AI dictation

**Target Application**:
The application and text field that should receive inserted transcript. For Live Insertion this is whatever currently has keyboard focus, not Handy itself.
_Avoid_: paste target, focused window (unless talking about OS focus)

**Cancel**:
Aborting a Dictation Session without treating it as a normal stop. Capture ends, leftover Tentative Transcript is discarded, and Handy does not insert further text. Already-inserted Committed Transcript is left in the Target Application.
_Avoid_: undo dictation, discard session

**Lookahead**:
Extra audio a streaming model waits for before promoting Tentative Transcript to Committed Transcript. Live Insertion uses a shorter Lookahead than Batch Insertion so commits happen sooner; accuracy and punctuation can drop. A setting next to Live Insertion picks Fastest / Fast / Balanced / Accurate on the loaded model's training menu (Nemotron Streaming 3.5 GGUF: 0, 3, 6, or 13).
_Avoid_: right context, att_context (unless naming the model knob)

**Lookahead Override**:
A CLI `--lookahead` value that replaces the Lookahead setting for one Plain Dictation start. It does not persist, does not change the Settings dropdown, and does not apply if Live Insertion is off, the model cannot stream, or the invocation stops a session.
_Avoid_: latency flag, att_context CLI

**Silence Feeding**:
During Live Insertion, every microphone frame — speech and silence — is given to the streaming model for the whole Dictation Session so Lookahead can finish without the user stopping. Silence still does not auto-stop the session or auto-finalize Tentative Transcript.
_Avoid_: VAD bypass, always-on stream, endpointing
