# Variant v003 Insight

## Helped

(No dimension moved in a favorable direction beyond rounding noise.)

## Hurt

- per-op-time: The put-miss path now compares the minimum-counter victim against threshold T=1 and returns without inserting when the victim is above it, so put picks up a post-selection branch and a refused-insert path; the get path is specified as unchanged, so the get latency movement is not explained by this change and is more likely measurement noise or layout effects.

## Takeaway

Gating admission on the victim's access counter (T=1) did not add hit-rate on workload-a beyond the exact min-counter victim selection already in the parent: the refusal rule only fires when every slot in a full bucket has a counter of at least 2, so it protects a narrow set of buckets and the trace shows no measurable gain. The follow-up should check how often the refusal branch actually fires on workload-a before adding further admission logic, since a near-zero effect suggests the frequency signal is already captured by the victim choice. Note: the overlay source could not be read in this session, so this interpretation is drawn from the hypothesis text and the fitness vectors rather than the diff itself.
