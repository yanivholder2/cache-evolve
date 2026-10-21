# Cross-Cycle Insights (through cycle 0007)

## Recurring wins

- **Admission filtering is the main source of hit-rate gains.** S3-FIFO beat LRU, CLOCK, and LIRS in cycle 1. Every winner since then has been a refinement of S3-FIFO's admission window.
- **Making the ghost-into-`main` path stricter paid off steadily.** Ghost size went from near capacity to a half, a quarter, and then an eighth (cycles 3–5). Each step helped, by less each time.
- **`small` and the ghost act as one admission window.** Once ghost-only tuning stopped helping, the gains came from moving resident probation slots out of `small` into ghost entries while keeping `small + ghost` roughly constant (cycles 6–7). Ghost entries cost no resident slots, so the freed slots go to `main`.
- **Exact LRU ordering is not needed for hit rate.** CLOCK-style coarse recency matched LRU (cycle 1), and CLOCK serves as `main`'s retention mechanism in the S3-FIFO line.
- **Changes that only touch representation leave hit rate alone.** v003's hasher and narrow links (cycle 1) and the fingerprint ring (cycle 5) left hit rate exactly unchanged. Both are safe to stack onto any policy. Whether they save memory or time is unmeasured.

## Recurring failures

- **Loosening admission costs hit rate.** Longer ghost windows, ARC-style adaptive `small` sizing, and ghost-hit fast paths that skip proof of reuse all regressed (cycle 2).
- **Making keys prove reuse twice backfires once the window is short.** Two-hit promotion bars and routing ghost hits back through `small` regressed or stayed flat (cycles 3–4). Admission can be too strict as well as too loose.
- **Gates on a single admission path leak.** A gate on only the ghost path, or only `small`, lets keys reach `main` through the other path (cycles 3–4). Gates have to cover every route into `main` and compare against the real eviction victim.
- **Frequency-sketch admission (count-min, TinyLFU) gains only marginally.** Fixing the leak and the victim target did not change that (cycles 4–5). The short-ghost S3-FIFO filter already rejects most of what the sketch would.
- **Retention tuning in `main` is close to exhausted.** Raising `MAX_FREQ` gave a small gain alone (cycle 6) and did not stack with the split change (cycle 7). Starting credits on one admission path only hurt (cycle 7), because ghost evidence is weaker than a hit while resident in `small`.
- **Halving the ghost alone stopped helping once it was smaller than `small`** (cycle 6).
- **Tuning LIRS did not close the gap to S3-FIFO** (cycles 2–3).
- **Process failure: no cycle has measured `metadata-size` or `per-op-time`.** Every memory or speed claim is unverified. Results at a second cache size and on a second trace were asked for repeatedly and never delivered.

## Open questions

- Do the admission-window gains hold at a second cache size or on another trace, or do they just fit workload-a's reuse distances? This should be checked before trusting further marginal gains.
- How small can `small` go before keys arriving in `main` with too little proven reuse outweigh the extra slots `main` gains? What split of the combined window is best?
- Credits: would carrying credits over from `small` into `main`, or giving the same starting credit on both admission paths, make a higher `MAX_FREQ` useful?
- Costs: do the fingerprint ring (one shared hash, no key clones) and the hasher plus narrow-index graft actually cut `metadata-size` and `per-op-time`? Can the harness report those dimensions at all?
- Are there hit-rate levers outside admission filtering and retention credits that have not been tried yet?

## Per-policy notes

- **S3-FIFO (main line):** current best has `small` at about a fortieth of capacity, a larger ghost so `small + ghost` matches the earlier window, `MAX_FREQ = 7`, and a CLOCK-managed `main`. The sequence-number ghost fix is correct but has no measurable effect. The fingerprint ring has been validated on an ancestor but not yet ported to the current line.
- **CLOCK:** matches LRU's hit rate with a cheaper hit path. Tail-latency risk on `put` during the victim sweep is unmeasured. It is no longer pursued as a standalone policy.
- **LIRS:** beats LRU but trails S3-FIFO, and has a heavier node and a linear bound-walk. A larger HIR fraction hurt. It is parked until costs can be measured and the bound-walk is made O(1).
- **ARC-style adaptivity:** drifting away from the fixed probation split cost hit rate. Dropped.
- **TinyLFU / count-min:** gains only marginally on top of S3-FIFO, and its per-access cost is unmeasured. Only worth revisiting if it turns out to be nearly free.
- **Strict LRU (baseline):** useful as a host for representation-only controls, and is beaten on hit rate by every admission-filtering variant.
