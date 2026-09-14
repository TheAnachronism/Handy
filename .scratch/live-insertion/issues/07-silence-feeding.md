## Parent

#1 Live Insertion: type Committed Transcript during Plain Dictation

## What to build

While Live Insertion is active, every microphone frame — speech and silence — is given to the streaming model for the whole Dictation Session so Lookahead can finish and the tail can become Committed Transcript without the user stopping. A pause still does not auto-stop the session and does not auto-finalize Tentative Transcript. When Live Insertion is off, Batch Insertion may still drop silence after VAD hangover.

## Acceptance criteria

- [ ] Live Insertion session: every mic frame (including silence) is fed to the streaming model until stop or Cancel
- [ ] A pause does not end the Dictation Session
- [ ] A pause does not auto-finalize Tentative Transcript
- [ ] After a pause, remaining Tentative can become Committed and be Direct-typed without the user stopping
- [ ] Live Insertion off: existing VAD hangover / drop-silence behaviour for Batch Insertion is unchanged
- [ ] Cancel still discards leftover Tentative and does not type it

## Blocked by

None (can start immediately)
