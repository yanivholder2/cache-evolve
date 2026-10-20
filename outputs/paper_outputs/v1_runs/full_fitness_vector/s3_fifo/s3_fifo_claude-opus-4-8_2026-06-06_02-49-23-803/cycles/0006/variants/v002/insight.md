# Variant v002 Insight

## Helped

- per-op-time: scanning `main` before `small` in `get`/`find_entry_mut` shortens the expected probe length for the common case — hits land on proven keys in main — so the scored get p50 (the summary) drops below the parent's long-stuck floor.

## Hurt

- per-op-time: the same reorder pushes get p99 and put p50/p99 up; with the short small queue now walked last, the rarer small-resident and miss paths (and put's lookup) traverse main fully first, widening the tail.

## Takeaway

Reordering the resident scan to visit `main` first is a pure ordering change (identical key set, so hit-rate and the 24-byte metadata are untouched) that finally moves the long-stuck get p50 floor — confirming that floor was scan-order-sensitive and dominated by main-resident hits walking the small queue first. The win is real on the median but redistributes cost to the tail: p99s and put lookups, which now pay the full main walk before reaching small.
