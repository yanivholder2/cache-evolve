## Recurring wins

- Efficiency changes that add no hot-path work keep paying off. Examples are cheaper hashing (c1), compact `u32` links (c2) and a fused single-pass conservative-update increment held in registers (c4). Each one lowered per-op cost or memory without changing behaviour.
- Better estimate accuracy is the only lever that has raised hit rate so far. Conservative update (c3) and wider `u16` counters at the same byte size (c4) each gave a small gain. The gains are shrinking, so this lever is now close to used up.
- Spending sketch bytes on width rather than counter depth works. Saturating `u16` counters lost nothing, and the bytes they freed paid for less collision noise.

## Recurring failures

- Sketch aging/halving has failed twice: with 4-bit counters (c2) and with full-width counters plus conservative update (c4). Workload-a rewards stable long-term frequency. In a strict duel with no window, decay removes the margin that keeps scan traffic out.
- Changes that add correlated collisions or blur estimates hurt admission. Examples are the blocked/cache-line sketch layout (c3) and coarse aged counters that create more ties (c2).
- Changing the tie rule didn't help. Letting newcomers win ties (`>=`, c3) lowered hit rate, so the strict `>` rule is not over-rejecting.
- Heavier resident-list structure or extra per-op work cost latency and memory with no hit-rate payoff. Examples are W-TinyLFU window + SLRU with per-hit relinking (c1), LRU-end victim sampling (c2) and extra per-miss hashing (c2).
- Layout changes aimed at the sketch's cache behaviour have nothing left to win, because the sketch already fits in cache.
- Per-op latency readings are unreliable. Variants with no code-level cause have repeatedly come out "faster on every op" (c2–c4). Trust a latency claim only if it has a mechanism in the code and has been benchmarked side by side with its parent.

## Open questions

- Do the fused increment and the `u16`/2× width sketch combine cleanly, keeping both the latency gain and the hit-rate gain?
- Now that sketch accuracy has stopped giving gains, does the remaining hit-rate gap lie in policy structure: a small admission window or doorkeeper ahead of the duel, or a smarter victim than the plain LRU head? Any such change must stay cheap per hit.
- Would a very mild form of aging help, such as a much longer period or decay driven by misses only? Or should aging be dropped for workload-a for good?
- Is workload-a close to its hit-rate ceiling for TinyLFU-family designs, leaving efficiency as the only axis left to improve?
- Is the unkeyed fast hasher an acceptable risk on adversarial or collision-prone key traces? This hasn't been tested.

## Per-policy notes

- TinyLFU admission (CountMin duel against the LRU victim). Hit rate depends on how far apart cold newcomers and cold victims sit in their estimates. Conservative update plus wider `u16` counters is the current best sketch. Keep the strict `>` rule, and keep rows independent.
- Plain LRU resident list with compact `u32` links. This is the cheap base that every later cycle has built on. Victim sampling near the LRU end adds nothing, because the estimates there are nearly equal.
- W-TinyLFU (window + SLRU). Tried once (c1) with about a 1% window and a sketch that never aged. It was a net loss on cost with almost no hit-rate gain. It hasn't been retried since aging failed. If it is revisited, it needs low relink cost and possibly a different window size.
