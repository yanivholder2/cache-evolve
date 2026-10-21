# Cycle 0010 Insight

## Cohort outcome

All three variants kept each key in its single hashed bucket on the cycle/0009/v002 base. Following Cycle 0009's continuation, the cohort tried one last placement lever (v001: a better-mixing bucket-index hash) and two cost-only changes. v002 moved the GCLOCK counter out of `Item` into a packed per-bucket word. v003 put a SWAR byte-tag filter in front of the bucket scan and shared one hash between the lookup and the ghost check. v002 and v003 win because they held hit rate exactly. v001 also held hit rate within noise, but it adds per-lookup cost and gains nothing, so H01 is refuted.

## Patterns

- The placement lever is used up. Alternate buckets are forbidden (Cycle 0009), and hash quality now changes nothing. The remaining misses on workload-a look like capacity or in-bucket replacement misses, not hash-skew conflicts. All levers that decide which bucket a key goes to, and which slot gets evicted inside it, have now been tried.
- This makes five cost-only changes in a row that kept hit rate exactly (Cycles 0007, 0008 and 0009, plus v002 and v003). Moving metadata from the item into positional per-bucket storage, or into a parallel side array, does not change the decision sequence as long as it stays in lockstep with `items`. The risky spot is the shift on `remove_at`. Both winners guard it with edge tests and debug asserts against the old path.
- Neither winner's savings are one-sided, and neither can be measured. v002's item shrink is partly cancelled because the bucket got fatter than the hypothesis planned: a wider counter word plus always-present side `Vec` headers. v003 moves hashing onto the hit path, and the miss path still re-hashes the evicted key, so it does not reach "one hash per op". The trade-offs flagged in Cycles 0007 and 0009 (hashing versus memory) are now built into two candidates.
- This is the fifth cycle with no hit-rate gain on workload-a, and the fifth in which metadata-size and per-op-time went unrecorded. Two winners that behave identically now differ only on dimensions the evaluation does not record.

## Open questions

- If v002's side storage were conditional, for example an enum so that clock mode carries only one counter word, would the net per-entry saving come close to the planned item size?
- Does v003's byte-tag rejection beat direct `u64` key compares over a 16-way bucket once hashing on hits is counted? Would hashing the evicted key once at insert, or storing its fingerprint, close the miss-path gap?
- v002 and v003 touch separate state. Can they be combined, with the tag array and the packed counters together, without breaking each other's lockstep invariants?

## Continuation

Use cycle/0009/v002 as the behavioural reference. Both cycle/0010/v002 (packed counters) and cycle/0010/v003 (tag filter) are equal-hit-rate candidates whose value depends on cost metrics that do not exist yet. Drop all placement and bucket-hash ideas, and do not propose any hit-rate tweak on workload-a. The next cycle should either make recording metadata-size and per-op-time the explicit deliverable, or refine the two winners' known leaks: trim v002's side storage, and remove v003's evicted-key re-hash and its unused hash on large buckets. A combined v002+v003 variant only makes sense if those metrics will judge it.
