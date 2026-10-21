# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0008

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Variants probed memory layout for get (a contiguous tag-first scan, 64-byte bucket alignment) and nudged the insertion point from len/4 to len/3. None improved get. v001 and v002 were Winners, but both on thin evidence, since their own insights report a hit-rate loss and a slower put (v001), or a hit-rate loss with an unexplained put gain (v002).

## Patterns

- Get latency again did not move, now across five cycles. Arithmetic, hashing, item movement and layout (tags, alignment) are all ruled out. Buckets are probably small and cache-resident, so get is likely dominated by something outside bucket memory access.
- Layout changes that must stay in sync with promotions and swaps (parallel tags) raise write cost and risk drifting from the parent's policy. Pure allocation-placement changes (alignment) are safe, but their benefit showed up only on put tail latency.
- Insertion depth is a free hit-rate knob. Moving from len/4 toward the protected end lowered hit rate, which fits the earlier saturation finding that len/4 or lower is the useful range.
- Put-latency gains without a mechanistic reason (v002) are the same noise pattern seen in cycles 0001 and 0002.
- No variant changed metadata size.

## Open questions

- Are the v001 and v002 Winner statuses real, or are they artifacts of noisy put latency and a flat get?
- Does the optimum insertion point lie below len/4, such as len/5 or len/8?
- Where does get time go if not memory layout, hashing or reordering? It may be harness overhead, dispatch or the key and value path.
- Does cache-line alignment's put-tail gain repeat, and does it stack with swap-based insertion?
- Do any gains hold on a second workload?

## Continuation

Stop attempting get-latency gains through bucket layout, tags or hash arithmetic; the evidence says they do not work. Keep the swap-based len/4 insertion as the policy baseline, try slightly lower insertion points, and consider adopting plain 64-byte alignment as a cheap put-tail improvement after re-checking it. Add a second workload to validate any claimed Winner, and discount Winners whose own insight shows a loss.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
