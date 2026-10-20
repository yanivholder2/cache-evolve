# Variant v003 Insight

## Helped

- None: no fitness dimension improved relative to the parent.

## Hurt

- hit-rate: Setting `freq = 1` on ghost-hit re-admissions gives each re-admitted key one extra second-chance pass in `evict_main`, and on lirs workload-a at capacity 2048 that pass displaced more useful main-queue residents than the re-admitted keys returned as hits.

## Takeaway

Carrying a frequency of 1 into main on ghost re-admission, the path H03 targeted, is a net loss on this trace: the premise that re-requested keys deserve an extra main pass did not hold. The change is confined to the ghost branch in `put`, so the drop is attributable to that admission frequency alone. The variant's fitness vector reports only hit-rate, so the metadata-size and per-op-time cost expectations in H03 remain unverified.
