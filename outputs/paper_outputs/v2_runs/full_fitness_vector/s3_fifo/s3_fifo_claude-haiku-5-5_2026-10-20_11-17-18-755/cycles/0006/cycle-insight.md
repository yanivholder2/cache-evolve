# Cycle 0006 Insight

## Cohort outcome

The cohort tried three edits: a half-main cap on the fingerprint ghost, merging the two per-queue indexes into one id-keyed map, and raising the small-queue share on its own. The index merge is the cleanest Winner, with a container-size and put-cost benefit and no hit-rate change. The ghost cap is the only edit with a hit-rate movement, and it is paid for on the put path. The share change has weak support.

## Patterns

- The ghost cap combined with fingerprint storage gave a small hit-rate gain, but its per-eviction drain adds put-side cost. This erodes the put-latency benefit the fingerprint ghost showed in earlier cycles, so the two effects need to be separated.
- The index merge reduced container state and put cost while leaving eviction, ghost, and promotion logic untouched. Hit rate was unchanged. Its only visible cost is a small decode on the get hit path, which is within noise.
- The small-queue share change alone left hit rate unchanged on the one workload reported. This is consistent with Cycle 0003: the earlier hit-rate gain came from the victim-choice change bundled with the larger share, not from the share itself. Its latency movement is modest and comes from a single run.
- Get-path timing moved in variants whose get code was not changed, matching the shared-noise pattern from Cycles 0001 through 0005.
- Container-level footprint is again the unit that matters. Each edit changed the container, not `Entry`.

## Open questions

- Can the ghost-cap drain be amortized, for example by compacting in batches, so the hit-rate gain survives without the put-side cost?
- Does the index merge's put gain reproduce under a baseline rerun, and does the tagged-slot decode matter at larger sizes?
- Is the small-queue share change's latency shift real, or is it single-run noise?

## Continuation

Next cycle should pursue the index merge as the strongest candidate: it has no hit-rate cost and a structural benefit, so validate it on more traces before treating it as settled. Keep the half-main ghost cap only if its drain can be made amortized; otherwise drop it. Do not credit the standalone small-queue share change as a Winner without repeated runs. Re-run a get-path baseline before crediting any get-latency change.
