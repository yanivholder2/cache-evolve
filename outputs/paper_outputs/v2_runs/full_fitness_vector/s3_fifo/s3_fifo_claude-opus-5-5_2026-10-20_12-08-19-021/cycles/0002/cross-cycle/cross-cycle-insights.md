## Recurring wins

- Capacity levers beat movement-rule levers. Making the small/main split adaptive (ARC-style) is the only change so far that improved hit-rate on workload-a, and it added no per-entry metadata.
- Structural/data-structure fixes that keep policy behaviour unchanged give clean latency wins. Replacing linear scans with hashed indexing kept hit-rate identical and cut `get`/`put` latency sharply. Stacking it on the adaptive-split parent produced the current best lineage (0001 v003 → 0002 v003).
- Building on the latest winner, one change per cycle, has produced every gain so far.

## Recurring failures

- Movement-rule tweaks layered on the adaptive split have not paid off. The looser graduation threshold was flat, and ghosting main victims was slightly worse. The likely cause is that extra graduations or ghost hits feed the split adaptation and cancel the intended effect. This is unconfirmed.
- Latency claims measured on scan-based parents were unreliable. Their latency was dominated by O(capacity) scans, so latency attributed to policy changes in early cycles, gains and regressions alike, should be discounted.
- Infrastructure flakiness (Maven 429s at build time) wasted evaluation slots in cycle 0001. A rejection caused by a build failure says nothing about the hypothesis.
- "Winner" status within a weak cohort does not by itself show the variant beat its parent. Check its result against its parent directly.

## Open questions

- Where does `small_size` settle under the adaptive split? Is the gain from adaptation itself, or just from moving to a better fixed split? No variant has reported or compared against a fixed split near the settled value.
- The resize triggers are lopsided: "grow main on graduation" fires almost all the time. Would rebalancing them (or making grow-small fire less often) improve hit-rate?
- Could a main-victim ghost kept separate from the split adaptation, so it does not trigger "grow main", recover hit-rate from the ghost-main idea?
- On the hashed base, do movement-rule changes still cost latency, or was that cost only a side effect of the scans?
- `metadata-size` only counts `Entry<K,V>` and misses structural overhead (indexes, ghost structures). Real memory cost of the current lineage is under-reported.

## Per-policy notes

- S3-FIFO baseline (fixed 25/75 small/main split): the fixed split is not optimal for workload-a, and capacity adaptation beats it.
- S3-FIFO + adaptive split: hit-rate leader. It seems sensitive to anything that changes graduation or ghost-hit rates, since those signals drive resizing.
- S3-FIFO + adaptive split + hashed indexing: current parent and latency baseline. Compare all future latency against it. Its hidden structural memory cost is not captured by the metadata metric.
- Graduation-threshold variants: dropped after a flat result on the adaptive base.
- Ghost-main-victim variants: retry only in a form decoupled from the split adaptation.
