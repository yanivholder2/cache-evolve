# Variant v003 Insight

## Helped

(No dimension improved.)

## Hurt

- per-op-time: The tail-latency goal failed. Get and put p99 both got much worse instead of better, so the hypothesis's confirmation criterion is not met. But the get and put **medians** also went up by about the same factor. The swap only touches the full-target stash-promotion branch. That branch is a slow path and is never on put's path, so it cannot by itself slow down the median of every operation. A uniform slowdown like this points to one of two causes: something outside the guarded branch (a shared helper the swap reuses, or extra work added to the common lookup/insert path), or a change in the benchmark environment. It does not point to the swap's own cost.

## Takeaway

Replacing evict-and-cascade with a direct target↔stash swap left hit-rate exactly the same in every cell, at both sizes and on both traces. The hypothesis's refutation case (large-cell hit-rate falls) did not happen. So the old neighbour/probing retry was not keeping useful residents cached on these traces, and the swap is safe for hit-rate. It is just as consistent with the guarded full-target branch hardly ever firing, so the claim that it "fires much more often" than the probing arm is still unconfirmed. Because the latency regression is uniform and even hits put, its cause is unknown, and the variant does not show that the swap is slower. Before rejecting the idea, check that the change is limited to the guarded branch and re-run timing against the parent in the same session. Note: this was written from the hypothesis and both fitness vectors. The overlay source files could not be listed in this session, so the code-level reasoning assumes the hypothesis was implemented as written.
