# Cycle 0008 Insight

## Cohort outcome

The cohort tried three levers: confining the packed-flag mask to one inlined index helper with a raw-OR REFERENCED set on hits (v001), collapsing the `put` miss path onto a single key-map lookup (v002), and admitting ghost-ring hits at probation instead of protected (v003). v001 is the only variant whose insight records an improved dimension, so it is the credible Winner. v002 carries a Winner label but its insight records no improvement and a worse `put` tail, so the label is not supported. v003 lost hit rate and is not a Winner.

## Patterns

- Consolidating the masked index reads (v001) recovered per-operation cost on the packed layout without changing policy or metadata size. The mask still runs on every read, so this addresses the call-site cost from cycle 0007 but not the mask itself.
- The single-lookup rewrite of the `put` miss path (v002) did not reduce `put` cost. The forced remove-then-insert ordering appears to absorb the saved hash, so the second key hash is not the dominant `put` cost.
- Ghost-ring admission has now lost hit rate in both directions: to protected in cycle 0007 and to probation here, on workload-a at 2048. The ghost idea has no working admission target on that trace.
- `put` tail movement appears again across variants (v002 and v003 here, and several earlier cycles), while `get` tails moved without a `get` change in v002. This pattern points to measurement noise or a shared layout effect more than to any single diff.
- Two of the three insights rest on hypothesis text and fitness vectors because the source diffs could not be inspected. Their code-level claims are inferred.

## Open questions

- Is v001's gain carried by the raw-OR REFERENCED set on hits, by the helper consolidation, or by both?
- Is v002's `put` tail regression real, given that its `get` tail also moved without a `get` change?
- Does the ghost-ring hit-rate loss hold on workloads other than workload-a at 2048?
- Are the recurring `put` tail movements a run-to-run artifact of the measurement setup?

## Continuation

Next cycle, build on v001's single-helper index access as the packed-flags base, and test whether the raw-OR REFERENCED set on hits is what drives its gain. Drop the ghost ring as an admission lever on workload-a, since neither promotion target held hit rate. Do not pursue the single-lookup `put` rewrite, and re-measure `put` tails before crediting or blaming any further `put`-path change.
