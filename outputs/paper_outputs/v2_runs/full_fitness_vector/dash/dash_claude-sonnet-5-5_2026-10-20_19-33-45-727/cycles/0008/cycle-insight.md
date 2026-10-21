# Cycle 0008 Insight

## Cohort outcome

Three admission-filter variants on different parents: a one-cache-line aged two-probe doorkeeper (v001), unconditional admission while a segment has free slots (v002), and a tiny direct-mapped fingerprint doorkeeper (v003). All were Winners. Only v001 was a clean improvement: put latency fell and hit-rate held or rose slightly, with metadata flat. v002 gained nothing and regressed latency. v003 cut latency but lost hit-rate.

## Patterns

- Shrinking the doorkeeper to one cache line fixed the cycle 0007 trade-off. Footprint and reset method, not the filtering itself, drove put cost, so a small filter aged often enough to stay sparse can match a large one.
- Cheap put-side wins come from skipping insert and eviction on one-hit keys. A lossy filter without aging (v003) pays for it in hit-rate, as in cycle 0006. Aging and sparsity are what keep a small filter accurate.
- Warm-up loss is not the hit-rate limit. Admitting first touches into free space did nothing, since the cache fills quickly and the filter works almost entirely in the full-segment regime.
- v002's large get and put regression on an untouched or trivially changed path is again most likely layout or measurement noise. The noise floor is still unmeasured, and v001's small get gain and hit-rate gain may be noise too.
- Variant authors for v001 and v002 could not read the source, so the mechanisms are inferred from fitness vectors and the hypotheses.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? This has now been asked for five cycles.
- Is v001's hit-rate gain real or within noise?
- Would a cache-line-sized filter with a different aging threshold, or a fingerprint filter with aging, push the hit-rate and put cost trade-off further?
- Does v001's gain hold on a second trace with scans or frequency skew?

## Continuation

Adopt v001 (the one-line aged doorkeeper) as the new incumbent for policy work. Drop warm-up-admission variants and unaged fingerprint tables. Re-measure v001 and its parent with repeated runs to set the noise floor before crediting any latency gain, and check it on a second trace. If a further variant is tried, tune v001's aging threshold or probe count, and avoid hot-path edits to get.
