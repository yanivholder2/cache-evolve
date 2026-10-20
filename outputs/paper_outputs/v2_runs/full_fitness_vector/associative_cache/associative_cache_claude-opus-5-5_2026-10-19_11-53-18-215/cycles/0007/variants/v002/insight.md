# Variant v002 Insight

## Helped

- hit-rate (stayed within budget): Move-ahead-4 inside protected leaves protected *membership* unchanged. The probation→protected promotion, the 7/8 cap and the demote-at-cap rule are all untouched. Only the choice of which protected item gets demoted becomes approximate, so hit rate moved only slightly and stayed well inside the hypothesis's 0.002 loss budget.
- metadata-size: No change. `PROMOTE_STEP` is a compile-time constant and the 16-byte `Item` was kept, so the bounded promotion adds no per-item or per-cache state.

## Hurt

- per-op-time (get): The hypothesis predicted get-p50 would fall toward the 3/4-cap level, but get got slower at both p50 and p99. The likely reason is that the bound does almost nothing at this bucket geometry. With protected capped at 7 (8-slot bucket) or 14 (16-slot bucket), a full rotate was already only a few 16-byte items. The new `(pos + PROMOTE_STEP).min(last)` makes the rotate length variable, which costs about as much as the memmove it saves. A key that is hit once also no longer reaches MRU, so hot keys settle at mixed depths instead of clustering at the top.
- per-op-time (put): Put regressed sharply, especially at p99, even though the only put path the change touches is the resident-update `hit()` call, and that call now moves *less* data. The likely cause is indirect. Approximate protected order demotes still-warm items into the 1-slot probation, which means more probation evictions and ghost re-inserts. Each of those runs a whole-bucket `insert`/`remove` memmove. Part of the regression may also be benchmark noise, because none of put's own code paths changed.

## Takeaway

Bounding protected-hit promotion to move-ahead-4 kept the 7/8-cap hit rate almost intact, but it did not recover any of 0006 v001's latency. That rules out "long protected rotates on deep hits" as the main source of the 7/8-cap get slowdown. With 16-byte items and protected segments of 14 slots or fewer, the rotate is already cheap, so the latency cost of the 7/8 cap must come from somewhere else. The thin probation segment and the churn it causes in put's insert/evict paths are the more likely target. More approximate recency inside protected is not the fix.
