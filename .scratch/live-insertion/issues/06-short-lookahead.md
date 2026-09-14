## Parent

#1 Live Insertion: type Committed Transcript during Plain Dictation

## What to build

When Live Insertion is on and the loaded model streams, Handy starts the stream with a short Lookahead so Committed Transcript (and therefore Direct typing) happens sooner. Cache-aware Nemotron uses ~80 ms Lookahead (right-context 1), not 0 ms. Chunked Unified EN uses the smallest on-menu chunk and right, keeping a large left if the menu allows. When Live Insertion is off, streaming stays accuracy-first. If the stream extension is rejected, start with the model's default StreamOptions and still Live-Insert committed text. There is no extra latency setting. Non-streaming models keep the existing Batch Insertion fallback.

## Acceptance criteria

- [ ] Live Insertion on + Nemotron Streaming: stream begins with ~80 ms Lookahead (right-context 1)
- [ ] Live Insertion on + Parakeet Unified EN: stream begins with smallest on-menu chunk and right; left stays large if the menu allows
- [ ] Live Insertion off: streaming uses the model's accuracy-first default Lookahead
- [ ] Rejected or unaccepted stream extension: fall back to default StreamOptions; Live Insertion still types Committed Deltas
- [ ] No new user-facing latency/accuracy control
- [ ] Non-streaming models: Live Insertion still hidden / Batch Insertion fallback as today
- [ ] Tentative Transcript stays overlay-only; each Committed Delta is still typed immediately

## Blocked by

None (can start immediately)
