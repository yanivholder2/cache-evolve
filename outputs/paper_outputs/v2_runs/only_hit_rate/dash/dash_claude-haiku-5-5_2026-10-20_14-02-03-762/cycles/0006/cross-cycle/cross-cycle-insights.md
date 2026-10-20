## Recurring wins

- Stash-hit landing at the MRU end of the target bucket is the one placement change that has held up. It was the only winner in cycle 1 and remained the base in later cycles.
- Admission filtering on reuse helps a little. Requiring a second hit before stash promotion (cycle 2) and, in weaker form, a saturating reuse counter (cycle 6) both point the same way: one-off hits displace residents that have not yet proven reuse.
- Stacking the second-hit gate on MRU landing (cycle 3) gave a small further gain. The effect is small and rests on one workload and one capacity.

## Recurring failures

- Moving items out of their current tier hurts. Removing and re-inserting probing-tier hits (cycle 1) and skipping the target retry for foreign evictions (cycle 1) both lowered hit-rate. The unconditional direct-to-stash shortcut (cycle 2) recovered only part of its loss when gated on free stash capacity, so the removed retry was valued under pressure.
- Using stash-marker state as an eviction signal hurts. Preferring never-re-hit stash entries as victims (cycle 3) discarded recently arrived items before they could prove reuse.
- Early readmission hurts slightly. The ghost-history fast path (cycle 5) let recently evicted keys skip the stash wait and lost a little hit-rate.
- Small refinements of landing, gating, and victim order show no measurable effect. Occupancy-conditional landing (cycle 2), occupancy-conditional gating (cycle 4), earliest-landed victim order (cycle 4), LRU-end demotion placement (cycle 5), and the gate reorder (cycle 6) all matched their parent. Occupancy counters (cycles 5 and 6) also matched parent, so their speed benefit remains unmeasured.
- Spare-bit marker relocation (cycle 3, cycle 4) left hit-rate unchanged. This does not show that any metadata saving was realised.

## Open questions

- Does any winning lever (MRU landing, second-hit gate, reuse counter) hold on workloads and capacities other than the single LIRS trace? Every cycle so far used that one trace.
- Per-op-time and metadata-size have not been measured in any cycle. The cost side of every lever, including the reuse counter's extra bits per entry, is unknown.
- Several insights rest on hypothesis text rather than the overlay diff (cycles 2, 4, 5, 6). Whether the implementations match their descriptions, including the spare-bit layout and the reuse counter's reset on demotion and eviction, is unverified.
- Winner labelling is inconsistent. Some winners were inherited or carried forward with no change from parent (cycles 2 and 4), and one was labelled a winner despite a slight hit-rate drop (cycle 5). The baseline used for "win" needs to be fixed.

## Per-policy notes

- All six cycles explore one policy lineage: the probing, stash, and normal tier design evaluated on the LIRS trace. No cross-policy comparison exists yet.
- Stash tier: landing position and reuse-based promotion are the only levers with a hit-rate signal. Victim selection and occupancy-dependent variants have shown no effect.
- Probing tier: residency is valuable. Changes that remove or re-insert probing hits have consistently lost ground.
- Normal/target tier: foreign-eviction retries and demotion placement carry real but conditional value. They are not promising levers by themselves.
- Ghost history: readmission shortcuts have only a negative measured result so far.
