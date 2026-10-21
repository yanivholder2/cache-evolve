# Cycle 0003 Insight

## Cohort outcome

All three variants were built on cycle-0002 v003 (MRU stash re-entry) and each tested a different lever. v001 stacked admission by raising the stash threshold to 3. v002 changed stash victim choice to a CLOCK-like scan that prefers counter-0 slots. v003 changed cascade routing so that evicted foreign residents go straight to the stash. None of them moved hit-rate beyond noise. v002 is the nominal winner, but only by a noise-level margin.

## Patterns

- The design looks saturated on this trace at this size. Once the stash works as an MRU-placed victim cache with a pre-credit, admission, victim choice and routing all end up choosing between victims that can be recovered cheaply. Changing which one is chosen barely matters.
- The pre-credited ex-residents seem to dominate stash traffic. That explains all three null results. A higher threshold has no fresh keys to filter (v001). The scan skips the same items that MRU placement already protects (v002). Rerouting the cascade swaps one victim the stash can bring back for another (v003).
- Position-based protection and frequency-based protection in the stash overlap almost completely. Stacking two retention mechanisms that protect the same items adds nothing.
- The neighbour cascade damage flagged in cycle 0001 no longer matters for hit-rate now that the stash recovers demoted items cheaply.
- v001's result matched its parent exactly. That could mean a real no-effect, but it could also mean the change never reached the evaluated build. None of the variant writers could inspect the overlay source.

## Open questions

- Did v001's threshold change actually reach the evaluated build?
- Does v003's cascade-free routing lower per-op-time, as its rationale predicted? It cost no hit-rate, so it may win on that axis instead.
- Is the plateau a property of this trace and cache size, or of the design? Every result so far comes from one LIRS trace at one size.
- Are stash capacity, or the split of slots between the stash and the normal tier, the remaining hit-rate levers?

## Continuation

Stop running hit-rate-only variants on this single trace: three different levers came out flat, so the design looks saturated there. Next cycle should put per-op-time and metadata-size measurement first (v003's cascade-free routing is the natural candidate for a per-op-time win), check that v001's overlay was applied, and evaluate on at least one more trace or cache size. If hit-rate work continues, try structural capacity changes such as the stash-to-normal slot ratio. Drop further tweaks to stash victim choice and admission thresholds.
