# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0007

## Winners

- v002
- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_19-39-35-392/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Variants targeted cost reduction around the mid-insertion/transpose baseline: a per-bucket ghost admission filter (v001), O(1) swap-based mid-insertion (v002), and single-hash/mask indexing with tag-first scan (v003). v002 was a clear Winner and v001 the other Winner on thin evidence, since its own insight reports a hit-rate loss and slower puts. v003 sped up put only.

## Patterns

- Put-path cost was the lever that moved. Removing the half-bucket shift (v002) and redundant hashing/modulo (v003) both cut put latency, with hit rate and metadata unchanged.
- Approximate ordering among displaced entries does not matter. Where the new entry lands matters, not the exact order of the rest, so exact bucket ordering is not worth paying for.
- A one-entry-per-bucket ghost is too weak a signal. It added put work and slightly hurt hit rate, penalizing reusable keys along with scan items.
- Get latency did not improve through any real mechanism. This is the fourth cycle with no get gain. Arithmetic, hashing and item movement are now all ruled out, and small get shifts look like noise or codegen effects.
- No variant grew metadata.

## Open questions

- Is the v001 Winner status real, or within noise given the negative insight?
- Is get time dominated by memory access to the bucket (cache lines, slot layout, tag/key separation)? It is untested.
- Would a deeper or more selective admission history help, or is scan resistance already saturated by insertion position?
- Do any gains hold on a workload other than workload-a?

## Continuation

Adopt v002's swap-based mid-insertion as the policy baseline and fold in v003's hash-once indexing only if it can be made to help rather than hurt get. Drop single-entry ghost filters and further arithmetic or hash micro-optimizations. Aim the next hypotheses at memory layout for get (cache-line alignment, packing tags or keys contiguously, prefetch) and validate on a second workload.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
