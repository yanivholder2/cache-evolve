# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-28T19:29:59+00:00

## timeline

### 19:30:20 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-28_18-22-32-262/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (cycles 0001–0008)\n\n## Recurring wins\n\n- **Don't put a newly proven item in the victim slot.** This placement rule has won four times: MRU-end stash demotion (c1), `Back` target promotion (c3), and `Back` probing promotion (c5 v001, stacked in c6). Fixing where an item lands paid off more reliably than tuning the rules for admitting it.\n- **Reuse history that outlives eviction beats counters kept on the item.** The FIFO ghost ring of stash-evicted keys (c5), sized at factor 2 and consumed on re-put, was the first lever to move `web_0` 128K. Its lifecycle was settled by c6–c7: changing the length either way or spending its credit twice did not help.\n- **A strict room threshold for bypassing the gate.** Skipping the two-hit gate only when the target has two free slots (c4) kept the `web_0` 128K gain without the 16K cascade loss.\n- **Moves that use free slots and displace no one are safe.** The stash-overflow rescue (c7) regressed nowhere, though it only pays off at 128K.\n- **Plain LRU holds up.** It is confirmed for stash eviction (c4). In full 2-slot normal buckets, a CLOCK reference bit ends up choosing almost the same victims (c8).\n- **The lineage climbed steadily.** Each cycle's winner stacked on the previous one: stash `Back` demotion, then the two-hit gate, `Back` promotion, the occupancy bypass, the ghost ring, probing `Back`, the rescue, and CLOCK. Recent gains are marginal.\n\n## Recurring failures\n\n- **Easier routes back into a crowded target.** One-hit return (c3 v001), last-free-slot shortcut (c3 v002), and ghost credit on demotion (c6 v003) all lost hit-rate, mostly at 16K, because a taken slot pushes an eviction cascade onto the next promotion.\n- **Counters stored on items.** They either stay near zero because they reset or halve on moves (c2), or they never decay and pin items permanently (c4 v003). Every attempt at frequency admission or counter-based victim choice has failed.\n- **Changing the target→probing trigger.** Both delaying it (c4) and speeding it up (c5) lost hit-rate. The trigger looks near its optimum.\n- **Hard LRU-end insertion of fresh puts** (c2 v003). New keys never get their probation hit.\n- **Fixed changes to ghost reach.** A 4× ring credited returns that came too late (c6). A factor-1 ring helped `web_0` but cost `hm_0` 128K (c7).\n- **Routing more items into free slots after the rescue exists** (c8 v003), and **occupancy-conditioned credit** (c8 v002). These only reorder how the same headroom gets spent.\n- **Foreign-resident retry changes** (c7 v001) had no effect at all.\n- **Cost was never measured.** Per-op-time and metadata-size have gone unmeasured for 8 cycles straight, so every cost claim (lookup reorder, ghost metadata, extra scans) is still unverified.\n\n## Open questions\n\n- **What limits `web_0` 16K?** It has been flat across gate, ghost, ghost reach, placement, rescue, and victim-choice levers. Untested candidates:\n  - not moving items to MRU on a hit, so the reference bit becomes the real recency signal\n  - a victim signal that carries history across stash stays\n  - stash size relative to the segment\n- **What causes the small `web_0` 128K giveback after probing `Back` (c6)?** Foreign retries, ghost reach, and stash→full-target promotions are ruled out. Ghost churn caused by probing-eviction cascades is still a suspect.\n- **Should admission and credit depend on the value of the resident being displaced** (its recency or hit count) rather than on whether a slot is free? `hm_0` 128K shows that displacing residents can pay.\n- **Could adaptive ghost reach** (per trace or occupancy-conditioned) get both `web_0`'s short-ring gain and `hm_0` 128K's long-tail gain?\n- **Does the rescue silently turn bypass promotions into gated ones** by filling targets below the free-slot threshold?\n- **Does the target-first lookup order (c1 v003) lower latency?** Its hit-rate is identical to baseline.\n\n## Per-policy notes\n\n- **Stash (bucket-local LRU):**\n  - Use `Back` demotion, LRU eviction, the two-hit exit gate with a two-free-slot bypass, and a factor-2 FIFO ghost consumed on re-put.\n  - Rescue overflow victims into free slots only.\n  - Don't change victim selection or insertion position again.\n- **Normal / target buckets (2-slot):**\n  - Promotees land MRU.\n  - Victim choice under saturation is effectively LRU, so a new signal must carry information that recency order doesn't already have.\n  - Never let a key that has lost its slot outrank residents at 16K.\n- **Probing tier:**\n  - Promote with `Back` placement and keep the current trigger.\n  - Gains here concentrate on `hm_0`, whose hot set actually reaches probing.\n  - It interacts with `web_0` 128K through foreign-resident cascades.\n- **Trace/size regularities:**\n  - `web_0` 128K behaves like an admission problem: returning keys are the issue, and spare target room makes first-hit promotion near-optimal.\n  - `web_0` 16K is saturated and resists everything tried so far.\n  - `hm_0` rewards protecting residents and a longer ghost tail, and it is the cell most sensitive to regressions.\n  - Free-capacity levers pay off only at 128K.\n"
}
```

### 19:30:26 chat

I wrote `cross-cycle-insights.md`, built fresh from the eight-cycle ledger. It uses the four required sections in order and quotes no numbers.

The main patterns it records:

- **What keeps working:** don't put an item that just proved itself into the slot that gets evicted next. That placement rule has won four times. A short memory of recently evicted keys was the first change to move `web_0` at 128K. LRU is still the right eviction rule. Each cycle's winner built on the last, but recent gains are marginal.
- **What keeps failing:** making it easier for an item to get back into a full bucket, hit counters stored on the items themselves, changing when items move up to the probing tier, and changing how many evicted keys are remembered.
- **Still unmeasured:** per-op-time and metadata-size have not been reported in any of the eight cycles, so every claim about cost is unverified. The file flags this as a standing failure.
- **Open questions:**
  - `web_0` at 16K has stayed flat across almost every change tried so far.
  - The small `web_0` 128K loss that started in cycle 6 is still unexplained.
  - Admission and credit decisions might work better if they weighed the value of the item that would be pushed out, rather than just checking for a free slot.

The per-policy section groups these by stash, normal/target buckets, the probing tier, and how each trace and cache size behaves.


---

- finished: 2026-10-28T19:30:27+00:00
- duration: 27.73s
- status: ok
- input_tokens: 4
- output_tokens: 2534
- cache_read_tokens: 41550
- cache_write_tokens: 13696
- cost_usd: 0.168574
