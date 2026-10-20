# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T11:51:41+00:00

## timeline

### 11:51:47 thinking



### 11:51:54 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-cycle insights\n\n## Recurring wins\n\n- Id-keyed indexing in place of linear resident scans (Cycles 0005–0008). It is the most consistently pursued direction, leaves eviction, ghost, and promotion logic untouched, and shows no hit-rate change. Get and put speedups are plausible but still need a baseline rerun before credit.\n- Merging per-queue indexes into one id-keyed map (Cycle 0006) reduced container state and put cost with no hit-rate change. This is the cleanest structural gain in the series.\n- Fingerprint-keyed ghost storage (Cycles 0003, 0004, 0005) gave a put-side benefit with no hit-rate change in each cycle it was tested. Its cost is container-level bookkeeping, not per-entry size.\n- Whole-structure footprint is the right unit of measurement. Every Winner changed the container; none produced a measurable saving in `Entry` alone.\n- Batched, off-path ghost expiry (Cycle 0007) is the first edit to address ghost drain cost directly. Its hit-rate gain is small and rests on one trace.\n\n## Recurring failures\n\n- Lowering the frequency ceiling below 3 (Cycle 0008) cost hit rate on the workload tested and saved no metadata. The third level appears to earn its keep.\n- Enlarging the ghost toward full cache capacity (Cycle 0004) admitted more keys to main without a matching reuse gain and lost hit rate. Ghost size is not a free dial.\n- Moving singly-reused keys into main earlier (promotion threshold `freq > 0`, Cycle 0001) gave no attributable gain and cost hit rate.\n- Stacking a lowest-frequency victim choice on the larger small queue (Cycle 0003) lost hit rate. The victim change has little to act on once the small queue is larger.\n- Splitting the frequency counter into a side array (Cycle 0004) grew the container, a net footprint loss. Shrinking `Entry` alone did not shrink the structure, and it added a get-path read per hit.\n- Swapping the index hasher for a multiply-shift hash (Cycle 0008) shrank the container but made gets slower. Hashing does not appear to be the lever on the hit path.\n- Tagging ghost entries with sequence numbers (Cycle 0002) added put-path bookkeeping with no policy gain.\n\n## Open questions\n\n- Does the fingerprint ghost's put-latency gain hold across other traces and capacities? Does its hash-set cost and linear ghost-hit scan matter at larger sizes?\n- Does the id-keyed index keep its get and put speedups under a baseline rerun and on other traces? Does its extra index state matter at scale?\n- Is the get-path timing shift seen across many variants real, or shared environment noise? It has appeared in variants with unchanged get code in most cycles.\n- Does the half-main ghost cap keep its small hit-rate gain on other workloads, and does it still help once the ghost is stored as fingerprints? Can its drain be amortized so the put cost goes away?\n- Does the batched ghost window's hit-rate gain hold on other traces, and is the window close to a strict cap?\n- Do the index, the counter relocation, and the ghost window compose, given they touch overlapping state? None has been tested in combination.\n- Is the frequency ceiling of 3 specific to the workload where the loss was seen?\n- What is the dominant put miss-path cost, if it is not the ghost position scan?\n- How often does the second-chance rotation budget run out under the evaluated workloads, and does frequency aging ever change an eviction?\n\n## Per-policy notes\n\n- Small queue: enlarging its share alone left hit rate unchanged on the one workload reported. Earlier hit-rate gains appeared only when paired with the victim change, so the share change itself is unproven. Promotion threshold changes (`freq > 0`) showed no gain and should not be retried on current evidence.\n- Main queue: the post-budget lowest-frequency victim choice showed a hit-rate effect only with the smaller small queue and did not compose with the larger one. Frequency aging left hit rate unchanged on one trace, so it is untested rather than neutral. Frequency ceiling should stay at 3.\n- Ghost queue: fingerprint storage is the one ghost edit with repeated put-side benefit. Enlarging the ghost to full capacity lost hit rate. Capping it at half the main queue gained slightly, but only on one trace and only in combination with fingerprints. Batched expiry addresses drain cost, though its gain is also single-trace. Sequence tagging added cost without benefit.\n- Metadata and indexing: id-keyed lookup is the lead direction across cycles. Counter relocation and index merging changed container shape without hit-rate change. Hasher choice did not help gets. Footprint should be measured at the structure level.\n"
}
```

### 11:51:56 chat

I wrote `cross-cycle-insights.md` to the output directory, built only from the inlined ledger digest for Cycles 0001–0008. I did not read the previous version of the file.

The file has the four required sections in order: Recurring wins, Recurring failures, Open questions, and Per-policy notes. The main signals:

- **Id-keyed indexing** is the most consistently pursued direction across Cycles 0005–0008. Its get and put speedups still need a baseline rerun before they are credited.
- **Fingerprint-keyed ghost storage** showed a put-side benefit with no hit-rate change in Cycles 0003, 0004 and 0005.
- **Recurring failures** include lowering the frequency ceiling below 3, enlarging the ghost toward full capacity, and stacking the victim choice on the larger small queue.
- **Open questions** centre on whether the get-path timing shift is real, whether the fingerprint ghost's gain holds on other traces, and whether the index, counter and ghost-window changes compose.

No numbers were added beyond what the digest states.


---

- finished: 2026-10-20T11:51:56+00:00
- duration: 15.70s
- status: ok
- input_tokens: 4
- output_tokens: 3221
- cache_read_tokens: 40229
- cache_write_tokens: 13643
- cost_usd: 0.004742
