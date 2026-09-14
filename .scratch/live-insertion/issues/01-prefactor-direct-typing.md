## Parent

#1 Live Insertion: type Committed Transcript during Plain Dictation

## What to build

Batch Insertion still pastes exactly as today, but Direct typing is callable as a plain "type this string" action: no trailing space, no auto-submit, no clipboard save/restore, no paste delays. Live Insertion will use that action for Committed Deltas and leftover; this ticket does not turn Live Insertion on.

## Acceptance criteria

- [ ] Direct typing can be invoked with an arbitrary string without applying append_trailing_space, auto-submit, Copy to Clipboard, or clipboard Paste Method behaviour
- [ ] Existing Batch Insertion via Paste Method Direct still applies trailing space, auto-submit, and clipboard handling the same as before
- [ ] Clipboard Paste Methods (Ctrl+V and siblings) and Paste Method None are unchanged
- [ ] Linux preferred typing tool still applies to this Direct path, same as Batch Insertion Direct

## Blocked by

None (can start immediately)
