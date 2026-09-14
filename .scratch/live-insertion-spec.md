## Problem Statement

When I use a speech-to-text model that can stream, Handy already shows Committed Transcript and Tentative Transcript on the overlay while I talk. The Target Application only receives text after I stop the Dictation Session (Batch Insertion). I want an extra mode so Plain Dictation also types into the focused app as I speak.

## Solution

Add **Live Insertion**: an opt-in setting on the existing Plain Dictation hotkey. While a Dictation Session is open and the loaded model can stream, Handy Direct-types each new Committed Delta into whatever currently has keyboard focus. Tentative Transcript stays overlay-only. A normal stop types only leftover (finalized tentative). Cancel leaves already-typed text and does not insert leftover. Post-Process Dictation stays Batch Insertion only.

## User Stories

1. As a dictation user with a streaming model, I want Committed Transcript to appear in the Target Application while I talk, so I do not wait until I toggle dictation off.
2. As a dictation user, I want Tentative Transcript to stay on the overlay only, so the Target Application never has words that the model later revises.
3. As a dictation user, I want leftover Tentative Transcript finalized and typed once when I stop, so the last words are not lost.
4. As a dictation user, I want the same Plain Dictation hotkey as today, so I do not learn a third shortcut.
5. As a dictation user, I want Live Insertion to be a setting I turn on, so existing Batch Insertion behavior stays the default.
6. As a dictation user, I want that setting visible only when the current model streams, so I am not offered a mode the model cannot do.
7. As a dictation user, I want the setting to stay on if I switch to a non-streaming model, so I do not have to re-enable it when I switch back.
8. As a dictation user, I want Plain Dictation to fall back to Batch Insertion when the flag is on but the model cannot stream, so dictation still works.
9. As a dictation user, I want a one-shot notice in that fallback case, so I understand why text waited until stop.
10. As a user of Post-Process Dictation, I want that hotkey to keep Batch Insertion (full transcript, then OpenCC/LLM, then one paste), so live typing never skips post-process.
11. As a user whose Paste Method is a clipboard chord, I want Live Insertion to still Direct-type, so many small clipboard save/restore cycles do not happen.
12. As a user whose Paste Method is None, I want Live Insertion (when on) to still Direct-type, so "no paste" stays a Batch Insertion concern.
13. As a user of auto-submit, I want Enter (or Ctrl/Cmd+Enter) once at session end after leftover, so I do not submit mid-sentence.
14. As a user of trailing space, I want a space only on that last leftover insertion if the setting is on, so increments are not double-spaced.
15. As a user of Copy to Clipboard, I want the full model transcript copied once at stop, so the clipboard is not thrashed per delta.
16. As a push-to-talk user, I want Live Insertion while I hold the hotkey, so PTT is not second-class.
17. As a toggle user, I want Live Insertion for the whole open session, including pauses.
18. As a user who clicks another field mid-session, I want the next Committed Delta to land at the new caret, so I can retarget without stopping.
19. As a user who types in the Target Application between deltas, I want the next delta at the caret, so I can edit while speaking.
20. As a user with overlay style None, I want Live Insertion still available, so overlay is independent preview.
21. As a user with overlay style Live or Minimal, I want overlay updates unchanged (committed + tentative), so preview still works.
22. As a user, I want the overlay never to take keyboard focus, so Handy is never the Target Application.
23. As a user who pauses speaking, I want the Dictation Session to stay open, so silence does not auto-stop or auto-finalize tentative text.
24. As a user who Cancels (tray, overlay cancel, Escape while recording, CLI), I want capture to stop, leftover tentative discarded, and already-typed text left in place, so Cancel is not an undo of the Target Application.
25. As a user who Cancels, I want no further Direct typing and no history row, matching today's Cancel.
26. As a user whose Direct typing of a delta fails, I want that increment dropped, an error shown, and the session to continue, so one OS failure does not abort dictation.
27. As a user in that failure case, I want later deltas still attempted, so I can keep talking after a blip.
28. As a user who stops normally, I want one history row with the full model transcript even if some Direct attempts failed, so history is what the model said, not what the OS typed.
29. As a user who Cancels, I want no history row, matching today.
30. As a user on a non-streaming model with the flag off, I want unchanged Batch Insertion.
31. As a user starting Plain Dictation with Live Insertion on and a streaming model, I want each Committed Delta typed as soon as the model emits it, so latency stays low.
32. As a user stopping after several deltas, I want Handy never to re-type the whole session, so text is not duplicated.
33. As a user with clipboard paste delays configured, I want those delays not applied to live Direct deltas, so live typing is not paced like a one-shot paste.
34. As a settings user, I want the Live Insertion control near overlay / paste in General, so I can find it with related insertion settings.
35. As a Linux user with a preferred typing tool, I want live Direct typing to use the same Direct path as Batch Insertion Direct.
36. As a user who enables Live Insertion, I want existing models that advertise streaming (catalog streaming capability) to qualify, so I do not need a new engine.
37. As a user who uses CLI/signal toggle for Plain Dictation, I want the same Live Insertion rules as the hotkey.
38. As a user who uses tray Cancel during Live Insertion, I want the same Cancel semantics as Escape.
39. As a developer implementing this, I want glossary terms (Dictation Session, Live Insertion, Committed Delta, Target Application, Cancel) used in tests, so language stays aligned with CONTEXT.md and ADRs 0001-0005.

