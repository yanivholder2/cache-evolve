# Cycle 0009 Insight

## Cohort outcome

The cohort tried three changes: skipping the access-counter store on saturated get hits (v001), replacing the periodic global halving with lazy per-bucket halving (v002), and gating put-miss admission on the victim's counter at threshold T=1 (v003). None produced a clear hit-rate gain, and only v001 has any favorable timing movement, which is small. The listed Winners (v002, v003) have no helped dimensions in their own insights.

## Patterns

- Admission gating again showed no hit-rate effect. v003 refuses inserts only when every slot in a full bucket already has a counter of at least 2, so it touches few buckets. This matches Cycles 0006–0008: the admission gate has not produced a dependable hit-rate change.
- Decay scheduling is not the put-tail source. v002 kept hit-rate and metadata flat, yet moved the halving cost into ordinary put and get calls and made both tails worse. The whole-structure pass was not what drove the put tail, and the cost of decay is paid per operation wherever it runs.
- The saturated-counter store skip (v001) again shows a small get-tail gain, but its get median ticked up, so it does not beat its parent on the headline number. Its put-side movement is not explained by its edit and is probably run-to-run spread.
- Across the last several cycles, get-tail effects appear with no get-path change or with a change that adds per-op work. Single-run tails remain the dominant evidence, so these effects are not settled.
- Mechanisms are inferred from diffs and variant insights, not confirmed from overlay code, because overlays were not readable this cycle.

## Open questions

- Does the saturated-counter store skip's get-tail gain survive repeated runs, and does it hold when the get median is also considered?
- How often does the v003 refusal branch fire on workload-a? A near-zero rate would mean the victim choice already captures the frequency signal.
- What is the put-tail source on full-bucket puts? Six cycles of shortcuts to the scan, aging, and decay have not moved it.

## Continuation

Next cycle should not pursue admission gating or decay-schedule changes as hit-rate levers. If the saturated-counter store skip is retried, measure it with repeated runs and judge it on the get median as well as the tail. Drop the lazy per-bucket halving (v002) and the T=1 admission gate (v003), and treat the Winner labels on both as unsupported by this cycle's evidence.
</content>
