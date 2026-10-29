## Recurring wins

- Compacting node metadata is the most dependable lever. The `u32`-index layout (c1) became the base for every later cycle and costs nothing on any other axis.
- Packing policy state into bits the node already has works. A segment flag in a link's high bit and a counter in the padding byte (c2) both kept the node compact. Any new field that misses existing spare bits costs a full word, because the node has no free padding (c1).
- Changes that only touch representation move one axis cleanly and leave hit rate exactly unchanged (c1, c3 v002). Policy changes move several axes at once, often in opposite directions.
- SLRU is the strongest hit-rate policy so far. Once it was flag-packed onto `u32` nodes (c2) it kept its gains on every trace at both sizes, and its only remaining cost is on put.

## Recurring failures

- Hash-map speed levers: a cheaper hasher left get p50 flat (c1) and then made it worse (c2). The probe looks bound by memory and sensitive to how well the hash spreads structured block keys. This is inferred; probe lengths have not been measured.
- List and relink micro-optimisation as a get-latency lever. Removing relinks (c1), compacting nodes (c1) and specialising relinks (c3) all left get p50 within noise. List maintenance does not dominate the cost of a hit.
- Side structures kept per cache (ghost `HashSet`, SipHash sketch, c3) made get p50 worse even though get never reads them, probably through CPU-cache pressure. Each one also adds about one hash operation per probe to put.
- History-based admission in its crude forms fails on web_0. The eager form (a full-key ghost that promotes straight to protected and forces demotion) and the strict form (a miss-only, tie-rejecting gate) both hurt it, even though hm_0 gained slightly under each (c3).
- Policies that buy hit rate keep paying for it on put: SLRU's double relink, GCLOCK's sweep multiplied by decrement-by-one aging (c2), and the hashing added by ghosts and sketches (c3). The put path is bound by hashing and memory, not by list logic.

## Open questions

- web_0 at the large cache size has not improved in any cycle, whether through recency segmentation, small counters, a ghost or a sketch. It reacts to how aggressive admission is in both directions. Candidates: a tiny fingerprint ghost with gentle admission, W-TinyLFU (counts hits too, small window, cheap hash), or an adaptive probation share.
- SLRU put cost: avoid the second relink on promotion, add a fast path for updating keys that already exist, and avoid cloning the key on insert.
- get latency: table density, hash quality (a better-mixing finaliser), and map/arena locality are all untested. Speed of the hash itself is ruled out.
- Benchmark attribution: the GCLOCK get slowdown (c2) and the side-structure get regressions (c3) were never separated from noise caused by interleaving puts and gets. An isolated get benchmark would settle several open threads.

## Per-policy notes

- LRU (baseline): its node layout has no free padding. `u32` indices are a free improvement whatever the policy.
- SLRU: the default base since c2 (flag-packed, `u32`). It is weak at the small cache size, where the probation share is untuned. Adding a ghost or a TinyLFU gate in front of it has not beaten the plain version.
- CLOCK: lazy promotion gave a small metadata win, and get barely moved. Its single bit helped hm_0 at both sizes but was mixed on web_0. It has put tail latency from sweeps, and that is still unbounded.
- GCLOCK: its hit-rate gain was concentrated in hm_0 at the large size, and decrement-by-one aging inflates sweeps. It is only worth revisiting if aging moves off the eviction path (for example bulk halving).
- TinyLFU / ghost admission: only crude versions have been tried. Any retry has to be tiny, cache-friendly and gentle at admission.
