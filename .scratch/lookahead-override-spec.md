## Problem Statement

I already bind `handy --toggle-transcription` as a desktop keybind. Live Insertion’s Lookahead (Fastest / Fast / Balanced / Accurate) is only a Settings dropdown. I cannot start **one** Plain Dictation session with a different Lookahead from another keybind without changing the dropdown and changing it back.

## Solution

Add `--lookahead fastest|fast|balanced|accurate` as a companion to `--toggle-transcription`. When that invocation **starts** Plain Dictation, Live Insertion is **on**, and the model can stream, that session uses the Lookahead Override. The Settings dropdown is unchanged. The next start without `--lookahead` uses Settings again.

## User Stories

1. As a toggle-keybind user, I want `handy --toggle-transcription --lookahead balanced`, so that one press is Balanced Live Insertion without opening Settings.
2. As a user with several OS keybinds, I want one command per preset (`fastest`, `fast`, `balanced`, `accurate`), so that I can pick snappiness vs punctuation at the keyboard.
3. As a user of the in-app transcribe hotkey, I want that hotkey to keep the Settings Lookahead, so that the dropdown still means something.
4. As a tray user, I want tray start to keep the Settings Lookahead, so that the CLI is the only override path.
5. As a SIGUSR2 user, I want signals to keep the Settings Lookahead, so that I do not need a new signal encoding.
6. As a user who starts Handy with `handy --toggle-transcription --lookahead accurate` when it is not running, I want that first Dictation Session to use Accurate after the app is up, so that the same keybind works cold.
7. As a user who forwards the same command into an already-running instance, I want the Lookahead Override to apply if that invocation **starts** a session, so that second-instance CLI matches first-launch.
8. As a user who sends `--toggle-transcription --lookahead accurate` while a Dictation Session is already open, I want the session to stop and `--lookahead` to be ignored, so that stop stays stop.
9. As a user with Live Insertion **off**, I want `--lookahead` ignored and Batch Insertion at stop, so that the CLI cannot silently turn Live Insertion on.
10. As a user of a non-streaming model, I want `--lookahead` ignored and Batch Insertion, so that stream begin is not requested with an unused Lookahead.
11. As a Post-Process Dictation user, I want `--toggle-post-process --lookahead balanced` to still be Post-Process Batch Insertion, so that Lookahead never applies to that path.
12. As a user who runs `handy --lookahead balanced` with no start flag, I want a usage error and a non-zero exit, so that I notice the missing `--toggle-transcription`.
13. As a user who passes `--lookahead snappy` or `--lookahead 1`, I want a usage error and a non-zero exit, so that off-menu values never reach stream begin.
14. As a user who omits `--lookahead`, I want Settings Lookahead (default Fast), so that existing `handy --toggle-transcription` binds stay unchanged.
15. As a user watching the overlay, I want overlay commit timing to follow the same Lookahead as Direct typing for that session, so that preview and the Target Application stay in the same cadence.
16. As a user of the Advanced dropdown, I want it to keep showing the stored preset during an overridden session, so that I do not think Settings were written.
17. As a user who stops an overridden session and then uses the in-app hotkey, I want Settings Lookahead again, so that the override lasts one Dictation Session only.
18. As a user who Cancels an overridden session, I want the next start without `--lookahead` to use Settings, so that Cancel also clears the override.
19. As a user who does not want extra chrome, I want a log line when a Lookahead Override is honored, not a toast, so that keybinds stay quiet.
20. As a user who reads `--help`, I want `--lookahead` documented next to `--toggle-transcription`, with the four preset names.
21. As a user of `--cancel`, I want `--lookahead` unused on cancel, so that cancel stays cancel.
22. As a user of `--transcribe-file` / `--list-devices` / `--list-models`, I want `--lookahead` unused (or clap-rejected if we require a start flag), so that headless batch is unchanged.
23. As a user who later wants a start-only `--transcribe` command, I want this slice **not** to add it, so that toggle + `--lookahead` is enough for multiple keybinds now.
24. As a user who toggles Live Insertion in Settings after binding `--lookahead accurate`, I want a start with `--lookahead` to still **ignore** the override while Live Insertion is off, so that the Advanced toggle remains the master switch.
25. As a user of Silence Feeding, I want it still on whenever Live Insertion is on for that session, including overridden Lookahead, so that pauses can still commit.
26. As a user of Tentative Transcript, I want typing still to follow committed-enough text only (including frozen-commit recovery), so that `--lookahead` does not type raw tentative.
27. As a user who mistypes `--live-insertion-lookahead`, I want clap not to accept that alias in this slice, so that the flag name stays `--lookahead`.
28. As a user whose model cannot stream, I want the existing one-shot fallback notice behavior unchanged, so that Lookahead Override does not invent a second notice.

