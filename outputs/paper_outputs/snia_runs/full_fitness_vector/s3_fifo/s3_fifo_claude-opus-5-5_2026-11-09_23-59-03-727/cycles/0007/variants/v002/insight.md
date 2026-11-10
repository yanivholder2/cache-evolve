# Variant v002 Insight

## Helped

(No dimension improved meaningfully. web_0 at 131072 was the only hit-rate cell that did not fall, and its uptick is too small to tell apart from no change. So it only weakly supports the secondary idea that faster aging would undo the parent's web_0 overstay.)

## Hurt

- hit-rate: The targeted cell, hm_0 at 16384, fell instead of rising. web_0 at 16384 and hm_0 at 131072 also fell, so the summary dropped. Halving (`freq >>= 1`) changes behavior only for keys at freq ≥ 3, the keys with the strongest in-main reuse. For example, a key at 7 now drops to 3 in one sweep. `get` adds only +1 per hit, so the key would need about four hits between consecutive sweeps to stay at the cap. The keys being evicted earlier were therefore not stale. They were frequently reused keys whose inter-hit gap is longer than a sweep, and they leave main before their next burst.
- per-op-time: `get` p99 and `put` p99 rose slightly, while both p50s held. `get` was not touched, and halving should mean fewer recycles per `evict_main` call, not more. So this is most likely run-to-run noise. Either way, the hypothesized lower worst-case eviction work did not show up as a latency gain.

(metadata-size did not move: the change is a single operator in `evict_main`, and `Entry<u64,u64>` stays at 24 bytes.)

## Takeaway

Faster aging of main-queue counts hurts on these MSR traces, including small-cache hm_0. Keys at freq 3–7 in main are worth the long residency that linear decrement gives them, and the "dead residency" the hypothesis blamed did not exist in a form halving could reclaim. One correction to the hypothesis: under halving, freq 2 still survives two sweeps (2→1→0), not one. So keys at freq ≤ 2 behave exactly as in the parent, and every loss comes from the high-frequency tail. A lever for hm_0 at 16384 should give strongly reused keys more retention, or change the small/main split, not shorten the tenure of main residents.
