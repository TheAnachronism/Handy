# Live Insertion uses Direct typing

Batch Insertion honors the user's Paste Method (clipboard chords, Direct, or None). Live Insertion always injects via Direct keystroke typing, including leftover text on session stop and even when Paste Method is None, because clipboard paste is built for one blob and is a poor fit for many committed increments. Auto-submit still runs once at session end; `append_trailing_space` applies only to that last insertion. If Copy to Clipboard is on, the full model transcript is copied once at stop, not per delta.
