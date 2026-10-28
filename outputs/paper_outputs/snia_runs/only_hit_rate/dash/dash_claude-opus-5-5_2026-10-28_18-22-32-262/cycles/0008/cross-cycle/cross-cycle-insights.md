# Cross-Cycle Insights (cycles 0001–0008)

## Recurring wins

- **Don't put a newly proven item in the victim slot.** This placement rule has won four times: MRU-end stash demotion (c1), `Back` target promotion (c3), and `Back` probing promotion (c5 v001, stacked in c6). Fixing where an item lands paid off more reliably than tuning the rules for admitting it.
- **Reuse history that outlives eviction beats counters kept on the item.** The FIFO ghost ring of stash-evicted keys (c5), sized at factor 2 and consumed on re-put, was the first lever to move `web_0` 128K. Its lifecycle was settled by c6–c7: changing the length either way or spending its credit twice did not help.
- **A strict room threshold for bypassing the gate.** Skipping the two-hit gate only when the target has two free slots (c4) kept the `web_0` 128K gain without the 16K cascade loss.
- **Moves that use free slots and displace no one are safe.** The stash-overflow rescue (c7) regressed nowhere, though it only pays off at 128K.
- **Plain LRU holds up.** It is confirmed for stash eviction (c4). In full 2-slot normal buckets, a CLOCK reference bit ends up choosing almost the same victims (c8).
- **The lineage climbed steadily.** Each cycle's winner stacked on the previous one: stash `Back` demotion, then the two-hit gate, `Back` promotion, the occupancy bypass, the ghost ring, probing `Back`, the rescue, and CLOCK. Recent gains are marginal.

## Recurring failures

- **Easier routes back into a crowded target.** One-hit return (c3 v001), last-free-slot shortcut (c3 v002), and ghost credit on demotion (c6 v003) all lost hit-rate, mostly at 16K, because a taken slot pushes an eviction cascade onto the next promotion.
- **Counters stored on items.** They either stay near zero because they reset or halve on moves (c2), or they never decay and pin items permanently (c4 v003). Every attempt at frequency admission or counter-based victim choice has failed.
- **Changing the target→probing trigger.** Both delaying it (c4) and speeding it up (c5) lost hit-rate. The trigger looks near its optimum.
- **Hard LRU-end insertion of fresh puts** (c2 v003). New keys never get their probation hit.
- **Fixed changes to ghost reach.** A 4× ring credited returns that came too late (c6). A factor-1 ring helped `web_0` but cost `hm_0` 128K (c7).
- **Routing more items into free slots after the rescue exists** (c8 v003), and **occupancy-conditioned credit** (c8 v002). These only reorder how the same headroom gets spent.
- **Foreign-resident retry changes** (c7 v001) had no effect at all.
- **Cost was never measured.** Per-op-time and metadata-size have gone unmeasured for 8 cycles straight, so every cost claim (lookup reorder, ghost metadata, extra scans) is still unverified.

## Open questions

- **What limits `web_0` 16K?** It has been flat across gate, ghost, ghost reach, placement, rescue, and victim-choice levers. Untested candidates:
  - not moving items to MRU on a hit, so the reference bit becomes the real recency signal
  - a victim signal that carries history across stash stays
  - stash size relative to the segment
- **What causes the small `web_0` 128K giveback after probing `Back` (c6)?** Foreign retries, ghost reach, and stash→full-target promotions are ruled out. Ghost churn caused by probing-eviction cascades is still a suspect.
- **Should admission and credit depend on the value of the resident being displaced** (its recency or hit count) rather than on whether a slot is free? `hm_0` 128K shows that displacing residents can pay.
- **Could adaptive ghost reach** (per trace or occupancy-conditioned) get both `web_0`'s short-ring gain and `hm_0` 128K's long-tail gain?
- **Does the rescue silently turn bypass promotions into gated ones** by filling targets below the free-slot threshold?
- **Does the target-first lookup order (c1 v003) lower latency?** Its hit-rate is identical to baseline.

## Per-policy notes

- **Stash (bucket-local LRU):**
  - Use `Back` demotion, LRU eviction, the two-hit exit gate with a two-free-slot bypass, and a factor-2 FIFO ghost consumed on re-put.
  - Rescue overflow victims into free slots only.
  - Don't change victim selection or insertion position again.
- **Normal / target buckets (2-slot):**
  - Promotees land MRU.
  - Victim choice under saturation is effectively LRU, so a new signal must carry information that recency order doesn't already have.
  - Never let a key that has lost its slot outrank residents at 16K.
- **Probing tier:**
  - Promote with `Back` placement and keep the current trigger.
  - Gains here concentrate on `hm_0`, whose hot set actually reaches probing.
  - It interacts with `web_0` 128K through foreign-resident cascades.
- **Trace/size regularities:**
  - `web_0` 128K behaves like an admission problem: returning keys are the issue, and spare target room makes first-hit promotion near-optimal.
  - `web_0` 16K is saturated and resists everything tried so far.
  - `hm_0` rewards protecting residents and a longer ghost tail, and it is the cell most sensitive to regressions.
  - Free-capacity levers pay off only at 128K.
