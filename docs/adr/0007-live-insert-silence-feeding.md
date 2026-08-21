# Live Insertion feeds every microphone frame

ADR 0003 already keeps the Dictation Session open through silence and does not auto-finalize Tentative Transcript. For Live Insertion, every microphone frame — including silence — is sent to the streaming model so Lookahead can complete and the tail can become Committed Transcript without the user stopping. Batch Insertion may still drop silence after VAD hangover. A pause is not an utterance end. Direct typing of each Committed Delta stays immediate (ADR 0005).
