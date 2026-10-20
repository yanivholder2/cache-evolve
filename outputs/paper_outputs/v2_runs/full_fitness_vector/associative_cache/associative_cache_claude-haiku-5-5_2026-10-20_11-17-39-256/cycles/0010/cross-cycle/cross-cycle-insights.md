# Cross-cycle insights

## Recurring wins

- Retention signals that separate re-referenced keys from one-shot inserts have held hit-rate gains on workload-a across frequency, second-chance, and multi-level re-reference variants. This is the most consistent hit-rate lever in the ledger.
- Exact global-minimum victim choice over the full bucket is load-bearing. Tie-break order and rotating starts do not matter, but the minimum itself does.
- Narrower victim-scan width (fewer slots per bucket) lowered put cost with hit-rate flat. It is the only put-side change in many cycles with a clear, direction-consistent effect.
- Moving reference state into a per-bucket mask keeps hit-rate and cuts metadata. The time cost came from the scan form, and a word-level scan recovered most of the put median.
- Behavior-preserving rewrites of existing mechanisms can recover timing when the earlier regression was in the code form, not the design.

## Recurring failures

- Admission gates (doorkeeper variants, put-miss threshold gates) have gained, lost, and barely moved hit-rate across cycles. Their bookkeeping costs land on put and get paths they do not target. Not a dependable lever.
- Shortcuts to the victim search (rotating window, cached minimum, cached two-candidate pair, early exit on zero counters, merged scan with halving, narrowed scan) have not reduced the put tail. Several cost hit-rate because they weaken the exact-minimum view.
- Decay-schedule changes (extra miss-driven decay, lazy per-bucket halving) move the decay cost into ordinary operations and do not fix put tail. The periodic global halving is not the put-tail source.
- Aging rewrites (bulk-add form) made put slower; the aging rounds were not the dominant put-tail cost.
- Placement changes inside a bucket (midpoint promotion and reinsertion) and ghost-memory lengthening did not lift hit-rate. The reinsertion idea looks weak on this trace.
- Fingerprint prefilters added a second scan and write work with no hit-rate change.
- Hash or bucket-index changes do not move hit-rate; their timing movement is small and not yet reproduced.

## Open questions

- What drives the put tail on full-bucket puts? Many cycles of scan, aging, and decay changes have not moved it, while narrower width did. Is the put cost simply proportional to scan width?
- Does the retention gain and the narrower-width gain hold on traces other than workload-a and at other cache sizes? Every hit-rate result so far is single-trace.
- Why did the doorkeeper gain in one cycle and lose in another? Is the parent or the exact reject rule responsible?
- Are the recurring get-tail and put-timing shifts real, or run-to-run spread and struct-layout effects? Most evidence is single-run.
- Does the saturated-counter store skip's get-tail gain survive repeated runs, and does it hold when the get median is considered?
- The metadata-size probe has not registered per-bucket or per-slot state. Until it counts that state, storage cost comparisons are unreliable.
- What is the minimum counter ceiling and scan width at which hit-rate starts to fall?

## Per-policy notes

- Frequency-based victim selection: the hit-rate gain comes from protecting re-referenced keys, but the full-bucket minimum scan is the put-side cost. Shortcuts to that scan have not worked. A lower counter ceiling kept hit-rate but did not speed the paths.
- Second-chance / reference-bit: the strongest combined result so far, improving both hit-rate and put cost. Per-item state grew beyond one bit. Moving state into a per-bucket mask kept hit-rate and cut metadata, but needed a scan-form fix to avoid a time regression. Multi-level re-reference values gained hit-rate with a bounded aging cost on put.
- Bucket-local ghost and midpoint variants: no hit-rate win across the cycles that tried them. Treat as closed unless a different reuse signal is proposed.
- Admission gating (doorkeeper, threshold on victim counter): inconsistent hit-rate effect and added overhead. Do not build on it without reconciling the conflicting cycles.
- Decay and halving schedule: keep the periodic halving as is. Per-operation or lazy variants moved cost onto the hot path without a hit-rate benefit.
- Bucket geometry (slots per bucket, hash placement): narrower buckets are the current put-side timing lever with flat hit-rate. Hash changes are timing-only candidates pending repeated runs.
- Winner labels: several listed Winners (for example in Cycles 0007, 0008, 0009, and 0010) rest on single-run or small timing movement, or carry regressions in their own insights. Re-verify before relying on them.
