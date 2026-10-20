## Recurring wins

- Landing a re-referenced stash item at the MRU end of its target bucket is the only placement change that has held up. It was the winner in cycle 1 and has been the base for later stacked variants.
- Requiring a second hit before a stash item is promoted is the one filter that showed a hit-rate gain on its own (cycle 2). It kept its small gain when stacked on MRU landing (cycle 3).
- Admission and promotion filters that look at reuse help more than filters that change where items sit. Movement-changing edits have mostly hurt.
- Gating on persistent reuse (the 2-bit saturating counter, cycle 6) is the only reuse-tracking idea that showed any hit-rate signal, but the signal was marginal.

## Recurring failures

- Moving items out of their tier and back is harmful. Removing and re-inserting probing-tier hits (cycle 1) pushed proven-hot keys back through target competition and lost ground.
- Skipping the target retry for foreign evictions lowered hit-rate (cycle 1). Restoring it only when the stash is full recovered only part of the loss (cycle 2), so the shortcut is not worth pursuing as is.
- Using the seen-in-stash marker as the eviction criterion hurt (cycle 3), because it discards recently arrived items before they can prove reuse.
- Early readmission of recently evicted keys through a ghost-history fast path gave a slight hit-rate loss (cycle 5).
- Many small refinements are flat against their parent and should not be kept as leads: occupancy-conditional landing (cycle 2), occupancy-conditional promotion gate (cycle 4), earliest-landed stash victim selection (cycle 4), demotion placement at the LRU end (cycle 5), gate reordering (cycle 6), back-to-back reuse filter (cycle 7), reset-by-halving on demotion (cycle 7), and the probing-tier zero-hit admission filter (cycle 7).
- Changes that only alter layout or speed, such as the spare-bit marker (cycle 4), the occupancy counter (cycles 5 and 6), and the slot scan replacement (cycle 6), show no hit-rate change. Their benefit has never been measured.

## Open questions

- Does the second-hit promotion gate with MRU landing hold on workloads and capacities other than the LIRS trace at size 2048? Every cycle so far has used that single trace.
- What are the per-op-time and metadata-size costs of the promotion gate, its marker, and the saturating reuse counter? No cycle has recorded either metric.
- Winner labels do not match the reported evidence. Cycle 2's v001 matched its parent, cycle 4's v001 was unchanged from its parent, cycle 5's v001 was slightly below its parent, and cycle 7's v002 was within noise. Does the baseline for "Winner" need to be stated explicitly?
- Several insights describe intended changes rather than verified overlay diffs. Is the marker really packed into a spare bit, and is the counter reset on demotion and eviction implemented as specified?
- The rationale for the Front/Back default is described inconsistently across cycles. Which description is authoritative?

## Per-policy notes

- Stash tier (LIRS-style, size 2048): MRU landing and the second-hit gate are the only stash-side levers with any measured hit-rate signal. Victim-order and occupancy-conditional variants have shown no effect. Keep the MRU and second-hit combination as the base and test it on more workloads.
- Probing tier: moving hits out and back is harmful, and the zero-hit admission filter was flat. Do not pursue further probing-tier movement without a measured cost offset.
- Normal tier and eviction path: skipping the target retry for foreign evictions lowered hit-rate, and demotion placement changes were flat. The eviction path needs stash-capacity evidence before it is revisited.
- Reuse metadata (seen-in-stash marker, 2-bit saturating counter): the marker works as an admission signal but not as an eviction signal. The saturating counter is the only reuse-tracking lead, and it needs a second workload and cost measurements before it is kept.
