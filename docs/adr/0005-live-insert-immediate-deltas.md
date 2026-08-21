# Live Insertion types each committed delta immediately

Live Insertion sends each new Committed Delta to Direct typing as soon as the model emits it, rather than coalescing bursts or waiting for phrase endpoints. Latency beats fewer injection calls; the user may interleave their own typing at the caret between deltas.
