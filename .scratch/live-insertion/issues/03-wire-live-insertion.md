## Parent

#1 Live Insertion: type Committed Transcript during Plain Dictation

## What to build

Plain Dictation with Live Insertion on and a streaming model types each Committed Delta into the Target Application as the user speaks, using Direct typing. Tentative Transcript stays overlay-only. A normal stop types only leftover (finalized tentative) and must not paste the whole session again. Cancel leaves already-typed text and does not type leftover. Post-Process Dictation stays Batch Insertion. The setting persists, default off. Push-to-talk and toggle both apply. Overlay style is unchanged and must not take focus.

## Acceptance criteria

- [ ] Persisted Live Insertion flag exists, default off; Batch Insertion is unchanged when the flag is off
- [ ] With flag on and a streaming model, each new Committed Delta is Direct-typed immediately during the open Dictation Session
- [ ] Tentative Transcript is not typed into the Target Application
- [ ] Normal stop types only leftover not already sent; it does not re-type the full transcript
- [ ] Cancel stops capture, types no leftover, writes no history, and does not undo already-typed text
- [ ] Post-Process Dictation never enters Live Insertion
- [ ] Paste Method (including None and clipboard chords) is ignored for live deltas and leftover; Direct typing is used
- [ ] Paste delays are not applied to live Direct deltas
- [ ] CLI/signal toggle for Plain Dictation follows the same rules as the hotkey

## Blocked by

- #3 Prefactor: Direct typing without paste wrapping
- #2 Live Insertion policy: Committed Deltas, leftover, Cancel
