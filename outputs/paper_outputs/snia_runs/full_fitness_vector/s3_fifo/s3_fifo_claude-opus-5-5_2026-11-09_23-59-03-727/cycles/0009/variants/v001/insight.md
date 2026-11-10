# Variant v001 Insight

## Helped

- hit-rate: The size-gated 1/5 small share gives more room to main at 16384. That lifted hm_0 at 16384 as the hypothesis predicted. web_0 at 16384 also rose slightly, which goes against the rationale's fear (from cycle 5's global 10% split) that shrinking small would cost web_0. At 1/5, small still holds one-hit-wonders long enough that web_0's short-reuse keys reach main. The summary went up slightly, and because of `SMALL_CACHE_THRESHOLD`, both 131072 cells are exactly the same as the parent's.

## Hurt

- per-op-time: `get` p99 and `put` p50/p99 each rose by about 1 ns. The only code change is a branch on `capacity` inside `try_new`, which runs once at construction and never on the hot path. A slightly larger main and ghost bound at 16384 does not change the per-operation work either. So this is almost certainly run-to-run noise and not a cost of the change.

(metadata-size did not move: `Entry<u64,u64>` and `S3Fifo<u64,u64>` have the same layout, because the split only changes the queue capacities chosen at runtime.)

## Takeaway

Gating the split on capacity worked: it isolated the change to the small-cache regime and left the saturated 131072 cells unchanged by construction. At 16384, a mild move from 1/4 to 1/5 helped both traces. Cycle 5's 10% split had hurt web_0, so the small-queue share seems to have a sweet spot near 20% on this parent, not a monotonic "smaller is better" trend. The gains are small, though, so the small/main split looks nearly exhausted as a hit-rate lever in this policy family. The rationale's "converged" condition (web_0 losing more than hm_0 gains) was not triggered, but there is little headroom left.
