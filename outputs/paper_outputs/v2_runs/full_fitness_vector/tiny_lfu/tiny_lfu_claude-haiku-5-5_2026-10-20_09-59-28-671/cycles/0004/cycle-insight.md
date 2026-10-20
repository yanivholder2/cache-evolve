# Cycle 0004 Insight

## Cohort outcome

The cohort tried three changes: caching per-row sketch seeds at construction instead of re-deriving them (v001), inlining the protected-flag accessors on the get hit path (v002), and a small W-TinyLFU-style window in front of the frequency gate (v003). The Winners are v001 and v002, both of which kept hit rate at their parent's value; v003 did not move hit rate and was not a Winner.

## Patterns

- Per-op cost is the main lever again. Seed caching recovered constant-factor time lost to the Cycle 0003 per-row indexing change, with no hit-rate change, which fits the claim that counter indices are unchanged.
- Caching the seeds added fixed per-instance metadata. The struct grew, which the hypothesis had tried to prevent. Storing the same values in a `const` table would likely keep the speed without the growth.
- Accessor inlining on the get path helped gets but not puts. The put-side regression was not predicted and may be run-to-run noise, so the get gain should not be read as a general win yet.
- A recency window did not help. Hit rate did not move, and the added list work on both get and put made latency worse. This matches earlier cycles, where changes that touched admission or aging did not raise hit rate on this workload.
- Several attributions remain unverified. The overlay source was not inspected in any of the three variants, so the links between changes and timing shifts are inferred, and all evidence comes from one trace (workload-a).

## Open questions

- Do the cached seeds produce exactly the same counter indices as the per-access derivation for every key? Hit-rate parity is consistent with yes but does not prove it.
- Can the seed table live in a `const` item so the speedup holds without struct growth?
- Does the put-side latency change from inlining survive repeat runs, and does the get gain hold on a second trace?
- Does the window's bookkeeping appear in the reported metadata at all? The reported node size did not change, so that figure may undercount it.

## Continuation

Pursue seed caching from v001, but move the seeds into a `const` table and verify index equivalence against the public path before building on it. Keep the get-path accessor inlining from v002 as a candidate, and re-run its put timing before accepting it. Drop the W-TinyLFU admission window (v003) as a direction on this workload.
