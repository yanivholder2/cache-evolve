# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three variants branched from the baseline:
- **v001:** an admission-filtering policy (S3-FIFO).
- **v002:** a cheaper LRU approximation (CLOCK).
- **v003:** a representation-only control that keeps strict LRU and adds Fx hashing and `u32` links.

v001 won by raising hit rate on the scan- and loop-heavy workload-a. v002 roughly matched LRU's hit rate, and v003 left it unchanged, as designed.

## Patterns

- **Only hit rate was measured.** None of the variants reported `metadata-size` or `per-op-time`. Two of the three hypotheses (H02, H03) were aimed mainly at those dimensions, so their main claims are still untested.
- **Admission filtering is what helped.** On this trace, the gain came from changing which keys get admitted. Changing how recency is tracked did not help.
- **Exact LRU ordering adds nothing to hit rate here.** CLOCK's coarser second-chance recency held hit rate at LRU's level or slightly above. Precise LRU order can be given up without a hit-rate penalty.
- **All three variants remove or shrink the `prev`/`next` links.** Each predicts a smaller node and cheaper hits, but none has been measured. v001 also adds ghost structures stored outside `Node`, which could cancel its memory savings.

## Open questions

- Does v001's ghost queue and set add more per-key memory than dropping the links saves? And is its `get` actually faster?
- Does CLOCK's victim sweep cause tail-latency spikes on `put` when most entries are marked referenced?
- Are v003's representation changes (Fx hasher, `u32` links, MRU fast path) independent of policy? If so, they could be stacked onto v001 or v002 for a speed gain.

## Continuation

Re-measure `metadata-size` and `per-op-time` for v001, v002 and v003 before building on them. If the harness can't produce these dimensions, fix that first. Then use v001 as the parent and try:
- shrinking or compacting its ghost structures,
- grafting v003's hasher and narrow indices onto it,
- tuning the size of the small probation queue.

Don't spend further cycles on CLOCK-style recency tweaks unless measurement shows a clear memory or latency win.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Two variants changed cycle 1's S3-FIFO: v001 added a compact fingerprint ghost with a longer window, and v002 made the small queue adaptive in the style of ARC. A third, v003, built LIRS from scratch on the baseline. v001 was picked as the Winner, but it is only the best of a weak cohort. Both S3-FIFO changes lowered hit rate compared with their parent. LIRS beat LRU but did not reach the S3-FIFO hit rate.

## Patterns

- **Hit rate is still the only thing measured.** For the third hypothesis in a row, a claimed memory or speed win (v001's fingerprint ring) cannot be checked. LIRS's likely costs (a larger `Node`, a linear walk to enforce the ghost bound) cannot be checked either. The harness gap flagged in cycle 1 is still open.
- **Every change that made admission looser lowered hit rate.** v001's longer ghost window and v002's main-ghost fast path both let keys into `main` without proving reuse while they were resident. v002's growing `small` queue also exposed more of the cache to scans. Cycle 1 credited S3-FIFO's gain to strict admission filtering. This cycle supports that from the other side: weakening it costs hit rate.
- **A ghost hit does not mean the key will hit if kept.** On loops longer than the cache, a key can be re-referenced within the ghost window yet not within `main`'s residency. Lengthening the ghost window (v001) or treating ghost hits as a signal to resize (v002) both acted on this misleading signal.
- **The fixed ~10% probation split looks close to right.** v002 drifted away from it and lost hit rate. This answers cycle 1's question about sensitivity: hit rate depends on the split.
- **LIRS's idea of a protected set works, but in this form it is heavier and no better** than the simpler S3-FIFO filter on this trace.

## Open questions

- With the ghost window kept at the parent's size, does the fingerprint ring alone actually reduce `metadata-size` and `per-op-time`?
- Would LIRS with a larger HIR fraction and an O(1) non-resident FIFO close the gap to S3-FIFO, and what would it cost in memory?
- Are v001's stale-slot and early-decrement issues in the ring large enough to matter once the window is back to its original size?

## Continuation

Fix measurement of `metadata-size` and `per-op-time` before anything else. Further hit-rate-only cycles cannot judge the memory and speed claims that most hypotheses now make. Use cycle 1's v001 as the parent, not this cycle's. Then:
- keep the fixed ~10% probation split,
- try the fingerprint ring at the original ghost size, so the representation change is tested separately from the policy change,
- graft on cycle 1 v003's hasher and narrow indices.

Drop ARC-style adaptive sizing, longer ghost windows, and any other path that admits keys into `main` without proven reuse. Return to LIRS only once its cost can be measured and the linear bound-walk is fixed.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants changed a single policy knob. Two of them branched from cycle 1's S3-FIFO:
- **v001** shrank the ghost window to half of capacity.
- **v002** raised the probation promotion bar to two hits.

The third, **v003**, enlarged LIRS's resident-HIR queue. v001 won with a clear hit-rate gain. v002 was essentially flat, and v003 regressed slightly.

## Patterns

- **The ghost is the main lever on admission.** Cycle 2 lengthened the ghost and lost hit rate. This cycle shortened it and gained. Ghost size is a real tuning knob, and the parent's near-capacity ghost was too loose.
- **Probation strictness and ghost size can't be tuned separately.** With a large ghost, a key that v002 rejects from `small` comes back through the ghost-hit path straight into `main`. Its stricter bar only delays admission by one miss. Any test of "stricter admission" has to change the ghost path too.
- **Probation size doesn't explain why S3-FIFO beats LIRS.** Giving LIRS a bigger probation buffer cost protected LIR slots and didn't help. The gap between the two policies comes from somewhere else, most likely how each one handles ghost and non-resident re-admission.
- **Measurement is still missing.** For the third cycle in a row, only hit rate was reported. v001's smaller ghost probably saves memory, but that is unverified. Cycle 2's planned representation tests (fingerprint ring at the original ghost size, the hasher and narrow-index graft) were not attempted.

## Open questions

- Where is the optimum ghost size? Would going below half of capacity start dropping genuine re-references?
- Would sending ghost hits back into `small` instead of `main` beat simply shrinking the ghost? Does it make a stricter promotion bar meaningful?
- How much `metadata-size` does the smaller ghost actually save?

## Continuation

Use this cycle's v001 as the parent. Keep searching ghost size, going smaller (for example a quarter of capacity). Separately, test sending ghost hits into `small` rather than `main`. Revisit `PROMOTE_FREQ` only together with one of those ghost changes.

Drop the LIRS HIR-ratio tuning: restore the 1% split, or set LIRS aside. Measuring `metadata-size` and `per-op-time` remains the top harness priority. Once a ghost-size optimum is settled, run the representation-only experiments that cycle 2 asked for.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
