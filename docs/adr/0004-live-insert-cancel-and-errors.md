# Cancel leaves Live Insertion in place

Cancel stops capture and does not finalize leftover Tentative Transcript. Text already typed stays in the Target Application; Handy does not backspace it. A failed Direct increment is dropped (not retried); the session continues and an error is shown. Transcription history is the model transcript at a normal session end, one row, even if some increments failed to type. The Live Insertion flag stays on if the user switches to a non-streaming model; the control is hidden and Plain Dictation falls back to Batch Insertion with a one-shot notice.
