## Recurring wins

- Protecting hot residents helps hit rate modestly. Biasing eviction toward frequently used entries (cycle 2) and keeping single-touch stash entries out of target buckets (cycle 3) both reported small hit-rate gains, and both fit the concern that new entries displace hot target residents.

## Recurring failures

- Insert-position and admission gating has repeatedly cost hit rate. MRU promotion traded hit rate for latency (cycle 1), gating insert position on bucket fullness lost hit rate (cycle 2), and blocking first-touch admission lost hit rate (cycle 3). Delaying or redirecting entries on the way in tends to lose reuse.
- Stash-path bookkeeping does not pay on the measured workload. An occupancy fast path raised get and put latency (cycle 2), which suggests stash buckets are rarely empty.
- Latency claims are unreliable. Per-op and put-side gains or regressions have appeared without a matching mechanism in cycles 1, 2 and 3, and get-tail regressions have accompanied claimed latency wins.
- Winner labels do not track the measured dimensions. Variants with hit-rate losses or per-op regressions were still labelled Winners in cycles 1, 2 and 3, and the basis is not stated.
- Tie-breaking among equal eviction counters by recency did not move hit rate and added eviction cost (cycle 3).

## Open questions

- Does the stash-to-target promotion deferral's hit-rate gain hold on a stash-dominant trace, and does its get-tail regression go away there?
- Does frequency-aware victim selection hold on workloads other than workload-a before it is built on?
- Does admission gating help on scan-heavy traces, where it was meant to help?
- Is the get-path reordering's latency gain real on stash hits and across reruns, or is latency movement in general measurement noise?
- Does insert-position loss on hit rate depend on how full target buckets are, and is it stable across traces?
- Cycle 3 credits a counter-ranking effect to a Variant whose insight describes an admission gate. Is this a stale reference or a mislabelled Variant?
- Every signal so far comes from a single workload. No directional claim should be generalised until a second workload is measured.

## Per-policy notes

- Insert and promotion policy: MRU insert protects new keys but can displace hot residents, and fullness-gated front insert lost hit rate. Promotion from stash to target is the one variant in this family that gained hit rate, with a tail cost. Pursue only with a guard against displacing hot target residents.
- Admission policy: first-touch delay lost hit rate on workload-a. Keep only if it is aimed at scan-heavy traces and checked there.
- Eviction and victim policy: a small saturating frequency counter in victim choice is the only change in this family with a clear hit-rate gain. Recency tie-breaking among equal counters had no effect on hit rate. Prefer frequency-aware victim selection for the next cycle.
- Get path and stash handling: reordering tier lookups is the only change whose stated mechanism is latency, but the gain is unconfirmed. Stash occupancy fast paths did not pay. Re-measure the reordering on stash hits before building on it.
