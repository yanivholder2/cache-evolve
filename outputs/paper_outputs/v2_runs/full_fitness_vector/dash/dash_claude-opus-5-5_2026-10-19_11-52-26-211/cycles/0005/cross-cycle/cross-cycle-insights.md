## Recurring wins

- **Stash ordered as a probationary queue by proven reuse (c1–c2).** Demotees enter the stash at the MRU (Back) end and fresh puts enter at the LRU (Front) end. The two gains add up, and the result is the stable admission baseline for every later lineage.
- **Using idle capacity is the only hit-rate lever since c2 (c4–c5).** The two-choice stash (neighbour stash bucket) and rescuing stash victims into normal-tier holes each raise hit-rate. They draw on different capacity, so their gains stack. Neither adds metadata.
- **State-neutral lookup reordering moves latency onto rare paths (c1, c5).** A key lives in only one slot, so reordering probes never changes hit-rate. That has been confirmed several times by exact hit-rate matches. Scanning the rarest tier last (currently `s1`) charges only misses and the hits that land there.
- **Combinations reproduce the expected state (c2, c5).** Independent levers such as lookup order, insert position, neighbour stash and rescue have composed cleanly every time they were tested.
- **Metadata-size has never moved.** Every change so far derives its decisions from the hash, bucket position or fullness.

## Recurring failures

- **Admission filters on either promotion edge (c2–c3).** MRU placement on stash→target promotion, a second-hit gate, and native-ownership gates on target→probing all lowered hit-rate. The tiny buckets can't hold items long enough for an extra wait to pay off. Front placement on promotion already filters well enough.
- **Eager filling of holes with unproven items (c5).** Putting fresh items into target holes fills the normal tier with cold entries. Later reuse-proven promotions then trigger eviction cascades. Late filling (rescue) gets the same capacity without that cost.
- **Pure bookkeeping trims on the promotion path (c4).** Removing redundant hashing or re-scans doesn't reach get p50. The median is set by the scan on the common lookup path and by the moves themselves.
- **Tweaks to rarely-triggered placement paths (c1).** Balanced placement on stash hits was safe but had almost no effect.
- **Treating a fixed lookup order as universal (c1→c3).** Target-first won under the old placement, and stash-first won once fresh items were placed at stash Front. Each new placement policy needs its own order test.
- **Trusting raw latency deltas.** Large shifts on paths a variant never touched appeared repeatedly. Only credit per-op-time changes that have a code-level mechanism and a baseline measured in the same run.

## Open questions

- Does the c5 parent (two-choice stash + rescue, `s1` probed last) keep hit-rate exactly and recover get p50 as predicted? This is not yet verified.
- Can most `s1` probes on misses be skipped cheaply, for example with a per-`s0` overflow bit or fingerprints? What would that cost in metadata-size?
- How much does each of the neighbour-stash and rescue levers contribute separately? Would an independent second hash for `s1` beat the fixed `+1` stride?
- Can the promotion cascade be made cheaper without changing who gets promoted, for example with in-place swaps or a bounded cascade depth?
- Structural levers not yet tried: stash capacity and associativity, and eviction order inside target buckets. Bigger stash buckets might also let a second-hit filter work where 2-slot buckets couldn't.
- Is there a form of fresh-into-target fill that always gives way to promoted items and adds hit-rate on top of rescue?
- Hit-rate and get latency are now traded off directly, because more residency means more scanning. Can cheaper per-bucket scans break that link?

## Per-policy notes

- **Stash tier (holds most capacity):** this is the main hit-rate lever. Insert position matters a lot (reuse-ranked queue), and spreading over a neighbour bucket helps. The stash-hit → promote → demote cascade is the slowest hit path, and it carries more of the load as hit-rate rises.
- **Target tier:** very small buckets. Promotions in should land at Front/LRU, because MRU placement evicts residents that have already been hit. Free target holes are best filled late by rescued victims, not eagerly by fresh puts.
- **Probing tier:** gating promotions into probing (ownership or second hit) costs hit-rate. Climbing within the bucket under plain LRU order keeps the tier selective enough.
- **Lookup order:** the best order depends on the placement policy. On the current lineage it is `s0 → target → probing → s1`. The remaining get gap is the extra bucket that misses scan.
