## Recurring wins

- Retention signals on the bucket-local victim choice are the consistent hit-rate lever. Frequency counters (C2), a second-chance reference bit (C3), and a multi-level re-reference value (C4) each kept re-referenced keys through bursts of one-shot inserts.
- Behavior-preserving rewrites of an existing mechanism (per-bucket mask, word-level scan) keep hit-rate and metadata flat and are the only changes that have recovered timing lost to earlier variants (C4 to C5).
- The second-chance scheme (C3) is the only result that improved hit-rate and put cost together. It remains the best-supported base.

## Recurring failures

- Shortcuts to the victim search have not paid off: rotating window (C4), unreferenced-only scan (C6), cached minimum (C3), cached two-candidate pair (C7). Each lost hit-rate or failed to shorten the put tail.
- Admission gates are not a dependable lever. The doorkeeper gained (C6), lost (C7), barely moved (C8), and the T=1 gate had no effect (C9).
- Moving items within a bucket (midpoint promotion, midpoint reinsertion) hurt hit-rate and added hot-path shifting (C1).
- Bucket-local ghost memory has not helped. Lengthening the ghost ring did not lift hit-rate (C2), and the reinsertion idea looks weak on this trace.
- Decay changes are not the put-tail fix. Extra miss-driven decay erased frequency protection (C3), and lazy per-bucket halving moved the cost into ordinary ops and worsened both tails (C9).
- Fingerprint prefilters and aging-round rewrites add scan and write work on the hot path without hit-rate benefit (C2, C5).
- Hash placement is at most a small timing lever with no hit-rate gain (C7, C8).
- A separate counter array outside the item regressed get and put timing (C8).

## Open questions

- The put-tail source on full-bucket puts is still unidentified after many changes to scan, aging, and decay. Direct profiling of the victim path is needed before another variant is designed around it.
- All retention evidence comes from workload-a. Does the gain hold on other traces and cache sizes?
- The doorkeeper results conflict (C6 vs C7). Is the parent or the exact reject rule responsible?
- Is the get-tail gain from skipping the saturated-counter store real and reproducible with the counter kept in the item (C6, C8, C9)? Its get median has not moved in its favor.
- Per-item versus per-bucket reference state is unresolved: one bit per item (C3), a per-bucket mask (C4), or multi-level per slot (C4). Which costs less for its hit-rate is unknown.
- Several Winner labels (C7, C8, C9) rest on single-run timing or have no helped dimensions. Their status should be re-measured.
- The metadata-size probe has not registered new per-bucket or per-slot state (C1, C2). It cannot yet be used to judge storage cost.
- Mechanism attributions across cycles are inferred from diffs because overlays were unreadable. Confirm before relying on them.

## Per-policy notes

- Frequency-based victim selection (per-slot counters, periodic halving): produced the first hit-rate gain (C2). Keep the exact global-minimum search and the periodic halving schedule. Put-side shortcuts and extra decay have failed. The saturated-counter store skip is the get-side candidate to re-measure.
- Second-chance and re-reference retention (per-item bit, per-bucket mask, 2-bit per-slot value): the best-supported line (C3, C4). The mask saves metadata at a time cost that the word-level scan recovered (C5). The multi-level value gains hit-rate but raises the put tail. Open: whether one bit suffices.
- Bucket-local recency, midpoint movement, and ghost memory: no hit-rate benefit; drop as axes (C1, C2).
- Admission gating (doorkeeper, count-threshold gate): inconsistent results; not a lever until reconciled (C6 to C9).
- Decay scheduling (global or lazy per-bucket halving): keep the periodic global schedule. Lazy per-bucket halving worsened tails (C9).
- Placement and hashing (bucket index, Fibonacci hash): timing-only at best; not a hit-rate lever (C7, C8).
