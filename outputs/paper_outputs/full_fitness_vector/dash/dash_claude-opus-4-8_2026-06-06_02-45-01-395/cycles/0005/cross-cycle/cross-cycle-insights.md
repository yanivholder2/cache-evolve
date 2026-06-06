## Recurring wins

- Removing hidden hot-path cost reliably trims latency: O(n) front-shifts,
  redundant candidate scans, and per-op syscalls (`Instant::now`) were all
  cheap, repeatable wins early on.
- Per-entry metadata slimming paid off until exhausted: under a positional
  policy the `Item` carried timestamp + frequency fields the policy never
  read, so deleting them cut size and latency for free down to the key+value
  floor.
- The structure-vs-layout pivot beat per-entry tuning once layout saturated:
  relocating the promotion-heat signal into a segment-side table reached the
  long-sought Pareto corner (best hit-rate AT the metadata floor).
- Placement and footprint are orthogonal — porting a slim `Item` onto the
  MRU-promotion winner stacked both wins with no hit-rate cost.
- Devirtualizing the inner eviction-policy trait object produced a
  harness-resolvable latency win, establishing the granularity of change
  that is actually measurable.

## Recurring failures

- Tuning the promotion gate (width, positional vs frequency shape) keeps not
  paying off: MRU placement already captures the warm mass, so gate shape and
  placement are substitutes, not additive — gains land in the noise floor.
- Chasing hit-rate on the hot `get` path costs real latency (promotion
  cascades, per-hit counter RMW) for indistinguishable gain.
- Micro-latency reorders (single scan-per-hit savings) sit below measurement
  resolution and often move the wrong way due to variance.
- Raising set-associativity left hit-rate bit-for-bit unchanged — misses are
  capacity/compulsory-bound, not conflict-bound.
- Frequency/admission (ghost-list) levers moved hit-rate nothing, only adding
  put-side bookkeeping cost.
- Stacking the size (side-table) and latency (devirtualization) winners on
  one hot path is substitutive: the side-table hash probe, not the vtable,
  bounds latency, so the two speedups cannot be collected together.
- Outer-layer monomorphization regressed everything: replacing a compact heap
  pointer with a fat inline struct widened `Vec` stride and cost locality more
  than the saved dispatch.

## Open questions

- Is there a hot-path layout where side-table relocation and devirtualization
  stop competing, or are the metadata-floor corner and latency floor mutually
  exclusive?
- Is there a structural change that removes indirection while preserving or
  improving locality (rather than fattening layout or adding bookkeeping), or
  has latency also saturated?
- Where does the next size win come from now that per-entry layout is at its
  floor — bucket/segment structure, allocation strategy?
- Can any latency win reduce p99 rather than just p50? Side tables so far
  trade tail latency for size.

## Per-policy notes

- ClassicLRU (active eval policy): purely positional — ignores recency/
  frequency metadata, which is why metadata trimming was near-free and why
  frequency/admission levers do nothing.
- Hit-rate under ClassicLRU is confirmed capacity/compulsory-capped across
  every tried class: placement, gate shape, associativity, and admission all
  exhausted. Treat hit-rate as fully closed.
- The frontier stands as two banked, substitutive corners: a devirtualized
  latency baseline and a side-table size corner at the metadata floor.
