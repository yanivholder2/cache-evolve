## Recurring wins

- Two-segment SLRU segmentation has been the strongest single structural gain across all cycles; partitioning entries into probationary and protected consistently outperforms any single-pool eviction tweak.
- Acting at insertion time (SLRU routing) beats acting at eviction time alone; mechanisms that wait until eviction to differentiate reuse quality are weaker because the signal arrives too late to protect the working set.
- Persistent frequency counters with counter-halving on segment transitions have proven load-bearing: they let accumulated warmth survive demotion so a node's history remains readable at the next eviction decision. This signal collapses when reset to zero on demotion.
- Applying the frequency-aware eviction rule symmetrically across both segments (protected and probationary) consistently improves hit rate over applying it to only one; asymmetric policy leaves the signal stranded at the wrong boundary.

## Recurring failures

- Ghost admission with a window bounded to probationary capacity has been tested twice (once bundled with SLRU, once isolated) and shown negligible independent effect on this workload; the reuse events it targets are not dense enough in the trace to justify the bookkeeping.
- Expanding the ghost window to full cache capacity regressed performance: keys whose reuse distance spans the full capacity do not carry reliable locality signal on this trace, and fast-pathing them into protected pollutes the working set.
- Frequency-aware demotion fails when the scan window is a fixed small constant rather than a fraction of the segment; a constant window creates no meaningful selection pressure as segment sizes vary.
- Suppressing position updates (move-to-tail) on protected hits — whether via a second-chance reprieve bit or any other mechanism — degrades the recency ordering the LRU eviction logic depends on; the hot entries the mechanism tries to protect end up stranded near the demotion head regardless.
- Widening the protected-segment scan fraction beyond one quarter produces diminishing returns on this workload; the frequency variation near the LRU end is already captured by the narrower window.

## Open questions

- Is the probationary LRU-half scan fraction near optimal, or should it be adaptive? Workloads with longer or more skewed reuse-distance distributions may need a different fraction at the probationary eviction site.
- Could an intermediate ghost window (e.g., 50% of total capacity) recover meaningful cross-residency signal without the admission-quality regression seen at full capacity?
- With frequency awareness now applied at both segment eviction sites, what structural gaps remain? A wider workload suite would clarify how general the accumulated gains are.
- Could a second-chance reprieve concept be salvaged by setting the referenced bit while still performing move-to-tail? The cycle-3 failure was caused by the position-freeze trade-off, not the reprieve concept itself.
- Would either or both scan fractions benefit from being proportional to observed reuse-distance patterns (adaptive) rather than fixed?

## Per-policy notes

- **SLRU (two-segment):** Consistently the strongest baseline. The 80/20 protected-to-probationary split has proven robust; widening probationary to 30% hurt by shrinking the protected working set. All winning variants since cycle 1 have built on this foundation.
- **Frequency-aware demotion (protected segment):** Failed in cycle 2 due to a constant small scan window and counter reset on demotion. Repaired in cycle 3 with a fractional window and counter-halving; has been a net win since. Scan widening beyond 1/4 in protected offers no further gain on this workload.
- **Frequency-aware eviction (probationary segment):** Introduced in cycle 4 as the symmetric complement to protected demotion. Scanning the LRU half of probationary and applying the frequency counter there closed the remaining asymmetry gap and produced the cycle-4 win.
- **Ghost admission:** Shown negligible in isolation (cycle 3) and regressive when the window expands to full capacity (cycle 4). Deprioritized; the reuse pattern this mechanism targets does not occur at sufficient density in the current workload.
- **CLOCK:** Eliminated linked-list pointer maintenance; confirmed an improvement over plain LRU in cycle 1, but weaker than SLRU because it acts only at eviction time. Its throughput advantages under high concurrency remain unexplored.
- **Frequency-window eviction (fixed 8-slot):** Bounded by the constant window in cycle 1; the structural ceiling was later confirmed to be the fixed-constant design rather than the frequency concept itself.
- **Second-chance reprieve bit:** Failed in cycle 3 because it froze node positions, severing the recency signal. The concept is not inherently flawed but requires position updates to remain intact.
