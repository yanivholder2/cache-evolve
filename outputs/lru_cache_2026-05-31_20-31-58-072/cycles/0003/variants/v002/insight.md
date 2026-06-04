# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Adding a single-bit `hot_credit` to gate Hot→Cold demotion behind two consecutive ref-clear visits did not move hit-rate on lirs:multi1@512 — the parent's score reproduces exactly. The change lives entirely inside pass 2 of `evict_one` (plus the ghost-promoted insert path), but pass 1 already drains a Cold ref-clear victim on essentially every eviction at this trace/size, so pass 2 rarely fires and the credit never gets consumed. The lesson is that on this workload Hot-demotion aggressiveness is not the bottleneck the parent insight named; protecting Hot slots harder only matters when pass 1 is forced to fail, and at cap=512 the Cold pool stays large enough that it doesn't.
