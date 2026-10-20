# Cross-Cycle Insights (through cycle 0009)

## Recurring wins

- **Using idle capacity is the only hit-rate lever that has kept working (cycles 2, 4–8).** Gains came from stash order that ranks proven reuse (demotees at stash Back, fresh puts at stash Front), the two-choice neighbour stash, and rescuing stash victims into holes. None of these added metadata. Rescue that writes only into target holes keeps almost all of full rescue's gain.
- **State-neutral, lossless filters reliably cut get latency (cycles 6, 7, 9).** Examples are the exact per-segment `borrowed`/`foreign` counters that gate the `s1` and probing scans, and the inline one-byte key tags. Hit-rate stayed exactly the same each time. The gates stack, so a full miss can skip both neighbour probes.
- **Gates are cheapest when mutations already know the item's class (cycle 7).** Counters fed by slot type or a hash the caller already computed cost nothing on put. A counter that has to rehash shows up in the put tail.
- **Avoiding the dependent load into each bucket's heap item buffer finally moved get p50 (cycle 9).** Tags beat flattening because they skip the load on missing probes, which are most probes, and they leave mutation paths alone. They also help put, through the "already present?" check.
- **Single-hash plumbing along the stash-hit/demotion chain trims the get tail at no cost (cycle 8).** The chain is a tail event, so the payoff is at p99 only.
- **Independent changes compose cleanly and predictably.** Examples are lookup order with insert position, two-choice with rescue, and the two gates together. When a refactor claims to be state-neutral, an exact hit-rate match has been a dependable check.

## Recurring failures

- **Admission filters on either promotion edge (cycles 2–3).** Tried: MRU placement on stash→target, a second-hit gate, and a native-ownership gate. Every one blocked real reuse. The tiny buckets can't hold items long enough for an extra wait to pay off.
- **Fixed lookup-order tweaks as a latency lever (cycles 1, 3, 5, 6).** Which order wins flips with placement policy. Once `s1` is last, the remaining cost comes from cache state (fuller buckets), not from probe order.
- **Removing hashes or dispatch to move get p50 (cycles 4, 7, 8).** Bookkeeping trims, devirtualising at bucket level, and concrete buckets that still point to a heap `Vec` all left the median flat. Several also made the tails worse.
- **Recency ordered by position in inline/flat slot layouts (cycle 9).** Every insert, remove or reorder turns into shifting, which raised put p50 and p99.
- **Low-yield residency sources.** Eagerly filling target holes with fresh items fills the tier with cold items and moves the cost onto later promotions (cycle 5). Probing holes at native-demotion time almost never exist (cycles 6, 8). An independent-hash `s1` adds little hit-rate for the cost of a full extra hash (cycle 7).
- **Trusting the put-p99 baseline.** It has swung in several cycles (7–9) without a code-level cause. Per-op-time claims without a mechanism have repeatedly turned out to be noise or environment drift (cycle 1 onward).

## Open questions

- Right-sized tags with a smaller `max_size` (a `u8`) and a smaller header stride: how much more get latency comes back?
- Do tags stack with flat/inline slot storage if recency doesn't move items (age/clock bytes or an index permutation)?
- With cheaper probes, is target-only rescue's per-resident scan cost now low enough to justify its hit-rate? Can rescue be throttled by victim reuse, or set to give way first?
- Stash geometry (wider, fewer stash buckets / associativity) is still untested. Cycle 9's attempt produced no result. Could larger stash buckets make a second-hit filter viable?
- Can `borrowed` be maintained entirely from flags the caller already knows, so it never rehashes? Does `foreign` stay near zero once rescue is stacked on?
- Re-measure the put-p99 baseline in the same run before crediting any write-path change.

## Per-policy notes

- **Stash (probationary LRU tier):** Main capacity and hit-rate lever. Scan-resistant ordering works: fresh items at Front, demotees at Back. Two-choice overflow helps, but overflow barely clusters, so extra choices add little. Probe `s1` last and gate it with `borrowed`.
- **Target buckets:** Tiny, so promotion placement at MRU and admission gates both hurt. Holes here are the only rescue source worth using. Fuller target buckets lengthen scans and add promotion cascades, so this is the core hit-rate vs get-latency trade.
- **Probing buckets:** Keep them clean of rescue so the `foreign` gate stays effective. Native-demotion-into-probing-hole doesn't pay. Gating with `foreign` is free and effective.
- **Bucket eviction policy implementation:** LRU ordered by position is fine behind a heap `Vec`, but costly when stored inline. Any move to inline storage needs recency metadata that doesn't move items.
