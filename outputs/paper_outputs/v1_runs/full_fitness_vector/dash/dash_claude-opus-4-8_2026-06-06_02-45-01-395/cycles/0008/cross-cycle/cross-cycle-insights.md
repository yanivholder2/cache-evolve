# Cross-Cycle Insights

## Recurring wins

- Removing hidden hot-path cost is the most repeatable latency win: O(n)
  front-shifts, `Instant::now()` syscalls, and SipHash-on-integer-keys all
  trimmed op-latency once excised, with no algorithmic cleverness needed.
- Structural latency changes that *remove indirection while preserving
  locality* pay off: devirtualizing the inner eviction-policy trait object,
  and swapping the keyed permutation for a trivial `u64` hasher on the
  side-table probe, both cleared the noise floor.
- Per-entry metadata slimming was a near-free size+latency win until it
  bottomed out — every field the positional policy never reads (timestamp,
  LFU counter) was pure padding once removed.
- Relocating the promotion-heat signal out of `Item` into a segment-side
  table reached the long-sought Pareto corner: best hit-rate held at the
  metadata floor.
- Variance reducers are safe and hit-rate-neutral: capacity pre-reservation
  and a 2×-sized direct-mapped array cut p99 tails and put-p50 bit-for-bit.

## Recurring failures

- Tuning hit-rate via placement, gate width, gate shape, set-associativity,
  or admission/ghost lists — all saturate. Gains are noise-level because
  hit-rate is capacity/compulsory-bound, and gate shape is a *substitute* for
  MRU placement, not additive.
- Sub-scan micro-optimizations (branch reorders, removing one scan per hit)
  sit below the harness noise floor and often move latency the wrong way.
- Stacking two banked winners on one hot path keeps failing: the side-table
  size corner and the devirtualized latency floor are *substitutive* on the
  shared get path (the hash probe, not the vtable, bounds it).
- Moving indirection the wrong way regresses locality: outer-layer
  monomorphization fattened the `Vec` stride; a position-aligned parallel
  `Vec<u8>` paid an O(n) shift on every reorder. Heap pointers were buying
  locality.
- Relocating work *off* the median path is not free — it resurfaces on the
  destination branches as a tail (p99) regression (per-put heat clear,
  eviction-site moves).

## Open questions

- The get median resolves in the **stash**, not the promoted tiers — does a
  stash-*first* reordering actually win get-p50, or just relocate the cost?
- The put-p99 cascade is now a priced latency-vs-hit-rate trade (single-hop
  demotion rescues a few hits). Is it worth banking, and is there a cheaper
  conflict-mitigation step that keeps most rescued hits?
- Is get latency fully converged at the hasher floor, or is any median lever
  left?
- "Hit-rate fully capped" is no longer exact: the demotion/conflict machinery
  shows real sensitivity. How much hit-rate does the cascade actually buy?

## Per-policy notes

- **ClassicLRU (active eval policy):** purely positional/recency-driven. It
  reads no frequency or recency metadata, so all such fields are dead weight
  on size and latency, and frequency/admission levers move hit-rate not at
  all. Warm mass already concentrates near the MRU head, so MRU placement
  captures most protectable retention and makes gate-shape tuning redundant.
  Hit-rate is recency-bound and largely capacity/compulsory-limited, with the
  only known crack in the demotion-cascade trade.
