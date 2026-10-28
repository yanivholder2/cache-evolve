# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-28T19:40:23+00:00

## timeline

### 19:40:26 thinking



### 19:40:43 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Placement held up three times (c1 stash demotion to MRU, c3 promotion to target `Back`, c5/c6 target→probing `Back`): an item that just proved itself should not land in the victim slot. The gains were largest where slots are contested.\n- History that outlives eviction worked where in-item counters failed. The global FIFO ghost ring (factor 2, consumed on put) was the first lever to move `web_0` 128K. Tests of longer, shorter, double-spend and partitioned versions in c6, c7 and c9 all confirmed the current design.\n- Levers that act on separate tiers or traces mostly added up when combined (gate + MRU demotion, ghost + probing placement). Levers that act on the same small stash population interfered with each other instead.\n- Moves that only use free slots and displace no one never regressed (c4 strict bypass threshold, c7 rescue). Their value is limited to the large size, though, and this direction is now used up.\n- The lineage moves forward mostly by avoiding regressions. Since about c7, winners have gained marginally, often at noise level.\n\n## Recurring failures\n\n- Easier routes into a crowded target lost four times (c3 v001, c3 v002, c6 v003, partly c8 v002). In tight caches, a \"free\" slot or an extra credit triggers an eviction cascade onto later promotions.\n- Changes to the target→probing trigger lost in both directions (c4 delayed it, c5 sped it up). The trigger is treated as near-optimal.\n- Counter and bit state lost repeatedly. Counters that reset on moves stayed near zero (c2). Counters that never decay pinned items permanently (c4). A 1-bit CLOCK in 2-slot buckets mostly picks the same victim LRU would, and entry and sweeps clear it before it is read (c8, c9).\n- Stash victim selection that departs from plain LRU lost on every cell (c4). LRU-end insertion of fresh puts starved new keys (c2).\n- `web_0` 16K has been flat for about six cycles. It resisted ghost reach, placement, free-slot routing, CLOCK victim choice, ghost-carried bits and value-aware gating. It looks capacity-saturated, and any lever based on position or a single bit is exhausted there.\n- Per-op-time and metadata-size have not been measured for nine cycles. Every cost claim is unverified, so variants whose only purpose is cost reduction (c1 v003, c9 v003) produced no usable signal.\n\n## Open questions\n\n- Would a multi-bit, per-resident reuse count that persists through entry, sweeps and stash stays give victim choice something LRU order doesn't already encode? This is the main remaining idea for `web_0` 16K.\n- Should admission or credit depend on how valuable the displaced resident is, rather than on whether a slot is free? `hm_0` 128K shows that displacing residents can pay.\n- Would ghost-origin promotees placed at the LRU end with the reference bit set let CLOCK finally diverge from LRU?\n- Could an adaptive ghost reach (per trace or occupancy-conditioned) combine `web_0`'s preference for a short ring with `hm_0` 128K's long tail?\n- What causes the small `web_0` 128K giveback that appeared in c6? Ruled out so far: foreign retries, stash→full-target promotion and fixed ghost reach. Ghost churn from probing evictions has not been tested.\n- Could a per-bucket index over the global ghost FIFO cut lookup cost while keeping its reach? This only matters once per-op-time is measured.\n\n## Per-policy notes\n\n- Stash (bucket-local LRU): plain LRU eviction is confirmed. Use MRU-end demotion and the two-hit exit gate, with the gate bypassed when the target is empty (strict threshold). Placing gated promotees at the target `Back` is essential. The stash-overflow rescue into free slots is settled for the large size.\n- Ghost ring: use one global FIFO at factor 2 that is consumed on re-put. It is the main fix for returning keys (`web_0` 128K). Partitioning, longer reach, double-spending and occupancy-conditioned credit all hurt. `hm_0` 128K depends on its long tail.\n- Normal buckets (LRU / 1-bit CLOCK): CLOCK is effectively the same as LRU in 2-slot buckets, and the c8/c9 winners were the same as their parents within noise. Move-to-MRU on a hit already carries the recency signal, so any new victim signal must carry information beyond position.\n- Probing tier: `Back` placement for target→probing promotees pays off on `hm_0`, whose hot set reaches probing. Leave the trigger timing alone. Changes to how foreign residents retry their target had no effect.\n- Traces: `hm_0` rewards protecting residents and long ghost memory, and its 128K cell is the most sensitive to regressions. `web_0` rewards fast first-hit admission when targets have room and a shorter ghost ring. The two traces often pull timing levers in opposite directions.\n"
}
```

### 19:40:49 chat

I've written `cross-cycle-insights.md` from the cycles 1–9 ledger, using the four required sections in order. Since about cycle 7, winners have been ahead mostly by not regressing, with gains at about noise level.

- **Recurring wins:** Not putting an item that just earned its place into the eviction slot paid off three times: stash demotion, target promotion and probing placement. The global ghost ring (factor 2, consumed on re-put) is now settled. Levers that act on different tiers stack. Moves that only fill free slots never regressed, but that direction is used up.
- **Recurring failures:** Making it easier to get back into a crowded target lost four times. Changing when items move to probing lost whether it was made sooner or later. Counter and 1-bit state never carried a useful signal. `web_0` 16K has stayed flat for about six cycles. Per-op-time and metadata-size have never been measured in nine cycles, so no cost claim is verified.
- **Open questions:**
  - A multi-bit reuse count that survives moves and sweeps (the main remaining idea for `web_0` 16K).
  - Admission based on how valuable the displaced resident is.
  - Placing ghost-returning promotees at the LRU end with the reference bit set.
  - An adaptive ghost reach.
  - The unexplained small `web_0` 128K giveback.
  - Making ghost lookups cheaper, which only matters once per-op-time is measured.
- **Per-policy notes:** These cover the stash, the ghost ring, normal buckets (LRU vs. CLOCK) and the probing tier. I added a short note on how the two traces differ, because `hm_0` and `web_0` keep pulling the same timing levers in opposite directions.


---

- finished: 2026-10-28T19:40:50+00:00
- duration: 27.26s
- status: ok
- input_tokens: 4
- output_tokens: 2469
- cache_read_tokens: 42588
- cache_write_tokens: 14550
- cost_usd: 0.174314
