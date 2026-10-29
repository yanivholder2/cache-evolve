# Cycle 0009 Insight

## Cohort outcome

All three variants built on cycle-0008 v001 (CLOCK victim bit). v001 started ghost-returning promotees with the reference bit set. v002 required an extra stash hit when every resident of the full target was referenced. v003 split the ghost ring into per-stash-bucket sub-rings to cut put-path cost. The Winner is v001, but only by not regressing: its 128K gains are rounding-level noise and `web_0` 16K stayed flat for yet another cycle. v002 matched the parent exactly. v003 lost a little everywhere except a noise-level uptick at `web_0` 16K.

## Patterns

- A 1-bit CLOCK in 2-slot buckets can't carry a value signal, and both bit-based levers ran into that. MRU position already protects a newcomer, so setting its bit (v001) can only cancel the LRU resident's protection. MRU hits send items to probing, LRU hits move items to MRU, and entry plus `make_room_clock` clear bits, so an "all residents referenced" state (v002) almost never happens. The bit gets reset before anything reads it.
- The ghost's reuse evidence is real, but it does nothing on victim choice unless it lands where position doesn't already decide the outcome, and unless it is held in state that survives clearing.
- The ghost works because it is global and adaptive. A shared FIFO lends its reach to whichever buckets are evicting. Fixed per-bucket sub-rings (v003) cut reach in exactly the hot buckets, and the rescue path makes evictions more skewed, hitting `hm_0` 128K hardest. This confirms that the ghost lifecycle should stay settled.
- `web_0` 16K has now been flat through ghost reach, placement, free-slot routing, CLOCK victim choice, ghost-carried bits and value-aware gating. Any lever that relies on 1-bit or position state is exhausted there.
- Per-op-time and metadata-size are still unmeasured, now nine cycles in a row. v003's only possible benefit is therefore unverifiable.

## Open questions

- If ghost-origin promotees were placed at the LRU end with bit 1, would CLOCK finally spare them over an unreferenced MRU neighbour?
- Would a multi-bit, per-resident hit count that is not cleared on entry or by the sweep give value-aware victim choice or gating something to act on?
- Could a per-bucket index or filter over the global ghost FIFO make lookups cheaper while keeping global reach?

## Continuation

Build on v001, or on its parent, since the two are effectively the same. Drop single-bit CLOCK tweaks and occupancy- or all-referenced gates, and keep the global ghost FIFO. Partitioned rings are ruled out. For `web_0` 16K, the remaining untried directions are a multi-bit resident counter that persists through entry and sweeps, or ghost-origin placement at LRU with bit 1. If per-op-time is not added to the fitness vector, stop proposing cost-only variants.
