# Cycle 0003 Insight

## Cohort outcome

The cohort probed three orthogonal directions off the cycle-2 baselines: isolating ghost admission's marginal contribution, rescuing frequency-aware demotion with a larger scan window and persistent counters, and testing a second-chance reprieve bit in the protected segment. Only the repaired frequency-demotion variant (v002) won, delivering a meaningful hit-rate gain by scanning the LRU quarter of protected and halving rather than resetting counters on demotion.

## Patterns

- Ghost admission is now confirmed as minimally effective on this workload. When grafted cleanly onto the cycle-1 SLRU baseline, it produced negligible lift, resolving cycle-2's ambiguity: the earlier gain attributed to ghost admission was almost entirely SLRU segmentation doing the work. The ghost window (bounded to the probationary capacity) is too narrow to intercept the reuse gap typical of this trace.
- Cycle-2's frequency-demotion failure was entirely a design flaw, not a fundamental ceiling. Scaling the scan window to a fixed fraction of the protected segment — rather than a constant small count — creates genuine selection pressure. Counter halving on demotion is the necessary complement: it lets accumulated history survive segment transitions so newly promoted nodes remain distinguishable from long-tenured ones.
- The second-chance bit failed because it required freezing node position on protected hits, which severed the continuous recency signal that makes LRU ordering useful. A one-time reprieve cannot substitute for a move-to-tail on every access; the hot entries it was meant to protect end up stranded near the demotion head anyway.
- Continuous position updates on protected hits are load-bearing. Any mechanism that suppresses them — regardless of its reprieve logic — degrades the protected segment's ordering and should not be adopted unless the position-signal loss is compensated elsewhere.

## Open questions

- Can frequency-aware demotion (v002) coexist with ghost admission without noise? Given ghost admission's negligible effect on workload-a, the combination is likely neutral, but the interaction has not been tested directly.
- Is the LRU-quarter scan fraction robust across workload shapes, or does the right window size vary with the reuse distance distribution? A workload with longer reuse distances might need a wider scan.
- Could a second-chance approach be salvaged by setting the referenced bit while still performing the move-to-tail (preserving recency ordering)? The current failure was the position-freeze trade-off, not the reprieve concept itself.

## Continuation

Next cycle should take v002 as the new baseline: two-segment SLRU with ghost admission, frequency-aware demotion scanning the LRU quarter of protected, and counter halving on demotion. Ghost admission can be deprioritized — it has been tested in isolation and found to contribute little on this workload. A productive direction is to explore whether the scan fraction (currently one quarter) should be adaptive or tuned, and whether combining frequency-aware demotion with a significantly larger ghost window (e.g., equal to total capacity rather than probationary capacity) can make cross-residency warmth visible enough to matter.