## Implementation Decisions

- Honor ADRs 0001-0005: Direct-only Live Insertion; committed-only in the Target Application; leftover-only on stop; current focus; Cancel leaves typed text; drop failed increments; immediate deltas; flag hidden unless model streams; flag persists across model switches; overlay independent; Post-Process Dictation batch-only.
- Add a persisted boolean setting, default off. Show the control only when the currently selected model reports streaming support. Keep the stored value when switching models. If the flag is on and the model cannot stream at Plain Dictation start, use Batch Insertion and show a one-shot notice.
- Introduce one Live Insertion policy object (the test seam). It owns insertion cursor for one Dictation Session. Inputs: StreamTextEvent snapshots (committed, tentative) and session commands (start with capability/flags, stop, cancel, report Direct failure). Outputs: insertion commands (type this Committed Delta, type leftover, auto-submit, copy full transcript, no-op on cancel leftover). It does not call the microphone, STT worker, overlay emitter, or OS typing.
- Wire the policy from the existing stream-text emission path during an open Plain Dictation session when Live Insertion is active. Do not invent a second stream protocol.
- On each new snapshot, compute Committed Delta as the prefix growth of committed since the last successfully typed length. Type that slice immediately via the existing Direct typing implementation (same Linux tool / enigo path as Batch Insertion Direct).
- Do not apply clipboard paste methods, clipboard save/restore, or paste delays to live deltas.
- On normal stop: finalize the stream as today; type only the suffix not yet typed (finalized leftover); then apply trailing space if set; then auto-submit if set; then copy full model transcript to clipboard if Copy to Clipboard is on. Do not paste the whole transcript again. Do not run Post-Process on this path (Post-Process Dictation never enters Live Insertion).
- On Cancel: tear down capture as today; do not finalize leftover into Direct typing; do not copy clipboard; do not history. Leave Target Application text as-is.
- On Direct failure for a delta: emit the existing paste-error (or equivalent user-visible error); do not retry that slice; advance the cursor as if typed so later deltas are based on model committed growth (dropped text is lost unless re-spoken). Session continues.
- History: one row at a normal session end with the full model transcript; insertion success does not gate the row; Cancel writes none. Keep today's WAV-saved gating if that already decides whether a row exists; do not invent a second history store.
- Overlay: keep emitting StreamTextEvent as today; do not force overlay style; overlay remains non-focusable.
- Push-to-talk and toggle both run Live Insertion while Recording. Document modifier-bleed as a user workaround (toggle / non-modifier hotkey), not as a code special case in this spec.
- Settings/UI: General settings next to overlay/paste; hide when current model does not stream; copy explains fallback plus one-shot notice.

## Testing Decisions

- Test only the Live Insertion policy's external behavior: given a sequence of snapshots and lifecycle commands, assert the sequence of emitted insertion commands (text slices, leftover vs none, auto-submit, clipboard copy, cancel, errors). Do not assert overlay internals, audio, or OS key injection.
- Do not mock the STT engine. Feed synthetic StreamTextEvent values.
- Good tests: two growing committed snapshots produce two deltas and never re-type the prefix; tentative-only growth produces no type command; stop after tentative produces leftover equal to remaining untyped text; stop after all committed produces no leftover (or empty); cancel produces no leftover type; Direct failure then another commit types only the new growth; auto-submit and trailing space only on the stop leftover command, not on mid-session deltas; copy-clipboard payload is the full assembled transcript once at stop; start with flag on and model not streaming yields no live type commands (caller uses Batch Insertion).
- Prior art: coordinator unit tests around push-to-talk classify/lifecycle in the transcription coordinator; prefer the same style of table-driven lifecycle tests for the policy. There is little existing clipboard unit coverage; do not add OS-level typing tests in this spec.
- Settings visibility can be asserted in the policy start flags or a tiny pure helper if the UI cannot be tested without a harness; do not add a second behavioral seam for insertion itself.

## Out of Scope

- Rewriting or backspacing Tentative Transcript in the Target Application.
- Live Insertion for Post-Process Dictation.
- Coalescing deltas, silence auto-stop, or silence auto-finalize.
- Changing Paste Method for Batch Insertion.
- New hotkeys.
- Making the overlay click-through or changing overlay focus policy beyond "must not become Target Application" (already non-focusable).
- Retry/buffer of failed Direct increments.
- Cloud STT; streaming remains local models that already advertise streaming.
- Prefactor of clipboard architecture beyond calling existing Direct typing.

## Further Notes

- Domain language: CONTEXT.md. Decisions: docs/adr/0001 through 0005.
- Today the overlay already streams; the gap is insertion. paste() runs once on the main thread after finalize_stream() or batch transcribe().
- Push-to-talk while Direct-typing can inject modifiers; called out in ADR 0003, not solved in this spec.
