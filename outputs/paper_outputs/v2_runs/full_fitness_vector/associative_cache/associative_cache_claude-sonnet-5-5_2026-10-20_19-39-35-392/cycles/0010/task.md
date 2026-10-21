# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0010

## Winners

- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Variants tried put-path-only adaptive insertion depth (v001), a position-gated two-slot hit promotion (v002), and a fixed len/4 swap insertion with first-empty/in-place victim overwrite as a control (v003). v001 and v002 were Winners, but none shows a clear gain: v001 lost hit rate and time, v002 gained marginal hit rate at a latency cost, and v003 (not a Winner) got cheaper puts but lost hit rate.

## Patterns

- The Winner-versus-evidence mismatch continues, a seventh straight cycle. v001 reports losses on hit rate and on both latencies, and v002's own insight calls its trade weak.
- Moving the adaptive signal off the get path did not make it free. The put-side counter and data-dependent branch cost put time, and the victim-position proxy was a noisier signal than static len/4.
- The fixed-position control answers last cycle's question. Swap insertion at a fixed len/4 cuts put cost but loses hit rate, because the swap pushes the displaced mid-bucket entry to the tail where it is evicted next. Any earlier adaptive gain therefore came from the adaptive part, and v001's weaker proxy did not reproduce it.
- Position as an implicit "hit before" signal is free in metadata but not in time. Faster promotion for already-promoted entries again gives only marginal hit rate, matching cycles 0007 and 0008: transpose-by-one is hard to beat.
- Get latency again has no real gain; every extra compare or swap on the hit path made it slower.
- No variant grew metadata.

## Open questions

- What signal made the cycle 0008 adaptive scheme work (global hit/miss counter), and can it be updated cheaply without a per-get cost?
- Can swap insertion avoid sacrificing the displaced entry, e.g. by shifting rather than swapping into the tail, while keeping the put saving?
- Are any Winner statuses beyond noise, and do results hold on a second workload?

## Continuation

Drop victim-position proxies for adaptivity, position-gated promotion jumps, and fixed-position swap insertion as a hit-rate idea (keep it only as a put-cost option). Next, retry adaptivity with the cycle 0008 global hit/miss signal, or test a swap variant that does not demote the len/4 occupant to the tail. Add a noise check and a second workload, and keep discounting Winners whose own insight shows a loss.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
