## Recurring wins

- **Stash ordering by proven reuse** is the most reliable hit-rate lever seen so far (cycles 1–2). Demotees go to the stash MRU/Back end and fresh puts go to the LRU/Front end. The two gains add up, so the stash works as a scan-resistant probationary queue.
- **Using idle capacity beats admission policy** (cycle 4). Retaining items that would otherwise be dropped, by placing them in holes elsewhere (a neighbour stash bucket, or free target/probing slots), gave the first hit-rate gains after cycle 2. It adds no metadata.
- **State-neutral changes compose cleanly.** Lookup-order changes, stash insert positions and bookkeeping refactors reproduced the expected cache state exactly when combined.
- **Metadata-size has not moved in any variant so far.** Every lever tried derives placement from the hash or from bucket fullness, so it adds no extra state.
- **Latency gains come from doing fewer promotion moves or cheaper bucket scans** (cycles 1, 3). Skipping the remove → promote → demote → stash-insert chain reliably shortens the get tail.

## Recurring failures

- **Admission filters on either promotion edge.** MRU placement on stash→target (cycle 2), a second-hit gate (cycle 3) and a native-ownership gate on target→probing (cycle 3) all lost hit-rate. With tiny buckets, items can't stay resident long enough for an extra wait to pay off. Free signals such as slot position or ownership don't track recency well enough.
- **Placement tweaks that only fire in rare corner cases**, such as balanced placement on stash hits (cycle 1). They are safe but have no measurable effect.
- **Pure bookkeeping trims** (less redundant hashing or re-scanning) don't move get p50 (cycle 4). The common non-promoting lookup path sets the median.
- **Fixed lookup-order changes carried across lineages.** Target-first won in cycle 1 but lost after fresh-at-stash-Front placement made stash hits common (cycle 3). The best order depends on the placement policy.
- **Per-op-time claims without a code-level cause.** Several cycles show latency shifts on code paths the diff never touched, including large shifts across the whole cohort. Treat these as environment noise.

## Open questions

- Do two-choice stash and victim rescue stack? Both touch stash insert. Their hit-rate gains may overlap while their latency costs add up.
- Can the second stash scan on misses be filtered or short-circuited without losing two-choice retention? Should target/probing-first order be re-tested under two-choice?
- Can the promotion chain be made cheaper without changing which items get promoted, e.g. in-place swaps or a cap on how far it goes?
- Would per-slot fingerprints cut scan cost enough to justify the metadata-size cost?
- Would an independent second hash for the alternate stash bucket retain more than a fixed neighbour stride?
- What do stash capacity and associativity do? Larger stash buckets might finally let filters work.
- How do hits split between the stash and the normal tiers? This is still unmeasured, yet it drives both lookup-order and placement outcomes.

## Per-policy notes

- **Stash tier:** this is where most hit-rate leverage sits. Its insert-position policy (fresh at Front, demotees at Back) is now the settled base. Adding a second stash choice raises retention but adds a scan to every miss.
- **Stash→target promotion:** keep plain LRU-style promotion that places at the Front. Hotter placement and stricter gating both hurt. The stash-hit promotion path is now the slowest common hit path, and its cost grows as hit-rate rises.
- **Target→probing promotion:** gating it by ownership lost reuse. Leave admission as is and focus on what each move costs.
- **Normal (target/probing) buckets:** they have spare holes after promotions, and rescuing stash victims into them helps hit-rate. Fuller buckets make every lookup scan more.
- **General trade-off:** from cycle 4 on, gains in retention trade directly against get latency. Judge every hit-rate gain against its per-op-time cost, measured against a baseline taken in the same run.
