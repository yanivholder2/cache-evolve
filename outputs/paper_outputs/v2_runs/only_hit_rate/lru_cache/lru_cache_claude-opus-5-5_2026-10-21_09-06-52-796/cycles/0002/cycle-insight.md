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
