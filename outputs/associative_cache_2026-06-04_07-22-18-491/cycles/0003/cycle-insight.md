# Cycle 0003 Insight

## Cohort outcome

All three variants tuned parameters of the LFU-plus-stash baseline: stash capacity, eviction policy within the stash, and the promotion credit awarded when a stash item re-enters primary. Two of the three regressed hit rate; only the promotion-credit increase won.

## Patterns

- The stash's small FIFO cap is a self-cleaning mechanism, not a constraint. Doubling the cap let cold items linger long enough to earn spurious promotions that displaced genuinely medium-frequency residents — larger capacity accumulated noise rather than useful survivors.
- Applying LFU selection inside the stash is redundant: every item in the stash arrived there because LFU already identified it as the lowest-frequency item in primary, so the frequency gradient within the stash is too compressed to discriminate. FIFO's age-based ordering is better matched to the stash's actual role.
- The winning change operated at a different structural boundary — the transition back into primary — rather than inside the stash itself. Raising the promotion credit from +1 to +2 cleared the tie that caused promoted items to be immediately re-evicted on the next cold burst, converting round-trips into genuine secondary hits.
- Both regressions and the win together suggest the stash mechanism is now well-calibrated in its internal design; the remaining slack is at the interface between stash and primary.

## Open questions

- Is +2 the right promotion credit, or does the gain plateau or reverse at higher values? The win was described as small, implying the lever has a range that has not been fully explored.
- Would dynamic promotion credit — scaled by how many times an item has cycled through the stash — outperform a fixed value?
- The `metadata-size` measurement gap persists after three cycles of stash variants; the stash's per-bucket memory overhead remains unquantified.

## Continuation

Next cycle should branch from v003 and explore the promotion-credit lever further: test +3 and/or a simple repeat-promotion multiplier to determine whether the gain is monotone or peaks near +2. Stash-cap and stash-eviction-policy changes are confirmed suboptimal on this workload and should be dropped. The `metadata-size` gap is now a three-cycle-old blind spot — if the evaluator can report it, a variant that deliberately trades memory for hit rate (or vice versa) would finally be verifiable.
