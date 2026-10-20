## Recurring wins

- Policy changes drive hit-rate. Layout and mechanics changes drive speed and size. The two axes stay largely independent. Segmentation gave scan resistance, and that is established.
- Mechanics changes on the plain LRU were pure wins: u32 links, a lighter hasher, a head-touch early return and move-based eviction (no key clone).
- Removing unneeded hash work paid off. The fold-free hasher helped put and shrank the struct on the segmented cache.
- Reference-bit (second-chance) get path: cheaper get and a small scan-resistance gain.
- A larger protected share gave small, diminishing hit-rate gains.

## Recurring failures

- Shifting promotion/demotion work between operations (lazy demotion, relink-free get) only moves the cost: get gets better while put or the put tail gets worse.
- Packing the segment bit, or narrowing links, on a segmented policy saves size but not speed. Masking and NIL checks cost latency, and the extra head/tail pair grows the container.
- Skipping relinks only for head-adjacent nodes added a branch that few hits could repay.
- Constant-only and position tweaks (protected-share steps, demote position, `put`-update promotion) are exhausted or within noise.
- Bundling changes (hash fold plus layout) made attribution impossible. Authors cannot read diffs, so attribution is always inferred.
- Small per-op-time deltas are noise, as shown by constant-only edits that moved latency. Winner status at that scale does not mean improvement.
- Infrastructure failures (a Maven 429 in cycle 1) can hide a hypothesis without testing it.

## Open questions

- What is the per-op-time noise floor? Would repeat-run measurement separate real deltas from noise?
- Can the eviction-walk cap be made adaptive, or the choice of which referenced nodes to spare made cheaper, so that put p99 stays low and hit-rate recovers?
- Is there any hit-rate lever beyond segment-boundary policy, such as an admission filter, ghost-entry history or sizing?
- Does a smaller protected share help, or has the trend saturated?
- Can the segment be stored implicitly by list membership, with no masking?
- Do the hit-rate and latency findings hold on workloads other than the scan-heavy trace? This has been pending since cycle 1.

## Per-policy notes

- Plain LRU: mechanics changes (u32 links, light hasher, touch fast path, clone removal) compose and give speed and size with no hit-rate change.
- Segmented LRU: hit-rate gain over plain LRU on scans. The remaining cost is promotion/demotion relinking on the hot path, and no work-shifting trick has removed it. The current baseline is the cycle-0006 larger-protected-share variant, with the fold-free hasher and the reference bit.
- Second-chance (CLOCK-like) eviction: the cap on the walk trades put latency against hit-rate directly. Treat it as a dial.
- Next candidates: admission filter or ghost history. Evaluate on non-scan traces.
