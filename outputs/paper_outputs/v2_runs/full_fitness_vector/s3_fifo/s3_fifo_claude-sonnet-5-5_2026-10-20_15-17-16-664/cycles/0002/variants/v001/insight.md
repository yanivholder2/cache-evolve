# Variant v001 Insight

## Helped

- (none) No dimension improved. The hit-rate moved slightly the wrong way, and the other two dimensions did not improve.

## Hurt

- hit-rate: Promoting small-queue tail objects to main after a single re-reference (freq >= 1) lets more one-hit-wonder-like objects into main. Those objects displace entries that have proven themselves, so the hit rate ended slightly below the parent's. The loss is small on this one trace.
- per-op-time: Get and put latency both look worse than the parent's. The edit is a single comparison in the eviction branch and adds no per-access work, so the mechanism is unclear. The shift is most likely run-to-run measurement variance, as the parent's insight already suspected for its own latency drop. Don't read it as a cost of the lower threshold without a re-run.

## Takeaway

Lowering the promotion threshold to freq >= 1 on top of the 10% small queue did not beat the 10% ratio alone. The small queue's filtering role seems to matter more than quickly promoting re-referenced objects. The 10% small queue already makes that filtering window short, so a looser threshold mostly adds pollution to main. metadata-size is unchanged because no fields were added. I could only read the hypothesis and the fitness vectors, not the overlay source, so this interpretation rests on the hypothesis description of the change.
