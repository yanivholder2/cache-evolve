## Recurring wins

- Retention signals that separate re-referenced keys from one-shot inserts gained hit-rate on workload-a. This ran through frequency counters, the second-chance reference bit, and the multi-level 2-bit re-reference value (Cycles 0002–0004).
- Storage-layout changes that keep retention intact can save metadata. The per-bucket reference mask kept hit-rate, but at a time cost (Cycle 0004).
- Behavior-preserving rewrites of an existing mechanism can recover timing. A word-level scan over the per-bucket mask recovered put-side median time lost to the bit-by-bit clock walk (Cycle 0005).

## Recurring failures

- Shortcuts to the victim search keep failing. A rotating window, a cached minimum, a cached two-candidate pair, and a narrowed scan did not fix the put tail, and several cost hit-rate. The exact minimum search is doing real work (Cycles 0003, 0004, 0006, 0007).
- The put-side tail has not moved. Aging rewrites, early exits, and scan narrowing were each tried as its fix, and none helped (Cycles 0004–0008).
- Moving items within a bucket, through midpoint promotion or midpoint reinsertion, hurt hit-rate and added shifting work on the hot path (Cycle 0001).
- Ghost memory did not lift hit-rate, whether as a short bucket-local ring or a longer one (Cycles 0001, 0002).
- Stacking extra decay on the periodic halving erased frequency protection too quickly (Cycle 0003).
- Fingerprint prefilters added get and put cost with no hit-rate change (Cycle 0002).
- Doorkeeper admission gates are not a dependable lever. They gained hit-rate in one cycle, lost it in another, and barely moved it in a third, while adding put-side and metadata cost (Cycles 0006–0008).
- Hash placement changes did not move hit-rate. Bucket spread is not the miss driver, and any timing effect is unconfirmed (Cycles 0007, 0008).
- Moving the access counters into a separate array regressed get tail and put, without a retention gain (Cycle 0008).

## Open questions

- Where does the put tail actually come from on full-bucket puts? Shortcuts to both the scan and the aging have failed, so the cause is still unmeasured.
- Do the frequency and re-reference gains hold on traces other than workload-a and at other cache sizes?
- Are the timing movements real? Many get and put shifts across cycles are single-run, and some are unexplained by diffs. Repeated runs are needed before any timing Winner is kept.
- Is the get-tail gain from skipping the saturated-counter store reproducible with the counter kept in the item (Cycles 0006, 0008)?
- Can the exact global minimum be kept cheaply, for example by refreshing a candidate on counter updates?
- The metadata-size probe did not register new per-bucket or per-slot state (Cycles 0001, 0002). Its readings should not be used to judge storage cost until it is fixed.

## Per-policy notes

- Frequency-based victim selection (access counters with periodic halving): the main hit-rate source on workload-a. It is costly on put because each eviction scans the full bucket. Do not add extra decay or per-eviction halving (Cycles 0002, 0003).
- Second-chance reference bit: gained hit-rate and reduced put cost in one cycle, but its per-item metadata grew and the get path gained a write per hit. Attribution is not yet confirmed. Next steps are to cut state toward one bit and cut the get-side write (Cycle 0003).
- Multi-level 2-bit re-reference value: gained hit-rate, but its aging rounds carry the put-tail cost. The aging rewrite did not reduce that cost (Cycles 0004, 0005).
- Per-bucket reference mask: keeps hit-rate and saves metadata, but its get and put time cost has to be recovered. The word-level scan is the current put-side base (Cycles 0004, 0005).
- Rotating-window or narrowed victim scan: reduces put latency but weakens frequency protection. Dropped as a put-side fix (Cycles 0004, 0006).
- Doorkeeper admission gate: inconsistent across cycles. It needs reconciling before it is retried, and a looser second-sighting rule is the only variant still worth a test (Cycles 0006–0008).
- Bucket-local ghost memory and midpoint placement: no hit-rate gain. Treat as closed unless a different mechanism is proposed (Cycles 0001, 0002).
- Overlay code was unreadable in several cycles (0003–0008), so mechanism attributions across this ledger are inferred from diffs and variant insights and should be treated as provisional.
