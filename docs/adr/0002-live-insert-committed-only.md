# Live Insertion writes only Committed Transcript

Streaming models revise Tentative Transcript. Target applications cannot reliably rewrite already-inserted text. Live Insertion therefore types only Committed Transcript into the Target Application; Tentative Transcript stays on the overlay. A normal stop inserts only leftover text that was not already typed (finalized tentative), never the whole session again. Overlay visibility stays independent of Live Insertion. Post-Process Dictation stays Batch Insertion only.
