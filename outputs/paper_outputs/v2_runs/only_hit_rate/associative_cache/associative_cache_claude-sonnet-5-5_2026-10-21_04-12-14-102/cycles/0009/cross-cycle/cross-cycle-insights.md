## Recurring wins

- A structural probation/protected split (admission control separating one-touch from re-referenced entries) is the one big policy win; it beat plain LRU in every cycle that tested it.
- A lean positional split with no extra per-entry state (no ghost, counters, adaptive targets) captures nearly all the gain; it was rediscovered independently several times.
- Small probation (about slots/16, roughly 4 slots at baseline width) won; the trend turned when going smaller.
- Wider associativity stacks on the split, with diminishing increments (first doubling clear, later ones marginal).
- Speed-motivated, policy-preserving refactors "win" only as exact hit-rate ties; they are not real evidence.

## Recurring failures

- Weakening or delaying promotion (transposition, midpoint, half-promotion, two-hit rules) loses hit rate; first-hit promotion is hard to beat with short reuse distances.
- Ghost history and ghost-hit promotion add state without gain; a ghost hit is weak reuse evidence here.
- Small-window frequency victim choice and recency-stamp overrides of probation-first eviction were flat or negative.
- Insertion point tweaks (victim-end/LIP, mid-probation) cost hits: new keys need residency to earn a second reference.
- Multi-bucket placement (two-choice) fails the slot-legality invariant.
- Demotion placement and further probation shrinking are flat or negative; hit-rate knobs in this split are essentially exhausted.
- Many "wins" compare against the baseline rather than the incumbent, so they are not increments.

## Open questions

- Per-op-time and metadata-size have never been in the fitness vector (nine cycles), so no cost claim (wide scans, rotate refactor, skip-move, "lean" footprint) is verified. Fix this first.
- Does the marginal gain from 4x-wide buckets survive timing cost?
- Is slots/8 vs slots/16 probation on wide buckets actually better than the incumbent?
- Are further gains only possible from a different policy family (e.g. frequency-aware admission) or visible only on other traces?
- Are exact ties real neutrality or edits that never took effect (authors could not read overlay code)?

## Per-policy notes

- Plain move-to-back LRU: baseline; hard to beat via promotion tweaks alone.
- Segmented LRU (probation/protected, first-hit promotion, small probation, boundary-area insertion): the incumbent family; at or near its hit-rate ceiling on the LIRS-style trace.
- 2Q/ghost-style and adaptive variants: no benefit over the fixed lean split.
- Frequency-aware victim selection: no measurable effect in small windows.
- Wide-associativity segmented LRU: tiny, diminishing hit-rate gains; cost unpriced.
