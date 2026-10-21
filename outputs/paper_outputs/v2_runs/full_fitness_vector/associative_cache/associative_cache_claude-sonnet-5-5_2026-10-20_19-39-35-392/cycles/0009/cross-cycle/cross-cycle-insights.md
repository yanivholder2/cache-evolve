## Recurring wins

- Scan resistance via insertion policy is what moves hit rate. Capacity and associativity were never the limit (cycles 1–2).
- Mid-bucket (about len/4) insertion plus gradual transpose-by-one promotion beats plain LRU. It has been reproduced from baseline three or more times (cycles 3, 6, 8). It is stateless and adds no metadata.
- Cutting put-path work (no half-bucket shift, hash once) lowers put latency without hurting hit rate (cycle 4). Approximate order among displaced entries doesn't matter. Where the new entry lands does.
- The only adaptive gain with a real mechanism came from a global hit/miss counter that switches insertion depth (cycle 8). It gave better hit rate and cheaper puts, and it fit in existing padding.
- Metadata has never grown. The 40-byte budget has never been the binding constraint so far.

## Recurring failures

- Get latency has had no real gain in nine cycles. Item movement, hashing, arithmetic, layout (tags, alignment) and extra per-hit logic have all failed. Every per-hit addition (flag, counter, compare, swap) taxed get.
- Retuning the insertion constant (len/3, len/5) and other position-only tweaks are saturated. len/4 is a local optimum.
- Stacking scan-resistance mechanisms (reference bit plus low insertion, a second reuse gate on promotion) doesn't compound. It costs both hit rate and time.
- Faster or position-gated promotion (geometric jumps, two-slot jumps) removes the frequency filter that transpose-by-one provides. Hit rate drops or gains stay marginal.
- Stateless randomization (hash-parity victim, hashed MRU escape) carries no reuse signal and only adds branches.
- Weak signals fail: one-entry ghost filters, victim-position proxies for adaptivity, and per-slot frequency counters.
- Micro-fusing the put scan, restructuring put-hit control flow and alignment-only tweaks give noise, not wins. The apparent put-tail gains didn't reproduce.
- Winner status is repeatedly decoupled from a variant's own evidence. Seven straight cycles include Winners whose insights report losses. Put-latency gains with no mechanism have been noise every time.

## Open questions

- Which part of the cycle 8 adaptive gain is the global hit/miss signal? Can it be updated cheaply, off the get path or with amortized sampling?
- Can swap insertion keep its put saving without demoting the len/4 occupant to the tail (for example, shift a short span instead)?
- Where does get time go? Harness overhead, dispatch, and the key/value path are untested.
- Is the Winner label meaningful at all? A noise check and repeated runs are needed.
- Do any results hold on a second workload, or are they specific to workload-a's scan-heavy traffic?
- Can a real reuse signal fit under the metadata budget and pay for its cost?

## Per-policy notes

- LRU baseline: wider buckets and partial promotion don't help. Rotate versus remove+push has no effect on get.
- CLOCK reference bit: it helps alone by making put and the get tail cheaper. It loses when stacked on low insertion.
- Frequency counter: marginal hit gain, slows get and put. Redundant once scan resistance exists.
- LIP/BIP-style low insertion: good alone. Pushing all the way to the LRU end starves new entries.
- Mid insertion plus transpose: the settled baseline. Swap-based len/4 is cheap on put but loses some hit rate to the displaced entry.
- Adaptive insertion depth (len/4 versus len/2): the best lead when driven by a global hit/miss counter. Proxies based on victim position don't reproduce it.
- Ghost admission filter: a single-entry ghost is too weak. A deeper or more selective history is untested.
