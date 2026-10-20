## Recurring wins

- Landing position beats movement: placing a re-referenced stash item at the MRU end of its target bucket is the one change that has helped, and it still anchors the current best line.
- Admission filtering on stash promotion helps: requiring a second hit before promotion reduces one-off stash hits displacing resident items. Its gain is small and so far appears on top of MRU landing rather than alone.

## Recurring failures

- Moving proven-hot items out of the probing tier hurts. Removing and re-inserting probing hits pushes them back through target competition and loses hit-rate.
- Shortening the demotion path without compensation does not pay. Skipping the target retry for foreign evictions lowered hit-rate, and gating it on stash capacity recovered only a small part of the loss, which suggests the removed retry mattered under stash pressure.
- Refinements to the landing slot that depend on target-bucket occupancy show no measurable effect, and there is no evidence the full-bucket case they target occurs on the tested workload.
- Using the "seen in stash" marker as an eviction signal hurts. Preferring never-re-hit stash entries as victims discards recently arrived items that have not yet had a chance to prove reuse.
- Marker relocation into a spare tag or bitmap bit produced no hit-rate change, and it is unclear whether the layout actually changed. Treat it as unverified, not as a neutral result.

## Open questions

- Do the MRU landing and the second-hit gate hold on workloads and capacities other than the single LIRS trace used so far?
- Per-op-time and metadata-size have not been measured in any variant. Does the promotion filter or its marker carry a real cost?
- Is v001 of the latest cycle a win against the baseline, or only against its parent? Its reported result matches the parent.
- Would the removed target retry pay for itself if the shorter eviction path were paired with better stash capacity handling?
- Is the victim-preference idea dead in general, or does it fail only when the marker is the sole eviction criterion?
- Do several insights rest on hypothesis text rather than verified diffs, so the implementations as described are not confirmed?

## Per-policy notes

- Stash tier (promotion and landing): MRU landing for stash hits is the most consistent positive signal. Pair it with a second-hit promotion gate and test that pairing across more workloads and capacities.
- Stash tier (eviction): marker-based victim preference hurts; avoid it unless combined with other criteria that protect recent arrivals.
- Probing tier: keep resident probing hits in place; do not remove and re-insert them.
- Normal/target tier (foreign evictions): skipping the target retry is a regression on its own. Revisit only together with stash capacity handling, and only with per-op-time measured.
- Cost dimensions (all tiers): per-op-time and metadata-size are unmeasured across all cycles, so no policy has a known cost profile yet.
