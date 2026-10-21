## Recurring wins

- Mechanics changes (u32 links, lighter hasher, head-touch early return, move-based eviction instead of key clone) preserve eviction order and hit-rate. Speed and size gains come from mechanics. Hit-rate comes only from policy. The two axes stay independent.
- Segmentation gives scan resistance on the scan-heavy trace. This is established and needs no re-proving.
- Removing work from the hot path (the hash fold, the key clone) paid off. Shuffling work between operations did not.
- Combining independent wins into one baseline works, but later combinations compose only weakly, with gains near noise.

## Recurring failures

- Moving promotion/demotion relink work between `get`, `put` and eviction (lazy demotion, reference bit) only shifts the cost. One operation or tail gets worse as another improves.
- Cheap-skip heuristics (head-adjacent relink skip) failed. Few hits land in the skipped zone, so the extra branch costs more than it saves.
- Link-width and bit-packing changes on the segmented policy gave size savings but no speed. Widening, masking and NIL checks cost latency.
- Bundling several changes (hash fold with layout, hasher with touch path) blurs attribution, because authors cannot read diffs.
- Constant tuning and small hasher tweaks fall within the per-op-time noise floor and should not be credited as wins.
- Infrastructure failures (build rate limit) can wrongly reject an idea. Check a failure's cause before concluding the idea is bad.

## Open questions

- What is the per-op-time noise floor? Which recent speed claims fall below it?
- Can the eviction cap be made adaptive, or the choice of spared nodes cheaper, so put p99 stays low and hit-rate recovers?
- Would a smaller protected share improve hit-rate beyond the current split?
- Can the segment be stored implicitly, by list membership, so no masking is needed?
- Does the hit-rate behavior hold on workloads other than the scan-heavy trace? This has been pending for many cycles.
- Does the struct-size shrink seen when the fold was removed hold, and what caused it?

## Per-policy notes

- Plain LRU: u32 links and the fast hasher were pure wins in cycle 0002. This is the clearest case where mechanics gave both speed and size.
- Segmented LRU: hit-rate gain on scans, paid for by promotion/demotion relinking on the hot path and larger container metadata. A fold-free single-multiply-plus-rotate hash is enough. Protected share is a weak lever, and a larger share slightly hurt hit-rate.
- Second-chance reference bit (CLOCK-style): get is slightly cheaper and scan resistance rises a little. The cost moves to the eviction walk and the put tail. Capping the walk cuts latency but lowers hit-rate, so the cap is a direct latency/hit-rate dial.
- Current baseline: segmented, u32 links, fold-free hasher, reference bit, move-based eviction.
