# Live Insertion uses a short Lookahead

When Live Insertion is on, Handy asks the loaded streaming model for a low-latency Lookahead so Committed Transcript appears sooner. When Live Insertion is off, streaming (overlay preview, Batch Insertion) keeps the model's accuracy-first default. Tentative Transcript still stays overlay-only; a shorter Lookahead only changes when text becomes committed, not what is typed. Non-streaming models keep Batch Insertion; Live Insertion does not apply.

For cache-aware Nemotron, that Lookahead is ~80 ms (right-context 1), not 0 ms. For chunked Unified EN, pick the smallest on-menu chunk and right; keep a large left if the menu allows. There is no extra latency setting: Live Insertion on implies short Lookahead and Silence Feeding. If the stream extension is rejected, start with the model's default StreamOptions and still Live-Insert committed text.
