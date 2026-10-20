# Cross-Cycle Insights

## Recurring wins

- **Stash as a scan-resistant probationary queue (cycles 1–2).** The stash ranks items by proven reuse: demotees go to the MRU/Back end and fresh puts go to the LRU/Front end. The two gains added up and are still the base of every later parent.
- **Using idle capacity is the only hit-rate lever that has worked since cycle 2 (cycles 4–6).** The working forms are a two-choice stash (neighbour stash bucket) and rescuing stash-eviction victims into target/probing holes. Their gains stack because they draw on different capacity. Neither adds metadata.
- **State-neutral changes are reliable and easy to verify.** A key lives in exactly one slot, so changing lookup order or adding lossless filters leaves hit-rate exactly the same (cycles 2, 5, 6). Examples: probing `s1` last, and an exact per-segment borrowed counter that skips the `s1` scan. Use these to recover latency.
- **Lossless filters stored per segment, not per item (cycle 6)** cut get p50 without growing tracked metadata. Their cost shows up on put, mostly in the tail.
- **Changes on independent paths combine cleanly.** Stash insert position, lookup order and stash-overflow placement have recombined without interfering.
- **Metadata-size has never moved** across any variant so far.

## Recurring failures

- **Admission filters on either promotion edge (cycles 2–3).** These include MRU placement on stash→target, a second-hit gate, and an ownership gate on target→probing. Hotter promotion evicts proven residents. Stricter promotion blocks real reuse, because the tiny buckets can't hold items long enough to wait. Current admission looks like a local optimum.
- **Tweaks to rarely-fired corner cases (cycles 1, 6).** Examples: balanced placement on stash hits, and native-victim-into-probing-hole. Such holes almost never exist in steady state. These changes add hot-path branches for no gain.
- **Filling holes early (cycle 5).** Putting fresh items straight into target holes fills the normal tier with cold items. Later promotions then pay for eviction cascades that used to be free inserts. Filling holes late (rescue) is cheaper.
- **Bookkeeping trims on the promotion path (cycle 4)** don't reach get p50. The median is set by the common lookup scan and by the promotion moves themselves.
- **Using lookup order to fix residency cost (cycles 3, 5, 6).** The best order depends on how hits split across tiers, which changes with placement. The cost of fuller buckets is per scan and doesn't depend on order. Further order changes are exhausted.
- **Latency claims without a code-level mechanism.** Environment shifts have repeatedly moved get and put on paths that weren't touched (cycle 1 especially). Only credit per-op-time changes that have a causal path in the code.

## Open questions

- Does the borrowed-counter `s1` skip stack on top of two-choice + rescue? Does it still skip most probes once rescue changes stash turnover?
- Can the counter be maintained without re-hashing, by passing `s0` or a native flag from the caller, so that put p99 comes back?
- Can in-bucket filtering (tags/fingerprints) shorten the longer scans in rescue-filled buckets? Is the metadata cost worth it?
- Can the promotion cascade (remove → promote → demote → stash insert) be made cheaper without changing what gets promoted? Options: in-place swaps, or a bounded cascade depth.
- Structural levers not tried yet: stash capacity/associativity, an independent second hash for `s1` instead of the fixed stride, and eviction order inside target buckets.
- Does fresh-into-target add anything in a form that always gives way to promoted items?
- How do hits split between stash and normal tiers under the current placement? This is still unmeasured, and it drives which lookup-order and filter choices are right.

## Per-policy notes

- **Dash-style target/probing/stash (only policy so far).**
  - Hit-rate depends on stash eviction order and on how much idle capacity is kept in use.
  - Get latency depends on how many occupied slots each lookup scans. Higher residency trades directly against get p50.
  - Put cost is now a real axis to track: rescue checks, counter bookkeeping, and cascades into fuller buckets all add to it.
  - Current lineage: Back-demotion + fresh-at-stash-Front → two-choice stash with `s1` probed last → + rescue → + borrowed-counter gate (to be validated with no re-hashing).
