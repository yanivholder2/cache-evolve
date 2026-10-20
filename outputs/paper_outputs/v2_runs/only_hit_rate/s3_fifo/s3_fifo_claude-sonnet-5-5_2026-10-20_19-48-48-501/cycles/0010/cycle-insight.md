# Cycle 0010 Insight

## Cohort outcome

Three single-constant probes were tried on separate parents: small queue to ~5% (v001), small queue to ~2% (v002), and a ghost 1.5x longer (v003). v002 is the Winner with a slight gain. v001 tied its parent exactly. v003 lost a very small amount.

## Patterns

- The small-queue sweep is flat: 2% gave a slight gain, 5% was indistinguishable from its parent. Sizing sensitivity has largely run out within the range of ~2-5%.
- Lengthening the ghost slightly hurt, which fits earlier notes that the shortened ghost is near its optimum. Ghost length is a low-sensitivity, tuned constant.
- The sweep was not head-to-head. The three probes used different parents (0008/v002, 0009/v001, 0008/v002), so 2%, 3% and 5% are still not compared on one parent.
- The v001 exact tie may mean the parent already rounds to the same effective queue size, or the diff does not change behaviour. This repeats the earlier suspicion about no-op diffs.
- Variant authors still could not read overlay source. Per-op-time and metadata-size are still unmeasured, and all evidence is one trace at one cache size.

## Open questions

- Is v002's gain real or noise? It is very small and has no replication.
- Do the effective queue lengths for 2%, 3% and 5% actually differ at this capacity?
- Does the near-flat sizing curve hold on a second trace or cache size?

## Continuation

Treat small-queue ratio and ghost length as saturated and stop tuning them; keep the ~2% configuration (v002) as the base. Next cycle should move to a structurally different lever, or validate the current best on a second trace and cache size. Any further sizing probe must share one parent and confirm the effective queue lengths differ. Restore per-op-time and metadata-size measurements.
