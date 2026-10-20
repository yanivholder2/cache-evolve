# Cycle 0007 Insight

## Cohort outcome

The cohort tried three metadata and bookkeeping edits: moving the frequency counter out of the queue entry into the index record, replacing eager ghost removal with a push-count window and batched pruning, and replacing the per-queue position scans with a single id-keyed index. All three were labeled Winners, but the evidence is uneven: the ghost window is the only edit with a hit-rate movement, and the index and counter edits show no hit-rate change.

## Patterns

- The ghost window is the first edit in this series to address the drain cost directly. Expiry is batched and runs off the per-eviction path, which answers the open question from Cycle 0006 about amortizing the ghost drain. Its hit-rate gain is small and rests on a single trace.
- Index-based lookup is now a repeated direction. Cycles 0005 and 0006 moved toward id-keyed maps, and this cohort's index variant does the same again. Its get and put speedups have a plausible code-level rationale, but they are unverified and still need a baseline rerun before they are credited.
- The frequency-counter relocation changed `Entry` but not hit rate. Its container-size effect is not clearly reported, so the claim that the change is free in footprint is not yet established.
- Container-level footprint remains the unit that matters. Each Winner changed the container, and none changed `Entry` in a way that could be measured as a net saving.
- The metadata edits and the index edit overlap in the state they touch. The index variant and the counter relocation may both reshape the same record, and no variant was tested in combination with another, so their gains may not add.
- Get-path timing moved in at least one variant whose get logic was unchanged, which matches the shared-noise pattern from earlier cycles.

## Open questions

- Does the ghost window's hit-rate gain hold on other traces and capacities, and is the window size close to the strict cap the way the variant claims?
- Does the frequency counter's container-size cost stay at zero under a whole-structure measurement, or does the index record's extra byte show up at larger sizes?
- How much of the index variant's put-side gain is due to the index itself, as opposed to the inherited ghost behavior? An ablation with the index removed would separate them.
- Do the counter relocation and the index change compose, or do they conflict on the same record?

## Continuation

Next cycle should validate the ghost window on additional traces and capacities, since it is the only edit with a hit-rate movement and it addresses the drain cost raised in Cycle 0006. Treat the index lookup as established direction: validate its speedup with a get-path baseline rerun rather than crediting it again as new. Test the frequency-counter relocation only together with a whole-structure footprint measurement, and do not count it as a Winner on the `Entry` size change alone.
