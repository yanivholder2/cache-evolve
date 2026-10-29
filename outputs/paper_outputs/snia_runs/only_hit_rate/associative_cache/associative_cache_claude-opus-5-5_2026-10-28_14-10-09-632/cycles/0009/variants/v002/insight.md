# Variant v002 Insight

## Helped

- hit-rate (web_0 @ 16384): The cell went up a little. The window in 128-slot buckets grew from 4 to 16 probation insertions, so more of web_0's burst re-references now land inside it, get no counter bump, and reach the FIFO head at counter 0 instead of being promoted. But the cell only ties the best small-size web_0 from cycle 0008. It does not beat it, so by the hypothesis's own refutation clause the claim is not confirmed. Bursts are a minor limit on small-bucket web_0, not the main one.

## Hurt

- hit-rate (hm_0 @ 16384): The cell fell slightly. It stayed above the refutation floor, but the drop shows that a 16-insertion window in a 64-slot ring starts to catch some of hm_0's real short-gap reuse. That reuse now gets no bump and is evicted from probation. In v003 the 4-slot window did not reach it.

Not moved: both 131072 cells are unchanged, as expected. `crp_window_len(256)` is still `256 / 16 = 16`, so the 1024-slot buckets behave exactly as in v003.

## Takeaway

An absolute 16-slot window does not carry over from 1024-slot to 128-slot buckets the way v003's insight predicted. In a 64-slot probation ring, 16 slots is a quarter of the ring rather than a sixteenth. That much width filters a small gain's worth of web_0 bursts and costs about the same in hm_0 real reuse, so the overall summary barely changes. The burst filter is close to the limit of what it can do at small sizes. Further web_0 gains at 16384 will likely need a different lever than the window width, such as probation depth or the promotion rule.
