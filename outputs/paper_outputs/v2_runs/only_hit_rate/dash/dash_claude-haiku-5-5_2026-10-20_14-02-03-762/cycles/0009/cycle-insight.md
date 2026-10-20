# Cycle 0009 Insight

## Cohort outcome

The cohort tried three changes: collapsing the saturating reuse counter into the one-bit seen-in-stash marker (v001), adding an equivalence-preserving occupancy check that skips slot scans which cannot find a free slot (v002), and moving 128 slots from the stash back to the probing tier at fixed total capacity (v003). Only v003 is a Winner, on a small hit-rate gain over its parent on the single LIRS trace at size 2048.

## Patterns

- The capacity move is the only change with a hit-rate signal. Reversing the cycle 0008 shrink of the probing tier improved hit-rate, which is the opposite of what that shrink produced. Together the two results point to probing residency, not stash capacity, as the binding resource on this trace.
- The reuse-counter direction is again weak. Removing the counter's extra states (v001) produced a small drop just outside tolerance, so the counter may carry minor signal, but the effect is small and on one trace.
- The occupancy check (v002) matched its parent exactly, as a victim-preserving change should. Its cost benefit is unmeasured because per-op-time is absent from the fitness vector.
- No variant recorded metadata-size or per-op-time, so the cost side of the cohort is still unknown, including the bit cost of the reuse counter.
- None of the three insights confirms the overlay source. The v003 reading is tied to the stated change, not to verified constants, so the winning implementation is not yet checked.

## Open questions

- Is the v003 gain larger than trace-level noise, and does it hold on a second workload or capacity?
- Where is the probing-tier optimum? The gain from growing probing may flatten or reverse beyond 128 slots.
- Does the small v001 drop indicate a real loss of reuse-depth information, and is it worth the metadata it costs?
- Do the occupancy skip and the probing resize stay consistent with the stash-full retry and derived sizes?

## Continuation

Next cycle should pursue the probing-tier capacity increase from v003 as the lead, and test a few sizes around it on a second workload while recording hit-rate, metadata-size, and per-op-time. Drop the seen-in-stash collapse from v001, since it did not help and the reuse-counter lineage has shown no robust signal. Keep the occupancy check only as a cost-side check that records per-op-time, and verify the overlay diffs before treating v003 as settled.
