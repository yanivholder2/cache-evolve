# Cycle 0005 Insight

## Cohort outcome

All three variants were built on cycle-0004 v003 (BIP fresh-key insertion), and all three tried to rescue the fresh keys with moderate or long reuse distance that BIP sacrifices. v001 widened probation to two slots. v002 made the insertion choice adaptive with set-dueling DIP. v003 added a ghost table about one cache in size, where a ghost hit sends the returning key to the stash MRU end. Only v003 gained, and only a little. v001 was flat to slightly worse and v002 lost hit-rate.

## Patterns

- What decides the outcome is whether a variant takes protected capacity away from ex-residents. v001 handed protected slots to probation, and v002 restored MRU insertion in some buckets and phases. Both weakened the victim cache for pre-credited ex-residents, and both lost or broke even. v003 left that region alone and promoted only keys with evidence that they return, and it was the one that won.
- On this trace a protected slot holding an ex-resident is worth at least as much as an extra chance for a fresh key. The stash split is close to zero-sum, so a fresh key should get past probation only on a selective, evidence-based signal, not a blanket one.
- Cycle 0004 concluded that insert position is the lever and counter credit is not. This cycle confirms it a second time: ghosts failed when they changed the counter (0004) and worked once they changed placement and covered a window about the size of the cache (0005).
- BIP's gain does not appear to depend on loop phases on this trace. Adaptive fallback to MRU cost more than it saved. However, the duel's miss signal was noisy and its leader groups were small, so this result does not fully rule DIP out.
- The gains are getting small. The ghost win suggests that most of the remaining misses are true one-touch keys or keys reused at distances beyond one cache's worth of evictions.
- For the third cycle running, no variant writer could inspect the overlay source, and every measurement again came from a single LIRS trace and a single size, covering hit-rate only. The metadata the ghost table adds is not captured by any fitness dimension.

## Open questions

- Would a larger ghost window, or a ghost hit that also protects the key from the next scan pass, recover more of the long-reuse misses?
- Is the ghost table's metadata and per-op cost justified by its small hit-rate gain? Neither has been measured.
- Is tuning `BIP_PERIOD` worth trying, given that it is the only fresh-key survival knob that takes no capacity from the protected region?
- Do the BIP and ghost gains hold on other traces and cache sizes?

## Continuation

Build on v003 (ghost-guided MRU placement). Probe the ghost window size and whether a ghost hit should also give scan protection, and optionally tune `BIP_PERIOD`. Drop wider probation and set-dueling DIP, along with any other change that takes capacity from the protected ex-resident region. Gains on this single trace are now small. Next cycle should put measuring metadata-size and per-op-time first (the ghost table makes this urgent) and evaluate on at least one more trace or cache size.
