## Recurring wins

- Id-keyed indexing for resident and ghost lookup is the most repeated direction (Cycles 0005–0008, merged index in 0006). It removes linear scans on the put path and shows no hit-rate change. Get and put speedups still need a baseline rerun before they are credited.
- Fingerprint-keyed ghost storage has given a put-side benefit with no hit-rate change in several cycles (0003, 0004, 0005). Its cost is container-level bookkeeping, not per-entry size. Combining it with a ghost cap (0006) eroded that put gain.
- Batched ghost expiry takes drain work off the per-eviction path (0007 window, 0010 batched prune). It is a useful put-path tool even where it does not change hit rate.
- Policy-preserving data-structure changes (index swaps, counter relocation) leave hit rate unchanged. That is expected, and it makes them safe to keep when the cost case is clear.

## Recurring failures

- Weakening frequency signals loses hit rate. Lowering the frequency ceiling to 2 (0008), aging main-queue counters (0009), and promoting on `freq > 0` (0001) each cost hit rate on the trace tested. The frequency signal appears to earn its keep.
- Enlarging the ghost toward full capacity lost hit rate (0004). Ghost capacity is not a free dial.
- Victim choice after the rotation budget did not compose with a larger small queue (0002, dropped in 0003).
- Hash-function swaps did not reduce get cost; the multiply-shift hasher made gets slower (0008).
- Splitting per-entry state into side arrays saved bytes in the entry but grew the container, a net footprint loss (0004).
- Linear-probing ghost tables showed a put-tail regression (0009). Tombstone tables at lower load factor (0010) addressed it only partially, since the change bundled two edits.
- Get-path timing shifts appear in variants whose get code is unchanged (0001–0010). Treat them as shared noise, not as get-path effects.
- Labeling is inconsistent. Several cycles labeled variants Winner when their own evidence showed no hit-rate gain and no clear, unambiguous cost gain (0002, 0003, 0009, 0010). Winner labels should be checked against the evidence before carrying a variant forward.
- Nearly every hit-rate movement rests on one trace at one capacity. None of the hit-rate results is yet robust.

## Open questions

- Is the get-path shift real and reproducible with no code change? It has been unexplained for ten cycles and needs a baseline rerun.
- Does the fingerprint ghost's put-latency gain hold across traces and capacities, and is its hash-set cost acceptable at larger sizes? Heap-side ghost bytes have not been measured.
- Does the ghost cap's small hit-rate gain (0005, 0009) survive on other traces, and does it still help once expiry is batched (0010 found it flat)?
- Do the ghost-storage variants (fingerprint, tombstone table, bit array, index) compose? None has been tested in combination, and they touch overlapping state.
- Does the bit array's hit-rate uptick (0010) hold on other workloads, given its false-positive rate?
- Does a much longer aging period, or per-entry unhit windows, recover any hit rate lost to frequency aging?

## Per-policy notes

- Small queue: the promotion threshold at `freq > 0` was dropped (0001). A larger share alone showed no hit-rate change (0006). A larger share paired with victim choice gave a small single-trace gain (0002), but the gain did not carry over when the two were stacked (0003).
- Main queue: the rotation budget is the bounding knob. Post-budget victim choice is untested in the enlarged-small-queue regime. Frequency aging hurt hit rate (0009). The frequency ceiling of 3 should stay (0008).
- Ghost queue: fingerprint storage is the most consistent put-side win. Full-capacity ghosts lost hit rate (0004). A half-main cap gave a small gain that did not reproduce once expiry was batched (0010). Batched expiry is worth keeping. The tombstone table and bit array are unverified on other traces.
- Metadata and index: id-keyed maps are the lead direction for lookup cost. Counter relocation out of `Entry` is hit-rate neutral, but its footprint effect is unmeasured. Hasher changes are not a get-path lever.
