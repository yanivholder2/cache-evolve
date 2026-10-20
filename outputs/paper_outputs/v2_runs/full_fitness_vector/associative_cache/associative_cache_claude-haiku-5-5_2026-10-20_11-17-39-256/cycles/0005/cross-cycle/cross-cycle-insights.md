## Recurring wins

- Retention signals that separate re-referenced items from one-shot inserts keep helping hit-rate on workload-a. Frequency counters (C2), the second-chance bit (C3), and the 2-bit re-reference value (C4) all fit this pattern.
- Moving reference state into a per-bucket mask (C4) kept hit-rate. It is a viable metadata option, but it costs time.
- Behavior-preserving rewrites of the scan form can recover time without touching hit-rate. The word-level scan (C5) recovered put median that the bit-by-bit clock walk had lost (C4).

## Recurring failures

- Bucket-local ghost memory and reinsertion have not paid off. Midpoint promotion and midpoint reinsertion (C1) lost hit-rate. Lengthening the ghost ring (C2) did not help, so the reinsertion idea itself looks weak here.
- Shortcuts to the victim search have not removed the put-side cost. A cached minimum (C3) was invalidated by counter updates on get hits. An early exit on zero-counter slots (C5) gave only a small gain. Narrowing the scan to a rotating window (C4) lost hit-rate.
- Extra decay on top of the periodic halving (C3) erased frequency protection too fast. Per-eviction halving merged into the victim scan (C3) did not help.
- A fingerprint prefilter on 32-slot buckets (C2) added a second scan on get and write work on put, with no hit-rate gain.
- Rewriting aging as scan-then-bulk-add (C5) made the put path slower. The aging rounds are not the dominant put-tail cost, which contradicts the C4 diagnosis.
- Wider buckets (C1) gave a small hit-rate gain but a longer get-path scan.

## Open questions

- Where does the put tail come from on full-bucket puts, if neither aging nor the zero-counter scan dominates it? A profile of the victim search is needed before changing it again.
- How often is a zero-counter slot present at put time? Measuring this decides whether the early exit is worth keeping.
- Do the retention gains (frequency, second-chance, 2-bit re-reference) hold on traces other than workload-a and at other cache sizes? The wider-bucket gain needs the same check.
- Does narrowing the victim search lose hit-rate on other traces, or only on workload-a's one-shot pattern?
- Metadata-size has not moved in any variant that adds per-bucket or per-slot state (C1, C2). The size probe probably does not count the new state, so it should not be used to judge storage cost until fixed.
- Put and get latency shifts have been single-run and sometimes unexplained by the diffs (C1, C2, C5). Re-measure before relying on any timing gain, including the get-side movement in C5.
- Reference-state direction is unresolved: per-bucket mask and fewer bits (C4 v001) versus more bits per slot (C4 v003). Which direction is worth its cost is not yet shown.
- Mechanisms for C3–C5 are inferred from diffs and variant insights, not confirmed from overlay code. Confirm them before building on the attribution.

## Per-policy notes

- Frequency-based victim selection: the first retention win on workload-a. The full-bucket victim scan on put carries the hit-rate gain and is also the main put cost. Cached-minimum, early-exit, and merged-halving shortcuts have not reduced that cost. Rotating-window narrowing trades hit-rate for put latency.
- Second-chance reference bit: hit-rate gain with lower put cost. Per-item metadata grew beyond one bit and the get path writes on every hit, so the next step is to cut per-item state toward one bit and limit the get-side write. Mechanism is provisional until the overlay is checked.
- Per-bucket reference mask: same hit-rate as parent. Get and put both slowed under the bit-by-bit clock walk. The word-level scan recovered put median but not put tail. Get-side changes are unconfirmed.
- Multi-level 2-bit re-reference value: hit-rate gain with a put-tail cost from the aging rounds. The bulk-add aging rewrite made the tail worse, so the tail cost needs a different diagnosis, such as capping rounds or tracking aging state.
- Periodic halving schedule: keep as is. Extra or per-eviction decay has hurt.
- Bucket-local ghost ring and midpoint placement (promotion, reinsertion): no wins across C1 and C2. Do not tune the ghost-ring length further.
- Bucket geometry (wider buckets) and fingerprint prefilter: wider buckets gave a small hit-rate gain with a get-path cost. The prefilter gave no hit-rate gain and added scan and write cost. Neither is a current direction.
