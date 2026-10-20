# Cycle 0006 Insight

## Cohort outcome

Three structural changes to the incumbent: a tag-first bucket scan (v001), shared-then-upgrade reader-writer locking (v002), and a small fixed-size doorkeeper ring replacing the in-item second-hit filter (v003). All were Winners. v001 and v002 held hit-rate and metadata flat and made latency worse. v003 showed a large latency drop but lost hit-rate.

## Patterns

- Cycle 0005's finding held for a fifth cycle: structural edits to probing and locking did not pay off. Tag compares cost as much as full u64 key compares, and the upgrade lock paid for two acquisitions on hits that need promotion.
- The bucket scan and the lock mode are now both unlikely to be where hit-path time goes. The benchmark looks effectively uncontended, so reader-writer sharing has nothing to exploit.
- v003 is the only latency gain, and it came with a hit-rate loss. The ring is lossy, so recurring keys were probably evicted from it before their second touch.
- v003's get-latency gain is implausible on its own, because get was not changed. This again points to measurement noise or layout effects, and the noise floor is still unmeasured.
- Winner status is again weak evidence. Flat hit-rate and metadata make a variant a Winner even when latency regresses.
- Variant authors still could not read the source and reasoned from fitness vectors alone, so the mechanisms above are inferred.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? This has been asked for three cycles and not answered.
- Is v003's put gain real (skipped insert and eviction work on declined first touches), and could a larger or aged doorkeeper keep that gain without the hit-rate loss?
- Where does hit-path time actually go, if not the scan, the lock or the hashing?
- Does a different trace, such as scans or frequency skew, change any of these conclusions?

## Continuation

Drop further bucket-scan tagging and reader-writer lock variants. Do not edit the hot path blind again. First establish the noise floor by repeating baseline runs, and profile where get and put time go, or evaluate the incumbent on a second trace. If a policy variant is tried, make it a better-sized or aged doorkeeper, judged by whether it keeps v003's put-side savings at the incumbent's hit-rate. Re-measure latency with repeats before crediting any gain.
