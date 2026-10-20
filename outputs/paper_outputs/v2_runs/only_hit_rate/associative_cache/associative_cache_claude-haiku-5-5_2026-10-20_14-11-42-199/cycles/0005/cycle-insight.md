# Cycle 0005 Insight

## Cohort outcome

The cohort tried a shared-frequency admission gate on top of probationary-first eviction (v001, the Winner), a removal of demote-on-promotion in the protected half-bucket (v002), and inserting misses at the eviction-next position instead of MRU (v003). The two variants that address one-shot inserts both reported a hit-rate gain on workload-a; the protected-tier variant ended slightly below its parent.

## Patterns

- Both positive results target the same problem: one-shot inserts displacing entries with proven reuse. The admission gate (v001) filters them at miss time, and the eviction-next insert position (v003) lets them leave first. Whether these gains overlap or add is not yet known.
- The Winner's gain is recorded only qualitatively. No magnitude is given, so v001 and v003 cannot be ranked against each other on the evidence in their insights.
- Removing the forced demotion (v002) did not recover the loss from the protected tier in Cycle 0004. The protected cap itself, or the second-hit signal, is the more likely cause, consistent with Cycle 0004's suspicion.
- Hit-rate is still the only measured dimension in all three insights. Metadata size and per-op time remain unknown for the shared counter (v001) and the miss-position flag (v003), even though v003's hypothesis claims no new per-entry metadata.
- v002's insight says its overlay could not be read in this run, so its result is not confirmed against the actual diff. The other two insights do not state this.

## Open questions

- Are the admission gate (v001) and the eviction-next insert position (v003) redundant, or do they stack? Testing each alone against the parent would settle this.
- Does the v003 gain hold beyond workload-a, given the Cycle 0004 concern about single-trace results?
- What does the shared counter's metadata and per-op cost amount to once measured?
- Is the protected-cap failure in v002 a cap-size problem or a second-hit-signal problem?

## Continuation

Next cycle should pursue one-shot-insert filtering, testing the admission gate (v001) and the eviction-next insert position (v003) separately or in combination, and confirm the gain on a second workload. Include metadata size and per-op time in the fitness vector before accepting either change. Drop the protected half-bucket with demote-skip as a hit-rate lever, and do not retry it without a changed cap or second-hit signal.
