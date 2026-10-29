# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-28T19:19:46+00:00

## timeline

### 19:20:05 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0007)\n\n## Recurring wins\n\n- **Proven items must not land in the victim slot.** This placement rule has won three times at three different moves: stash demotion to MRU (c1), stash→target promotion to `Back` (c3), and target→probing promotion to `Back` (c5–c6). Each time it paid off most where slots are contested.\n- **Reuse history kept outside the item beats in-item counters.** The FIFO ghost ring of stash-evicted keys (c5) was the first lever to move `web_0` 128K. It decays by overwrite and holds evidence across evictions, which fixes both counter failure modes. A factor-2 ring with consume-on-put is settled (c6–c7).\n- **A two-hit stash exit gate with a strict room bypass.** The gate helps at small sizes. Bypassing it only when the target has real spare room recovered `web_0` 128K without the 16K cascade cost (c3–c4).\n- **Moves that use only free slots and cascade nothing are safe.** The rescue of stash-overflow victims into free target or probing slots (c7) regressed nowhere. It only pays off at the large size.\n- **Levers on separate tiers or separate traces stack.** Examples: ghost plus probing placement on `hm_0` (c6). The lineage advances by combining one-lever wins that don't overlap.\n\n## Recurring failures\n\n- **Easier routes back into a crowded target.** This has lost many times: the one-hit return for demotees (c3), the last-free-slot bypass (c3), and ghost credit on demotion (c6). An item that has lost its slot should not outrank current residents. In tight caches a \"free\" slot sets off a cascade.\n- **Counter-based signals without a lifecycle.** Counters reset on moves stay near zero (c2). Counters that never decay become permanent pins (c4). Frequency comparisons and counter-based stash victim selection both collapsed or lost hit-rate in every cell.\n- **Retuning the target→probing trigger.** Making it slower (c4) and faster (c5) both lost. The current trigger looks optimal, because every extra crossing adds an eviction cascade.\n- **Inserting fresh puts at the stash LRU end.** This amounts to evicting them on the next collision and starves new keys (c2).\n- **Fixed ghost-factor tuning.** A longer ring gives stale credit (c6). A shorter ring helps `web_0` but drops the `hm_0` 128K tail (c7).\n- **Foreign-resident retry changes** had no effect (c7).\n- **Cost never measured.** Per-op-time and metadata-size have gone unreported for seven cycles. Every cost claim is unverified, including the ghost's metadata and the lookup reorder (c1).\n\n## Open questions\n\n- **`web_0` 16K** has been flat for many cycles. It is ruled out as a problem of ghost reach, free slots, or placement, and looks capacity-saturated. Would choosing a different victim in full normal buckets move it? Options: recency vs. frequency, or ghost-informed choice.\n- **`web_0` 128K giveback** since probing placement was added (c6): is it ghost churn caused by probing-induced evictions?\n- **Adaptive ghost reach:** could a per-trace or occupancy-conditioned reach keep `web_0`'s short-ring gain and `hm_0` 128K's long tail?\n- **Rescue and bypass interaction:** does the rescue push targets below the bypass threshold, silently turning bypass promotions into gated ones?\n- **Lookup reorder (c1 v003):** it leaves hit-rate unchanged. Does it lower latency? That needs per-op-time measurement.\n\n## Per-policy notes\n\n- **Stash (bucket-local LRU):** plain LRU eviction is confirmed (c4). Demote to MRU, gate exit at two hits, and back it with the ghost ring. Levers here act on a tiny population, so they interact rather than add.\n- **Normal / target buckets:** promotees land at `Back`. The bypass needs real spare room, and with 2-slot buckets that effectively means an empty target. The rescue uses free slots only. Victim choice in full buckets is the untested lever.\n- **Probing tier:** `Back` placement pays off on `hm_0`, whose hot set reaches probing. Leave the trigger alone. Pushing foreign residents out costs a little on `web_0` 128K.\n- **Ghost ring:** FIFO, factor 2, consume-on-put. Most of its value comes from returns within one stash-capacity. `hm_0` 128K also uses the 1–2× tail.\n- **Trace and size regularities:** `hm_0` rewards protecting residents and longer memory. `web_0` rewards letting items in quickly and shorter memory. Small sizes punish anything that cascades. Large sizes reward using free capacity.\n"
}
```

### 19:20:12 chat

I wrote `cross-cycle-insights.md` in the cycle 0007 output dir. I built it fresh from the ledger digest for cycles 0001–0007, with the four required sections in order. It only describes results qualitatively and makes up no numbers.

- **Recurring wins:**
  - Never put a proven item in the eviction slot; this held at three different moves.
  - A memory of keys recently evicted from the stash (the ghost ring) beat counters stored on the items. Its current design looks settled.
  - The two-hit gate works, with a bypass only when the target has real spare room.
  - Moves that only fill free slots, without pushing anything out, never caused a regression.
  - Levers acting on separate tiers or traces add up.
- **Recurring failures:**
  - Giving evicted items an easier way back into a full target lost every time it was tried.
  - Counters that never decay, or that reset whenever an item moves, didn't work.
  - Changing when items move from target to probing lost in both directions.
  - Fixed changes to the ghost ring's length didn't help either way.
  - Per-op-time and metadata-size have never been measured in seven cycles, so every claim about cost is still unchecked.
- **Open questions:**
  - Would changing which resident gets evicted from a full bucket finally move `web_0` 16K?
  - Is the small `web_0` 128K loss since cycle 6 caused by churn in the ghost ring?
  - Could a ghost ring length that adapts to the trace or to occupancy beat the fixed setting?
  - Do the rescue (cycle 7) and the gate bypass interfere with each other?
  - Does the lookup reorder from cycle 1 actually cut latency?
- **Per-policy notes:** grouped by stash, target buckets, probing tier and ghost ring, plus how the two traces and two cache sizes behave differently.


---

- finished: 2026-10-28T19:20:13+00:00
- duration: 26.20s
- status: ok
- input_tokens: 4
- output_tokens: 2381
- cache_read_tokens: 40549
- cache_write_tokens: 12378
- cost_usd: 0.15477
