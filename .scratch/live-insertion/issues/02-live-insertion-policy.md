## Parent

#1 Live Insertion: type Committed Transcript during Plain Dictation

## What to build

A Live Insertion policy that, given StreamTextEvent snapshots and session commands, emits insertion commands only. No microphone, STT, overlay, or OS typing. Tests lock the rules from ADRs 0001-0005: Committed Deltas only, leftover on stop, nothing on Cancel, drop failed increments, no live commands when the model cannot stream.

## Acceptance criteria

- [ ] Two growing committed snapshots emit two Committed Deltas and never re-type the prefix
- [ ] Tentative-only growth emits no type command
- [ ] Stop after remaining tentative emits leftover equal to untyped finalized text; stop after all committed emits no leftover (or empty)
- [ ] Cancel emits no leftover type command
- [ ] After a reported Direct failure, the next commit types only new committed growth (failed slice is not retried)
- [ ] Auto-submit, trailing space, and copy-full-transcript commands occur only on stop leftover, not on mid-session deltas
- [ ] Start with Live Insertion requested but model not streaming yields no live type commands
- [ ] Tests use glossary terms (Dictation Session, Live Insertion, Committed Delta, Tentative Transcript, Cancel)

## Blocked by

None (can start immediately)
