## Parent

#1 Live Insertion: type Committed Transcript during Plain Dictation

## What to build

After leftover Direct typing on a normal stop: apply trailing space only on that last insertion if the setting is on, auto-submit once if on, and copy the full model transcript to the clipboard once if Copy to Clipboard is on. A failed Direct increment is dropped, an error is shown, and the Dictation Session continues; later deltas still type. One history row at a normal end is the full model transcript even if some typing failed. Cancel still writes no history.

## Acceptance criteria

- [ ] Trailing space is not inserted between mid-session Committed Deltas; it applies only to leftover on stop when the setting is on
- [ ] Auto-submit fires once after leftover on stop, never mid-session
- [ ] Copy to Clipboard copies the full model transcript once at stop, not per delta
- [ ] Direct failure of a delta: increment dropped, user-visible error, session continues, later growth still typed
- [ ] Normal stop writes one history row with the full model transcript; Direct success does not gate the row (keep today's WAV-saved gating if it already exists)
- [ ] Cancel writes no history row

## Blocked by

- #4 Wire Live Insertion into Plain Dictation
