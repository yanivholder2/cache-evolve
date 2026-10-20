# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-20T11:39:26+00:00

## timeline

### 11:39:32 thinking



### 11:39:38 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Fingerprint-keyed ghost queue: put-path benefit with no hit-rate change, seen in Cycles 0003, 0004, and 0005. Its cost is container-level bookkeeping, not per-entry size.\n- Put-path edits that remove per-eviction or per-put key handling have the clearest mechanism. Hit rate stays flat when eviction and promotion logic is left alone.\n- Whole-structure footprint is the right unit for metadata changes. Per-entry size is a poor proxy, because several changes shrank the entry or left it alone and still grew the container.\n- Ghost population has a sweet spot below full capacity. Capping it below main-queue size (Cycle 0005) gave a small gain, while enlarging it to full capacity (Cycle 0004) lost hit rate.\n\n## Recurring failures\n\n- Enlarging the ghost toward full cache capacity: re-admits more keys to main without a matching reuse gain, and hit rate falls (Cycle 0004).\n- Lowering the small-queue promotion threshold (`freq > 0`): moves singly-reused keys into main early, displaces useful residents, and costs hit rate with no attributable latency gain (Cycle 0001).\n- Stacking the lowest-frequency victim choice on the enlarged small queue: lost hit rate and did not compose with the larger share (Cycles 0002, 0003).\n- Side-array split of the frequency counter: entry bytes shrink, but the container grows and each hit pays an extra indexed read. Net footprint is a loss (Cycle 0004).\n- Frequency aging on main-queue rotation: no hit-rate change and no attributable benefit on the one trace reported. Untested, not shown neutral (Cycle 0003).\n- Sequence-tagged ghost queue: preserved policy but added per-eviction bookkeeping on the put miss path (Cycle 0002).\n\n## Open questions\n\n- Does the fingerprint ghost's put-latency gain hold across traces and capacities, and does its hash-set cost and linear ghost-hit scan matter in tail latency at larger sizes?\n- Does the half-main ghost cap keep its small gain on other workloads, and does it still help once the ghost is stored as fingerprints? Is there a useful ghost size between half-main and full capacity?\n- Do the small-queue share increase and the post-budget victim choice hold their small hit-rate gains across other traces and capacities? Are they in conflict, as the stacked result suggests?\n- Does the id-indexed lookup keep its get and put speedup under a baseline rerun, and does the extra index state matter at larger sizes?\n- Is the get-path latency shift real or shared measurement noise? It appears in variants whose get code is unchanged, and it is unexplained in most cases.\n- Where is the dominant put miss-path cost, if the ghost position scan and per-eviction key handling are not it? Does the O(n) ghost pop-back still dominate once membership is O(1)?\n- How often does the second-chance rotation budget run out under the evaluated workloads? Frequency aging and victim choice both depend on that.\n\n## Per-policy notes\n\n- Small queue: the promotion threshold (`freq > 0`) is a hit-rate loss. The size increase shows a small gain on one trace, and it does not combine with the victim choice. Validate before keeping either.\n- Main queue: the rotation budget and victim choice are coupled. The budget's exhaustion frequency is unmeasured. Frequency aging has no measured effect yet.\n- Ghost queue: fingerprint storage is the most consistent win on the put side. Enlargement toward full capacity is a failure. A capped population is a possible small gain. Sequence tagging and position-based removal have not shown a dominant cost.\n- Metadata layout and lookup: id-indexed hash lookup removes per-put and per-hit scans with no hit-rate change, but its get-path gain still needs a baseline rerun. Side-array counter storage is a footprint loss.\n- Cross-cutting: changes that touch overlapping state (ghost storage, lookup structures) have not been tested in combination. Their gains may not add.\n"
}
```

### 11:39:41 chat

I wrote `cross-cycle-insights.md` to the cycle 0005 cross-cycle output directory, with the four required sections in order. I built it only from the inlined ledger digest and did not read the previous file.

The main signals across cycles 0001–0005:

- **Recurring win:** the fingerprint-keyed ghost queue has shown a put-path benefit with no hit-rate change in three consecutive cycles (0003, 0004, 0005).
- **Recurring failures:** enlarging the ghost toward full capacity, lowering the promotion threshold to `freq > 0`, stacking the victim choice on the enlarged small queue, and the side-array counter split (a net footprint loss).
- **Ghost size:** a capped ghost population (half of main) gave a small gain on one trace, while full-capacity enlargement lost hit rate.
- **Open threads:** whether the fingerprint ghost's gain holds across traces and capacities, whether the get-path latency shift is real or noise, and whether the id-indexed lookup's get speedup survives a baseline rerun.

The per-policy notes group these by small queue, main queue, ghost queue, and metadata layout. Several claims rest on single traces or on hypothesis text, since the overlay source was not inspectable, and the file says so where it matters.


---

- finished: 2026-10-20T11:39:41+00:00
- duration: 15.16s
- status: ok
- input_tokens: 4
- output_tokens: 2780
- cache_read_tokens: 37674
- cache_write_tokens: 10574
- cost_usd: 0.003882
