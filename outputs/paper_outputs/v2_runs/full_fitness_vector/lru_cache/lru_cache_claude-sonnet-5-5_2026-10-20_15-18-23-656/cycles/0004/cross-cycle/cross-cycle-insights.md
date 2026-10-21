## Recurring wins

- Policy and mechanics are independent axes. Hit-rate comes only from the policy (segmentation, reference bit). Speed and size come from mechanics (hasher, clone removal, link layout). Mechanics changes that leave eviction order alone have preserved hit-rate in every cycle.
- Trimming hashing and eviction-path work pays off most on put: removing the eviction key clone, a lighter hasher, and dropping the avalanche fold all helped put.
- Segmentation reliably gives scan-resistance hit-rate. This is established.
- Combining independent Winners (segmentation, fast hasher, cheap touch, move-based eviction) worked as a merge strategy.

## Recurring failures

- Shifting promotion/demotion work between operations does not remove it. Lazy demotion (put), reference-bit get (eviction walk) and similar moves only relocate the cost, usually onto the put tail.
- Layout tweaks on the segmented policy (u32 links, packing the segment bit) bought size at best, not speed. They add widening, NIL checks and masking on every link access.
- Partial-coverage promotion skips (head-adjacent relink skip) add a branch and save little, because few hits land there.
- Bundling several changes (fold with layout, hasher with touch path and clone removal) blurs attribution. Authors cannot read diffs, so causes are inferred.
- Infrastructure failures (a build rate limit in cycle 1) leave ideas untested. Do not read them as negative results.

## Open questions

- Do the fold-free hasher and the reference-bit get path compose, and do the gains add up?
- Can the eviction walk be bounded or amortized (capping the referenced nodes passed per put) to fix put p99 without losing hit-rate?
- Can the segment be stored implicitly (by list membership) so there is no masking?
- Why did put slow in a variant that did not target it, and did the unexplained struct shrink hold up?
- Does segmentation's hit-rate gain justify its latency cost on traces other than the scan-heavy one? No cycle has checked yet.
- Is the u32 link speed benefit real at a larger working-set size?

## Per-policy notes

- Plain LRU: u32 links, a lighter hasher and a head-touch early return were pure wins, with smaller size and faster ops, and no hit-rate change.
- Segmented LRU: gained hit-rate on scans, but its promotion/demotion relinking is the standing latency cost, mostly on the put tail. Its container also grows from the extra head/tail pair. The merged form (u32 links, Fx hasher, move-based eviction) is the working baseline, and the fold should stay removed.
- Second-chance (CLOCK-style) reference bit: a small extra scan-resistance gain and a cheaper get, with the cost moved to the eviction walk and the put tail.
