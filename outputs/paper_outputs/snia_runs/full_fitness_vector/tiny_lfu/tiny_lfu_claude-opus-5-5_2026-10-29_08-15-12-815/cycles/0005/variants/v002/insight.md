# Variant v002 Insight

## Helped

- hit-rate: Almost all of the gain came from hm_0 at 16384. There, `try_new` now gives the window 819 entries, up from 163. A newly arrived key gets about five times as many accesses to build up sketch counts before it has to win the `cand_freq > victim_freq` contest. That matters on hm_0's recency-heavy short bursts. web_0 at 16384 barely moved, so the extra window slots did little for that trace's access pattern. At 131072 both traces are unchanged, as designed. `pct1` (1310) beats the capped `pct5` (820), so the window, the main region and protected_cap all match the parent exactly. Conservative update's large-cache gain is fully kept. Still, the summary landed just under the hypothesis's 0.56 target. The gain from the window floor added to conservative update, but it was smaller than the hypothesis expected.

## Hurt

- per-op-time: Every latency number (get and put, p50 and p99) roughly doubled, and the tails grew even more. No code change explains this. The diff only touches `try_new`, which runs once at construction. It changes `window_cap`, a field read only on the fill path (`lists[WINDOW].len > window_cap`). `get`, `put`, `evict_one`, `on_hit` and `increment_hashed` are the same as the parent's. A uniform slowdown on byte-identical hot code, with tails hit harder than medians, points to a change in the benchmark host (noisy neighbor, frequency scaling), not to this variant. This dimension should be re-measured before it counts against v002.

## Takeaway

A size-aware window floor is a working way to apply the 5% window only where it helps. The `max(pct1, min(pct5, 820))` form leaves large caches bit-for-bit identical to the parent, so the hit-rate effects add up cleanly with no cost at 131072. But at 16384 the benefit depends on the trace: it showed up on hm_0 and barely on web_0. Combining the two cycle-0004 levers gives a real but small gain, not a jump. Since the hot path is unchanged, this variant's latency regression is a measurement artifact. The next comparison of per-op-time across variants needs a re-run on a stable host.
