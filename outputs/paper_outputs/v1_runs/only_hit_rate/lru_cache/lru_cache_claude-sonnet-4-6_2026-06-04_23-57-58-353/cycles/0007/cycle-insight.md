# Cycle 0007 Insight

## Cohort outcome

Both tested variants abandoned frequency counting entirely — one replacing the full policy with SIEVE's visited-bit second-chance mechanism, the other switching to inter-reference recency (IRR) timestamps within the existing SLRU structure. The IRR-based variant (v003) won with a substantial hit-rate lift, breaking out of the three-cycle neutral-winner plateau.

## Patterns

- Switching the eviction signal from accumulated frequency counts to recency-of-recency (IRR) is the sharpest gain yet recorded. The LIRS-generated trace rewards the cache for asking "how recently were this entry's last two accesses relative to each other" rather than "how many times has this entry been accessed in total." Frequency counting was the right direction early in the series but accumulated access counts conflate historical warmth with current warmth in ways IRR does not.
- Both survivors share a common insight: a single reuse since insertion provides a more useful eviction signal than total access count. SIEVE acts on "was it touched at all since insertion"; IRR acts on "how tightly clustered were the last two touches." IRR is the stricter, more informative form of the same intuition, which explains why it dominated.
- Preserving IRR timestamps across protected-to-probationary demotion replaces the halving-on-demotion mechanism naturally — a hot entry (small IRR) retains its competitive advantage in probationary without any manual counter adjustment, while cold newcomers arrive with IRR = MAX and lose the eviction contest without special handling.
- The ghost set was dropped in v003 and it won, confirming what cycles 4 through 6 suggested: ghost admission adds no value on this workload and can be omitted from the baseline going forward.
- The adaptive segment-ratio hypothesis (H01) was never evaluated due to an implementer failure. It remains untested on both the old frequency baseline and the new IRR baseline.

## Open questions

- Does IRR-based eviction transfer to non-LIRS workloads, or is it specifically aligned to this trace's generation structure? A single workload cannot distinguish broadly strong policy from a perfectly matched one.
- Is the 80/20 protected/probationary split still appropriate for IRR-based eviction, or does the sharper eviction signal change what the optimal ratio looks like? H01's adaptive balancing idea was never tested and is now more interesting on the new baseline.
- Are the current scan-window fractions (LRU quarter of protected, LRU half of probationary) still well-calibrated with IRR as the signal? IRR is a sharper discriminator than frequency counts, so narrower scan windows might suffice.

## Continuation

Next cycle should carry v003 as the new baseline: SLRU 80/20, IRR timestamps (preserved across demotion), no ghost set, LRU-quarter scan in protected, LRU-half scan in probationary. The highest-priority untested thread is adaptive segment-ratio balancing (H01 transposed onto the IRR baseline) — it was blocked this cycle and directly addresses whether the fixed split is optimal. Second priority is scan-fraction calibration: with a sharper eviction signal, narrower windows may yield equivalent quality with less scan overhead. Multi-workload evaluation is overdue and should be treated as a near-term requirement before further parameter tuning.
