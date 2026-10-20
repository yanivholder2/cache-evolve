# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T11:42:51+00:00

## timeline

### 11:42:56 thinking



### 11:43:01 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Fingerprint-keyed ghost queue has given a put-path benefit with no hit-rate change in three cycles (0003, 0004, 0005). Its cost is container-level bookkeeping, not per-entry size, and its ghost-hit path scans linearly.\n- Id-indexed lookup and index consolidation reduced container state and put cost while leaving eviction, ghost, and promotion logic untouched (0005, 0006). Hit rate stayed flat in both. These are the cleanest structural gains so far, though the get-path speedup claim still needs a baseline rerun.\n- Edits that preserve policy have produced unchanged hit rate, which is useful evidence that the change was structural and not a policy shift.\n\n## Recurring failures\n\n- Enlarging the ghost queue toward full cache capacity lost hit rate (0004). It re-admitted more keys to main without a matching reuse gain.\n- Stacking a lowest-frequency victim choice on the larger small queue lost hit rate (0003). The rotation budget likely runs out less often once the small queue is larger, so the victim change has little to act on.\n- Ghost sequence-tagging added put-side bookkeeping without a policy gain (0002).\n- Splitting the per-entry frequency counter into a side array grew the whole structure and added an indexed read on hits (0004). Shrinking `Entry` did not shrink the footprint.\n- Lowering the small-queue promotion threshold to `freq > 0` moved singly-reused keys into main early, displaced useful residents, and showed no attributable latency gain (0001).\n- Get-path timing moved in variants whose get code was unchanged (0001 through 0006). Treat these shifts as shared noise or environment drift, not as variant effects.\n\n## Open questions\n\n- Does the fingerprint ghost's put-latency gain hold across other traces and capacities, and does its hash-set and linear ghost-hit cost stay acceptable at larger sizes?\n- Can the half-main ghost cap's per-eviction drain be amortized (for example, batch compaction) so its small hit-rate gain survives without the put cost? Does it still help once the ghost stores fingerprints?\n- Does the small-queue share increase help on its own? Its hit-rate effect has only appeared bundled with the victim change, and standalone runs showed no movement.\n- How often does the second-chance budget run out under the evaluated workloads, and does the victim choice help in that regime?\n- Is the get-path shift real and reproducible under a baseline rerun with no code change?\n- Do the index and ghost edits compose? None has been tested in combination.\n\n## Per-policy notes\n\n- Small queue: The promotion threshold change (`freq > 0`) was a hit-rate loss with no attributable gain; drop further tuning there. The share increase alone showed no hit-rate movement and only modest, single-run latency movement, so it lacks a Winner-grade case.\n- Ghost queue: Fingerprint storage is the most consistent put-side win. Enlargement toward full capacity hurt hit rate. A half-main cap gained a small amount but adds put cost through its drain. Sequence-tagging added cost without benefit. Position-based removal does not appear to be the dominant put cost.\n- Main queue: The post-budget lowest-frequency victim choice lost hit rate when stacked on the larger small queue, and its recovery benefit is untested in a regime where the budget runs out often. Frequency aging on rotation left hit rate unchanged on the one trace reported and has not yet been shown to change any eviction.\n- Lookup and metadata (cross-policy): Index consolidation and id-keyed lookup are the strongest structural gains. Per-entry packing changes have not paid off; the cost is in container-level state.\n- Caveat: Several mechanism claims rest on hypothesis text and fitness deltas because overlay source was not inspected, so attribution remains provisional.\n"
}
```

### 11:43:05 chat

I wrote `cross-cycle-insights.md` to the cycle 0006 cross-cycle output directory, using the four required sections in order. I did not read the previous version of the file.

The main signals across cycles 0001–0006:

- **Recurring wins:** The fingerprint ghost queue has shown a put-path benefit with no hit-rate change in three cycles (0003–0005). Id-indexed lookup and index consolidation cut container state and put cost without touching policy logic (0005–0006). Their get-path speedup still needs a baseline rerun.
- **Recurring failures:** Enlarging the ghost toward full capacity lost hit rate (0004). Stacking the victim choice on the larger small queue also lost hit rate (0003). The `freq > 0` promotion change had no attributable gain and cost hit rate (0001). Get-path timing shifts appear in variants whose get code is unchanged, so I treated them as noise.
- **Open questions:** Whether the fingerprint ghost's gain holds across traces and capacities, whether the half-main ghost cap's drain can be amortized, whether the small-queue share helps on its own, and whether the index edits compose.

The file notes that several mechanism claims are provisional because the overlay source could not be inspected. I kept the notes qualitative and did not add any numbers beyond those in the ledger digest.


---

- finished: 2026-10-20T11:43:05+00:00
- duration: 13.89s
- status: ok
- input_tokens: 4
- output_tokens: 2468
- cache_read_tokens: 38455
- cache_write_tokens: 11026
- cost_usd: 0.003824
