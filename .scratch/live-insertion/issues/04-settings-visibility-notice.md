## Parent

#1 Live Insertion: type Committed Transcript during Plain Dictation

## What to build

The Live Insertion control lives in General settings near overlay / paste. It is shown only when the currently selected model reports streaming support. The stored flag stays on if the user switches to a non-streaming model (control hidden). Starting Plain Dictation with the flag on and a non-streaming model uses Batch Insertion and shows a one-shot notice. Setting copy explains that fallback.

## Acceptance criteria

- [ ] Control is visible only when the current model advertises streaming; hidden otherwise
- [ ] Switching to a non-streaming model does not clear the stored flag
- [ ] Switching back to a streaming model shows the control still on without re-toggling
- [ ] Plain Dictation start with flag on + non-streaming model uses Batch Insertion and shows a one-shot notice (not every subsequent session spam)
- [ ] Control sits with overlay / paste settings in General
- [ ] Overlay style remains independent of Live Insertion

## Blocked by

- #4 Wire Live Insertion into Plain Dictation
