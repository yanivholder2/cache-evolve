# Cross-Cycle Insights (through cycle 0007)

## Recurring wins

- **Proven items must not land in the victim slot.** This placement rule has won three times at three different moves: stash demotion to MRU (c1), stash→target promotion to `Back` (c3), and target→probing promotion to `Back` (c5–c6). Each time it paid off most where slots are contested.
- **Reuse history kept outside the item beats in-item counters.** The FIFO ghost ring of stash-evicted keys (c5) was the first lever to move `web_0` 128K. It decays by overwrite and holds evidence across evictions, which fixes both counter failure modes. A factor-2 ring with consume-on-put is settled (c6–c7).
- **A two-hit stash exit gate with a strict room bypass.** The gate helps at small sizes. Bypassing it only when the target has real spare room recovered `web_0` 128K without the 16K cascade cost (c3–c4).
- **Moves that use only free slots and cascade nothing are safe.** The rescue of stash-overflow victims into free target or probing slots (c7) regressed nowhere. It only pays off at the large size.
- **Levers on separate tiers or separate traces stack.** Examples: ghost plus probing placement on `hm_0` (c6). The lineage advances by combining one-lever wins that don't overlap.

## Recurring failures

- **Easier routes back into a crowded target.** This has lost many times: the one-hit return for demotees (c3), the last-free-slot bypass (c3), and ghost credit on demotion (c6). An item that has lost its slot should not outrank current residents. In tight caches a "free" slot sets off a cascade.
- **Counter-based signals without a lifecycle.** Counters reset on moves stay near zero (c2). Counters that never decay become permanent pins (c4). Frequency comparisons and counter-based stash victim selection both collapsed or lost hit-rate in every cell.
- **Retuning the target→probing trigger.** Making it slower (c4) and faster (c5) both lost. The current trigger looks optimal, because every extra crossing adds an eviction cascade.
- **Inserting fresh puts at the stash LRU end.** This amounts to evicting them on the next collision and starves new keys (c2).
- **Fixed ghost-factor tuning.** A longer ring gives stale credit (c6). A shorter ring helps `web_0` but drops the `hm_0` 128K tail (c7).
- **Foreign-resident retry changes** had no effect (c7).
- **Cost never measured.** Per-op-time and metadata-size have gone unreported for seven cycles. Every cost claim is unverified, including the ghost's metadata and the lookup reorder (c1).

## Open questions

- **`web_0` 16K** has been flat for many cycles. It is ruled out as a problem of ghost reach, free slots, or placement, and looks capacity-saturated. Would choosing a different victim in full normal buckets move it? Options: recency vs. frequency, or ghost-informed choice.
- **`web_0` 128K giveback** since probing placement was added (c6): is it ghost churn caused by probing-induced evictions?
- **Adaptive ghost reach:** could a per-trace or occupancy-conditioned reach keep `web_0`'s short-ring gain and `hm_0` 128K's long tail?
- **Rescue and bypass interaction:** does the rescue push targets below the bypass threshold, silently turning bypass promotions into gated ones?
- **Lookup reorder (c1 v003):** it leaves hit-rate unchanged. Does it lower latency? That needs per-op-time measurement.

## Per-policy notes

- **Stash (bucket-local LRU):** plain LRU eviction is confirmed (c4). Demote to MRU, gate exit at two hits, and back it with the ghost ring. Levers here act on a tiny population, so they interact rather than add.
- **Normal / target buckets:** promotees land at `Back`. The bypass needs real spare room, and with 2-slot buckets that effectively means an empty target. The rescue uses free slots only. Victim choice in full buckets is the untested lever.
- **Probing tier:** `Back` placement pays off on `hm_0`, whose hot set reaches probing. Leave the trigger alone. Pushing foreign residents out costs a little on `web_0` 128K.
- **Ghost ring:** FIFO, factor 2, consume-on-put. Most of its value comes from returns within one stash-capacity. `hm_0` 128K also uses the 1–2× tail.
- **Trace and size regularities:** `hm_0` rewards protecting residents and longer memory. `web_0` rewards letting items in quickly and shorter memory. Small sizes punish anything that cascades. Large sizes reward using free capacity.
