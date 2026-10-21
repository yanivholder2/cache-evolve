# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:46:47+00:00

## timeline

### 09:47:06 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0007)\n\n## Recurring wins\n\n- **Admission filtering is the main source of hit-rate gains.** S3-FIFO beat LRU, CLOCK, and LIRS in cycle 1. Every winner since then has been a refinement of S3-FIFO's admission window.\n- **Making the ghost-into-`main` path stricter paid off steadily.** Ghost size went from near capacity to a half, a quarter, and then an eighth (cycles 3–5). Each step helped, by less each time.\n- **`small` and the ghost act as one admission window.** Once ghost-only tuning stopped helping, the gains came from moving resident probation slots out of `small` into ghost entries while keeping `small + ghost` roughly constant (cycles 6–7). Ghost entries cost no resident slots, so the freed slots go to `main`.\n- **Exact LRU ordering is not needed for hit rate.** CLOCK-style coarse recency matched LRU (cycle 1), and CLOCK serves as `main`'s retention mechanism in the S3-FIFO line.\n- **Changes that only touch representation leave hit rate alone.** v003's hasher and narrow links (cycle 1) and the fingerprint ring (cycle 5) left hit rate exactly unchanged. Both are safe to stack onto any policy. Whether they save memory or time is unmeasured.\n\n## Recurring failures\n\n- **Loosening admission costs hit rate.** Longer ghost windows, ARC-style adaptive `small` sizing, and ghost-hit fast paths that skip proof of reuse all regressed (cycle 2).\n- **Making keys prove reuse twice backfires once the window is short.** Two-hit promotion bars and routing ghost hits back through `small` regressed or stayed flat (cycles 3–4). Admission can be too strict as well as too loose.\n- **Gates on a single admission path leak.** A gate on only the ghost path, or only `small`, lets keys reach `main` through the other path (cycles 3–4). Gates have to cover every route into `main` and compare against the real eviction victim.\n- **Frequency-sketch admission (count-min, TinyLFU) gains only marginally.** Fixing the leak and the victim target did not change that (cycles 4–5). The short-ghost S3-FIFO filter already rejects most of what the sketch would.\n- **Retention tuning in `main` is close to exhausted.** Raising `MAX_FREQ` gave a small gain alone (cycle 6) and did not stack with the split change (cycle 7). Starting credits on one admission path only hurt (cycle 7), because ghost evidence is weaker than a hit while resident in `small`.\n- **Halving the ghost alone stopped helping once it was smaller than `small`** (cycle 6).\n- **Tuning LIRS did not close the gap to S3-FIFO** (cycles 2–3).\n- **Process failure: no cycle has measured `metadata-size` or `per-op-time`.** Every memory or speed claim is unverified. Results at a second cache size and on a second trace were asked for repeatedly and never delivered.\n\n## Open questions\n\n- Do the admission-window gains hold at a second cache size or on another trace, or do they just fit workload-a's reuse distances? This should be checked before trusting further marginal gains.\n- How small can `small` go before keys arriving in `main` with too little proven reuse outweigh the extra slots `main` gains? What split of the combined window is best?\n- Credits: would carrying credits over from `small` into `main`, or giving the same starting credit on both admission paths, make a higher `MAX_FREQ` useful?\n- Costs: do the fingerprint ring (one shared hash, no key clones) and the hasher plus narrow-index graft actually cut `metadata-size` and `per-op-time`? Can the harness report those dimensions at all?\n- Are there hit-rate levers outside admission filtering and retention credits that have not been tried yet?\n\n## Per-policy notes\n\n- **S3-FIFO (main line):** current best has `small` at about a fortieth of capacity, a larger ghost so `small + ghost` matches the earlier window, `MAX_FREQ = 7`, and a CLOCK-managed `main`. The sequence-number ghost fix is correct but has no measurable effect. The fingerprint ring has been validated on an ancestor but not yet ported to the current line.\n- **CLOCK:** matches LRU's hit rate with a cheaper hit path. Tail-latency risk on `put` during the victim sweep is unmeasured. It is no longer pursued as a standalone policy.\n- **LIRS:** beats LRU but trails S3-FIFO, and has a heavier node and a linear bound-walk. A larger HIR fraction hurt. It is parked until costs can be measured and the bound-walk is made O(1).\n- **ARC-style adaptivity:** drifting away from the fixed probation split cost hit rate. Dropped.\n- **TinyLFU / count-min:** gains only marginally on top of S3-FIFO, and its per-access cost is unmeasured. Only worth revisiting if it turns out to be nearly free.\n- **Strict LRU (baseline):** useful as a host for representation-only controls, and is beaten on hit rate by every admission-filtering variant.\n"
}
```

### 09:47:13 chat

I wrote `cross-cycle-insights.md` from the cycle 1–7 ledger, with the four sections in the required order.

**The main gap:** no cycle has measured `metadata-size` or `per-op-time`, so every memory or speed claim in the ledger is still unchecked. Every gain also comes from one trace at one cache size, and the recent gains are now small.

**Recurring wins:**
- **S3-FIFO's admission filtering** is where the hit-rate gains come from.
- **Shrinking the ghost** helped three times in a row (half, quarter, eighth of capacity), by less each time.
- **Moving slots from `small` into the ghost** while keeping their combined size fixed helped in cycles 6 and 7. The freed slots go to `main`.
- **Representation-only changes** (the hasher with narrow links, the fingerprint ring) left hit rate exactly unchanged, so they can be stacked onto any policy.

**Recurring failures:**
- Letting keys into `main` more easily lowered hit rate.
- Making keys prove reuse twice backfired once the ghost window was short.
- Gates on only one route into `main` leaked through the other route.
- Frequency-sketch admission (TinyLFU) gained only marginally, even after its flaws were fixed.
- Raising the frequency cap and giving starting credits are nearly exhausted.
- LIRS tuning never closed the gap to S3-FIFO.

**Open questions:**
- Do the gains hold at a second cache size or on another trace?
- How small can `small` get before it hurts?
- Would credits treated the same on both admission paths, or carried over from `small` into `main`, help?
- What do the representation changes actually cost in memory and time?

**Per-policy notes** cover the current S3-FIFO configuration, CLOCK, LIRS (set aside), ARC-style sizing (dropped), TinyLFU, and the LRU baseline.


---

- finished: 2026-10-21T09:47:13+00:00
- duration: 26.26s
- status: ok
- input_tokens: 4
- output_tokens: 2551
- cache_read_tokens: 39992
- cache_write_tokens: 13235
- cost_usd: 0.164914
