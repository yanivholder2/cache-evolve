# Cycle 0010 Insight

## Cohort outcome

All three variants branched from cycle 0009's v001 (CLOCK with `u8` counters), and each tested a separate lever. v001 packed the sketch into 4-bit counters. v002 replaced the 1-bit CLOCK reference bit with a 2-bit GCLOCK counter. v003 replaced the `fmix64` finaliser with a single xor-fold. All three are listed as Winners, but only two are real steps forward. v003 is the first change to lower `get` p50, and hit rate stays within noise. v002 is the first eviction-order change to raise hit rate, and it pays for it in tail latency. v001 lost hit rate and latency together.

## Patterns

- The `get` median is limited by the serial ALU chain before the counter loads, not by memory. Removing hash-finaliser work helped. Shrinking the sketch (v001 here, and cycle 0009) or adding mix and prefetch steps (cycle 0009) did not. This sharpens cycle 0008's claim that "hashing is not the cost": the number of hash calls doesn't matter, but the latency of the finaliser on the sketch path does.
- Without aging, counter range is not spare. Counts only grow, so a cap of 15 makes hot residents and hot newcomers tie at saturation, and the strict `>` rule rejects the newcomer. The `u8` cap of 255 was fine. The lesson from cycle 0009 that admission is decided by low counts holds only when the cap sits far above the counts that steady state reaches.
- Hit rate moved when eviction order inside the main region changed, while the admission duel stayed fixed. Every admission-side lever has been flat or negative. Giving residents with repeat hits more lives changes which cold resident the duel defends, and that helped.
- Extra fixed per-access work still shows up on the hot path, as in cycles 0007 and 0009. Nibble shift and mask math (v001) and a second bit-plane RMW plus longer sweeps (v002) both cost tail latency. v003 removed work and was the only variant whose median fell.
- `put` p50 and the `get` p99 tail did not respond to any lever this cycle. `put` is limited by map remove/insert and the sweep.

## Open questions

- Do v002 and v003 compose? Hash folding should be independent of GCLOCK victim choice.
- Would interleaving the two GCLOCK bits per slot in one word remove v002's extra RMW, and with it the tail regression, while keeping the same victim choice?
- Does a 3-bit usage counter add more hit rate, or does the longer worst-case sweep eat the gain?

## Continuation

Use v003 (xor-fold hash) as the new base. Then port v002's GCLOCK onto it, with both usage bits interleaved in a single word per slot, and require the hit-rate gain to survive with tails no worse than v003's. Drop 4-bit counters unless they come with halving or aging, and drop sketch-footprint changes as a latency lever entirely. Don't trim the hash further, because the remaining `put` and tail costs are in the map operations and the sweep.
