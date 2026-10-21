# Cycle 0007 Insight

## Cohort outcome

All three variants branched from cycle 0006's v002 (fingerprint-ring ghost). Two were cost-side changes that left the policy unchanged: v001 added an identity hasher on the ghost map, and v003 narrowed fingerprints to 32 bits. One changed the policy: v002 replaced main's FIFO reinsertion with a SIEVE-style hand that keeps survivors in place. v002 won with a hit-rate gain beyond its stated noise band. v001 matched its parent exactly and v003 stayed within noise, both as predicted.

## Patterns

- Main-queue eviction *order* was the one lever left untouched, and it is the first real hit-rate gain in several cycles. The "saturated" verdict from cycles 0005–0006 applied only to the partition, credit, graduation and re-admission knobs, not to the policy family. The likely mechanism ties back to the re-admission theme: freq-0 ghost re-entries stop being pushed toward the tail by reinserted survivors, so they get a full sweep to prove reuse. That mechanism is inferred and was not logged.
- SIEVE changes what the retention knobs mean. A credit is now an extra hand lap, not a trip back to the head. So the cap and decay results from cycles 0004–0005 were measured under FIFO reinsertion and may not carry over.
- Both cost-side variants were confirmed safe for the policy (the identity hasher is bit-for-bit identical, and 32-bit fingerprint collisions are negligible at this size). They are also independent and compose cleanly. Neither can be preferred yet, because per-op-time and metadata-size are still unmeasured, now for seven cycles. The Winner adds its own unmeasured costs: per-entry link fields, and sweep steps that chase pointers instead of popping contiguous deque entries.

## Open questions

- Under SIEVE, do lower caps, or clearing credit on pass instead of decrementing it, change the result? Cap and decay were settled only under FIFO reinsertion.
- Does the SIEVE gain hold on other traces and cache sizes, or is it specific to workload-a's reuse pattern?
- Does pointer-chasing in the sweep cancel out the per-op-time savings from the identity hasher and the narrow fingerprints?

## Continuation

Use v002 (SIEVE hand in main, all else as cycle 0006) as the baseline. Fold in the identity hasher and 32-bit fingerprints together as a single cost-only change, since both are proven policy-neutral. Spend the policy budget re-tuning retention under SIEVE (cap value, decrement vs clear-on-pass). Do not reopen partition, graduation, re-admission or ghost-capacity levers. Measuring per-op-time, metadata-size and a second trace or size is still the prerequisite for judging every cost-side change.
