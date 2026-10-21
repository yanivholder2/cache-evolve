# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0009

## Winners

- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Variants tried a lower insertion point (len/5, v001), plain 64-byte bucket alignment on top of swap insertion (v002), and a rebuilt stateless mid-bucket insertion with transpose promotion directly on baseline (v003). All three were Winners. Only v003 shows a plausible mechanism-backed gain, a hit-rate improvement with a put-tail cost. v001 and v002 have insights that report a loss.

## Patterns

- Winner status is again weakly tied to the variants' own evidence. v001 reports lower hit rate and slower get and put. v002 reports slower get and put with a hit-rate gain that has no plausible cause. This is the fourth straight cycle where Winners include variants whose own insights are negative.
- Insertion position looks saturated on both sides. Moving from len/4 toward len/3 (cycle 0005) and toward len/5 (this cycle) both failed to help, so len/4 is a local optimum for position-only tuning.
- Cache-line alignment, which cycle 0005 suggested might help put-tail latency, did not repeat. It made latency worse and its hit-rate change was unexplained. Allocation-only layout tweaks are not a reliable win.
- v003 independently reproduces the mid-insertion plus transpose result from cycle 0003. Stateless scan resistance works, and its cost is data-dependent shuffling in the put tail.
- Get latency again did not improve. This is now six cycles with no get gain.
- No variant changed metadata size.

## Open questions

- Are any of these Winner statuses real, or do they come from noisy latency and a flat get?
- Does v003's put-tail penalty vanish when it is combined with swap-based insertion at len/4?
- Where does get time go, given that position, layout, hashing and reordering are all ruled out? It may be harness or dispatch overhead.
- Does anything hold on a second workload?

## Continuation

Drop further tuning of the insertion constant and drop alignment or other layout work; both are exhausted. Treat swap-based len/4 insertion with transpose promotion as the settled baseline. Aim the next hypotheses at a different mechanism, such as the hit-promotion path or per-slot signals under the 40-byte metadata budget. Also add a second workload and a noise check, and discount Winners whose own insight shows a loss.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Variants moved off insertion tuning and probed other policy paths: a geometric halving jump for hit promotion (v001), skipping reorder on put-of-existing-key with a single-scan restructure (v002), and a stateless hash-parity choice between the last two slots for eviction (v003). All three were Winners, but none shows a real gain in its own insight.

## Patterns

- Winner status is again disconnected from the variants' evidence. v001 and v003 report hit-rate losses, v003 also reports slower put and get, and v002 reports no median put gain and a worse put tail. This is the fifth straight cycle of this pattern.
- Gradual one-step transpose promotion acts as a frequency filter. Faster promotion removes that filter and hurts hit rate on the scan-heavy trace, so the parent's promotion looks already tuned.
- Stateless randomization carries no reuse signal. Hash-bit victim choice only adds noise and an unpredictable branch. Scan-resistance information must come from recency or frequency, not key-hash bits.
- Put-hit reordering is not a meaningful cost, since the workload's puts are mostly misses. Restructuring put-hit control flow adds variance and no saving.
- Get latency still has no real gain, now seven cycles. Small get shifts in v001, v002 and v003 are attributed to noise or layout.
- No variant changed metadata size.

## Open questions

- Are any of the three Winner statuses real, or are they noise in tail latency?
- Is there a middle ground on promotion, such as a jump only after a second hit, that keeps the frequency filter?
- Can any signal that reflects reuse fit in the 40-byte metadata, or is the budget the real limit?
- Where does get time go? Harness or dispatch overhead is untested.
- Do any results hold on a second workload?

## Continuation

Drop geometric or aggressive promotion, stateless hash-based eviction, and put-hit control-flow tweaks. Keep swap-based len/4 insertion with transpose-by-one promotion as the baseline. Aim the next cycle at the put-miss and insertion path or at a small real reuse signal under the metadata budget. Add a noise check and a second workload, and discount Winners whose own insight shows a loss.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Variants tried a one-bit "seen once" gate on transpose promotion (v001), a fused single-pass put scan with swap insertion (v002), and an adaptive len/4 vs len/2 insertion point driven by a global hit/miss counter, built on baseline (v003). All three were Winners, but only v003 shows a real gain: better hit rate and clearly cheaper puts, at a slightly higher median get.

## Patterns

- The Winner-versus-evidence mismatch continues, a sixth straight cycle. v001 lost on hit rate and on both get and put latency. v002 got slower on both paths, and its hit-rate gain came from an unintended change in behavior, not from design.
- A second reuse filter on top of transpose promotion does not pay. This matches cycle 0007: transpose is already a gradual frequency filter, and extra gating only delays hot entries and adds per-comparison cost.
- Micro-fusing the put path is not a safe latency win. It slowed put and get, probably through code layout or extra per-slot work, and it quietly changed eviction and placement. Placement in the bucket seems to matter more than scan speed.
- v003 again shows that mid-bucket swap insertion plus transpose promotion beats LRU, now reproduced from baseline a third time. Its adaptive counter fit in existing padding, so metadata did not grow. Its insight cannot separate the adaptive switch from the fixed mid-insertion mechanics.
- Get latency still has no real gain. Every per-hit addition (a flag, a counter) taxed the hot path.
- Only v001 and v003 touched metadata, and neither grew it.

## Open questions

- How much of v003's gain comes from the adaptive switch, and how much from swap insertion at a fixed position? A fixed-position control is missing.
- Does v002's unintended placement change, such as first-empty-slot fill or overwriting the last victim, help hit rate, and can it be adopted deliberately?
- Can adaptivity be driven from the put path alone, to keep the get hot path untouched?
- Do any results hold on a second workload, and are the Winner statuses beyond noise?

## Continuation

Drop reuse-flag gating on promotion and put-scan fusion; both are exhausted. Pursue adaptive insertion from v003 as the lead, and add a fixed-position control to isolate it. Prefer signals updated on the put-miss path, since per-get work costs median get time. Also test v002's empty-slot fill and victim placement as a deliberate policy. Keep discounting Winners whose own insight shows a loss, and add a second workload and a noise check.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