## Implementation Decisions

- Flag: `--lookahead` with values `fastest` | `fast` | `balanced` | `accurate` (same snake_case as the Settings enum). Requires a start flag (`--toggle-transcription` or `--toggle-post-process`) via clap so a lone `--lookahead` is a usage error. The value is applied only on a Plain Dictation **start**. Combined with `--toggle-post-process` the session still starts; Lookahead is ignored (not a clap error). Invalid values are a usage error (non-zero exit) **in the process that parses argv**.
- Second instance: single-instance forwarding must parse `--lookahead` from forwarded args (not only detect `--toggle-transcription`) and pass the optional override into the same Plain Dictation start path as the in-app hotkey, **only if this invocation starts**.
- First process start: if argv includes `--toggle-transcription` and a valid `--lookahead`, after core init start one Plain Dictation session with that override (today `--toggle-transcription` on a cold start is forwarded only when a second instance exists; cold start must honor both flags).
- Resolution: when starting Plain Dictation, effective Lookahead is: if Live Insertion is on **and** the model can stream **and** a Lookahead Override is present for this start → override; else if Live Insertion is on and the model can stream → Settings preset; else Default (accuracy-first / no family extension).
- Consume the override at **start**; ignore it on stop of an already-open session. Do not persist. Do not mutate the Settings store or the dropdown.
- Post-Process start (`--toggle-post-process`) ignores `--lookahead` even if clap allows both flags together (ignore, do not error).
- Overlay uses the same `StreamOptions` Lookahead as Live Insertion typing (one stream).
- Log at info when an override is applied; log at debug/info when ignored (Live Insertion off, non-streaming, stop, post-process).
- No new start-only `--transcribe` command. No Unix-signal payload. No Settings UI change.
- Respect ADRs 0002–0008; do not close parent GitHub #1.

## Testing Decisions

- Tests describe external behavior (parse outcomes; which Lookahead a Plain Dictation **start** resolves to), not clap internals or stream-worker wiring.
- **Seam 1 — CLI parse:** `--lookahead` without a start flag fails; invalid values fail; `fastest|fast|balanced|accurate` with `--toggle-transcription` succeed. Prior art: clap `CliArgs` definitions.
- **Seam 2 — Lookahead resolution:** extend the existing `lookahead_for_live_insertion` (or a thin wrapper used by the stream start) so a session override + Live Insertion on + cache-aware/chunked maps to the same family Lookahead as the Settings presets; override is ignored when Live Insertion is off or neither family extension applies. Prior art: `live_insertion` unit table for presets.
- Do not add end-to-end typing tests, overlay tests, or transcribe-cpp stream begin tests in this spec.

## Out of Scope

- Start-only `--transcribe` command
- Raw milliseconds / frame counts / off-menu `att_context_right`
- Turning Live Insertion on from the CLI
- In-app extra hotkeys, tray override, SIGUSR encoding
- Changing the Settings dropdown during an override
- User-visible toast
- Post-Process Live Insertion
- Closing GitHub #1
- Unrelated `AGENTS.md` / scratch agent docs

## Further Notes

- Glossary: **Lookahead Override**. ADR 0008.
- Parent Live Insertion work remains GitHub #1.
- README desktop-keybind examples should show `--lookahead` as optional.
