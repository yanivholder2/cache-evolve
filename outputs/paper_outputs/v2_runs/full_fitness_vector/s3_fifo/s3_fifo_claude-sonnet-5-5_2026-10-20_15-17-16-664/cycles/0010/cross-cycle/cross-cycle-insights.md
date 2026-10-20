## Recurring wins

- Changes that remove redundant work with a clear mechanism keep paying off, with hit rate unchanged. Examples: scan consolidation, hash-index membership, O(1) ring ghost, tagged single index, split ghost map, keyed single lookup.
- The biggest `put` gain came from cutting hash-map probes per `put` (Cycle 6). Ghost maintenance was the earlier lever (Cycle 5).
- A cheap hasher helped `get` on the tagged index (Cycles 6–7).
- Shrinking the small queue from 10% to 5% to 3% gave small hit-rate gains, with diminishing returns. It is the only hit-rate lever that has worked.
- Gains with a mechanism show up in the tails (`put`/`get` p99). Medians are dominated by noise.

## Recurring failures

- Anything that loosens main admission lowered hit rate or gave nothing. This covers promotion at freq>=1, a larger small queue, ghost recycling, ghost head starts, a larger ghost, a recent-main second chance, freq reset and ghost age-gating. The small-queue filter matters most.
- Parameter and policy micro-tweaks are saturated on this trace: ratio sweeps, frequency cap, adaptive sizing, and the rotation cap.
- Adding per-entry state or counters (sequence numbers, carried hashes, adaptive counters) costs more latency than it returns.
- Hasher, finalizer and pre-size work is exhausted. It trades `get` against `put` tail with no free win, and hashing small integer keys is not the bottleneck.
- Reworking main-queue eviction and micro-optimizing the frequency store or deque showed no gain.
- Since Cycle 8, Winners selected without a mechanism are likely noise-assisted. Latency drops on one-constant edits are noise.
- Per-entry `metadata-size` hides whole-cache heap growth from the index and maps.

## Open questions

- What is the repeated-run noise floor for latency and hit rate? Which earlier gains survive it?
- Do the 3–5% small ratio and the structural gains hold on other traces and cache sizes?
- What is the real heap footprint of the tagged index, split ghost map and keyed lookup?
- Do the Cycle 9 keyed lookup and the Cycle 6–7 tagged index and split ghost map conflict, and can they be merged cleanly?
- What remains in the `put` tail (eviction, hash-map operations, small-to-main move)?
- Is this trace near its ceiling for this policy family? Would a different workload shape expose gains from promotion or ghost policy?
- Is special handling of main-evicted keys worthwhile? It would need an O(1) tagged design with evidence of faster reuse.

## Per-policy notes

- S3-FIFO (the only policy so far): the current base is a small queue of about 3–5%, a tagged index, a separate ghost map, a ring-buffer ghost and a cheap hasher.
- Hit rate has been nearly flat since the small-ratio gains. Every later gain is latency only.
- The ghost works as a one-hit-wonder filter. Sharing it with main evictions dilutes it, and a head start or larger size hurts.
- Main frequency cap 3 protects reused entries, and lowering it costs hit rate.
- All evidence is from one trace (workload-a) at one size. Future cycles should test several traces, run repeats, measure heap, and use only hypotheses with a clear mechanism.
