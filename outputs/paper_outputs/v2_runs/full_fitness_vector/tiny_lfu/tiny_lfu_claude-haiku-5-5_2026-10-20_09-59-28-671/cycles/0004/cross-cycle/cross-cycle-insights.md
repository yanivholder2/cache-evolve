# Cross-cycle insights

## Recurring wins

- Constant-factor work on the admission path has been the most reliable source of gains: relink guards, hoisted hashing, and cached sketch seeds all cut per-op cost.
- Changes that keep eviction order and counter indices intact hold hit rate flat while reducing cost.
- Metadata reduction can be decoupled from policy: packing state into existing fields shrank bookkeeping without moving hit rate.

## Recurring failures

- Changes that alter admission or aging have not raised hit rate: recency rotation of rejected victims, a one-hit doorkeeper, shorter sketch aging, and a small recency window all left it flat or lower. The frequency gate appears to already do the protective work.
- Capping the protected segment cost hit rate. The segmentation gain seems to depend on the protected set turning over.
- Switching index schemes trades fidelity against speed: matching the public per-row indexing restored hit rate but gave back most of the speedup.
- Bookkeeping keeps growing per node, per instance, or in list work, often without being reflected in reported metadata.
- Timing attributions are routinely unverified: speedups and regressions were assigned to specific helpers or accessors without source inspection.

## Open questions

- Does the segmentation hit-rate gain hold on any trace other than the one used so far?
- Do the cached and hoisted seed/hash helpers produce exactly the same counter indices as the public path for every key? Parity so far is consistent with yes but not proven.
- Can the seed table be placed in a `const` item so the speedup survives without per-instance growth?
- Are the unexplained timing shifts (put-side changes, latency drops in segmentation) real under repeat runs?
- Is the doorkeeper's storage fully accounted for in reported metadata, and does the doorkeeper conclusion change on a trace where one-hit keys dominate?

## Per-policy notes

- TinyLFU frequency admission gate: the strongest lever so far is per-op cost on the sketch path. Hit rate has resisted changes to the gate itself, including aging period, one-hit filtering, and recency windows.
- Sketch indexing and seeds: index-scheme changes are the main fidelity-versus-speed trade-off. Cached seeds look hit-rate neutral and should be verified against the public indexing before further reuse.
- Segmented (probation/protected) main region: the only source of hit-rate movement observed, but small, and it adds per-node and per-cache metadata. Capping it hurt. Needs a second trace before it is built on.
- Window and recency rotation: no hit-rate benefit observed; both added list work. Drop as directions on the current workload.
- Evidence base: every result across all cycles comes from one trace (workload-a), so no effect has been shown to generalize.
