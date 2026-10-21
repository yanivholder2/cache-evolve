# Variant v002 Insight

## Helped

- hit-rate: The parent's `evict_main` called `pop_back` and then `push_front` for every survivor with `freq > 0`. Each survivor jumped ahead of every freq-0 key that arrived after it, so those newer keys reached the tail first. The new SIEVE hand leaves survivors where they are. Ghost-readmitted keys enter at freq 0 and are the most fragile entries in main. They now sit near the head, behind the hand, so they get a full sweep of residency to collect hits before they are examined. Older survivors whose credit has drained get re-examined sooner, so they become the victims in place of those fresh re-entries. The move is in the direction H02 predicted, and it is larger than the ±0.001 band the hypothesis called informative. That makes this the first main-side lever after a run of flat partition, credit, graduation and re-admission changes. Everything else is held fixed (5% small, `MAX_FREQ = 7`, `freq > 1` graduation, freq-0 head re-admission, ghosting main victims, the fingerprint ring). The updated `matches_ring_reference_on_pseudo_random_streams` checks that small, ghost ring, `ghost_head` and residency all match the reference, and it now also checks main order and hand position. So the gain comes from victim order alone.

## Hurt

(No measured dimension regressed. Hit-rate is the only dimension in the eval. Two costs of the change go unmeasured. First, each `Entry` now carries `prev: u32` and `next: u32`, which adds 8 bytes of metadata per resident entry. That replaces the parent's `VecDeque<usize>` for main, which spent 8 bytes per main slot plus spare capacity, so the net metadata change is roughly neutral. Second, a sweep step is now a pointer chase across slab slots instead of contiguous deque pops and pushes. Survivors are never moved, which saves writes, but a step can now land in a different cache line. The effect on per-op time is unknown.)

## Takeaway

Holding the policy fixed, switching main from FIFO reinsertion to SIEVE in-place retention measurably raises hit-rate on workload-a at size 2048. On this trace, the order in which main examines entries matters more than any of the partition and credit knobs tried so far. The likely mechanism is that freq-0 ghost re-admissions are no longer pushed toward the tail by reinserted survivors. A natural next probe is to tune the SIEVE retention schedule itself, for example a lower `MAX_FREQ` or clearing freq on pass instead of decrementing it. With the hand, a high freq cap now means more sweep laps rather than more trips back to the head.
