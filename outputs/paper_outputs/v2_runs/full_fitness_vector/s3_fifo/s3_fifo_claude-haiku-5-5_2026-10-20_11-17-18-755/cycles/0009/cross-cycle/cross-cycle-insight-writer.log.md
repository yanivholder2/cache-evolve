# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T11:55:43+00:00

## timeline

### 11:55:52 thinking



### 11:55:58 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Id-keyed indexing in place of linear resident scans and position lookups has been the repeated direction across cycles 0005 through 0008. It leaves eviction and promotion untouched, so hit rate does not move. Speedups are still unverified and need a baseline rerun.\n- Fingerprint-stored ghost entries improved put-path cost with no hit-rate change in three cycles (0003, 0004, 0005). Its recurring cost is container-level bookkeeping (a hash set), not per-entry size.\n- Merging per-queue indexes into one id-keyed map (0006, 0007) reduced container state and put cost without changing policy.\n- Policy-preserving metadata and data-structure edits have consistently left hit rate unchanged. Hit-rate movement has come only from edits that changed eviction or ghost sizing.\n\n## Recurring failures\n\n- Lowering the main-queue frequency ceiling from 3 to 2 (0008) and periodic halving or aging of main-queue counters (0009) both cost hit rate. Cooling hot entries makes them evictable too early. The third frequency level and the existing second-chance rotation appear to earn their keep.\n- Enlarging the ghost toward full cache capacity (0004) raised re-admissions to main and lost hit rate.\n- Raising the promotion threshold to `freq > 0` (0001) lost hit rate and gave no attributable latency gain.\n- Stacking a lowest-frequency victim choice on the larger small queue (0003) lost hit rate. Neither the share change alone (0006) nor the victim choice alone produced a reproducible gain.\n- Splitting per-entry metadata into side arrays (0004) grew the whole structure, and it added an indexed read on the hit path. The footprint result was a net loss.\n- Swapping the index hasher for multiply-shift (0008) shrank the container but slowed gets. Hashing is not the lever on the hit path.\n- Flat linear-probing ghost table (0009) produced a put-tail regression with no hit-rate change.\n- Crediting get-path latency shifts to a variant whose get code is unchanged has repeatedly proved to be noise (0001 through 0009). Such shifts should not be credited without a baseline rerun.\n\n## Open questions\n\n- Is the get-path latency shift that appears across variants real, or a shared environment or measurement artifact? A baseline rerun is needed before any get-latency change is credited.\n- Do the index speedups, the fingerprint ghost's put gain, and the half-main ghost cap hold across other traces and capacities? Do the gains add when combined, given that the three edits overlap in ghost and index state?\n- Does the fingerprint ghost's hash-set cost, and its linear ghost-hit scan, stay acceptable in tail latency at larger sizes?\n- Does the ghost cap's drain cost on the put path come down under batching, as the push-count window in 0007 attempted? Does the window's small hit gain hold beyond one trace?\n- Do footprint claims count heap-allocated table bytes? Several recent footprint readings did not.\n- Several cycles labeled variants as Winners on weak evidence (0007, 0008, 0009). Labeling criteria should be checked before any of these are carried forward.\n\n## Per-policy notes\n\n- **Small queue:** The share increase alone left hit rate unchanged on the one workload reported (0006). Its earlier hit-rate gain (0002) did not survive stacking with the victim choice on a larger share (0003).\n- **Main queue:** The second-chance rotation budget is unexercised in the traces seen so far, so its aggregate cost is unknown. The frequency ceiling should stay at 3, and counter aging hurt hit rate in its current form. Relocating the counter into the index record (0007) did not change hit rate; a side-array counter (0004) was a footprint loss.\n- **Ghost queue:** Sequence-tagged ghost entries (0002) added put cost. Fingerprint storage is the only ghost representation with a repeated put-side benefit. Ghost size shows a non-monotonic pattern: enlarging to full capacity lost hit rate, while a half-main cap showed a small gain on one trace. Batched expiry (0007) is the only drain change that addresses the cost directly. A flat table (0009) traded put-tail for nothing measured.\n- **Policy-neutral metadata (index, hasher, layout):** The index is the most consistent structural win. Hasher choice and the bookkeeping for ghost removal are not levers for hit-path cost on current evidence.\n"
}
```

### 11:56:01 chat

I wrote `cross-cycle-insights.md` to the cycle-0009 cross-cycle output directory. It has the four required sections in order, built only from the inlined ledger digest. I did not read any previous version of the file.

The main signals across cycles 0001–0009:

- **Recurring wins:** Id-keyed indexing has been the repeated direction in cycles 0005–0008, with no hit-rate change. The fingerprint ghost showed a put-path benefit in three cycles (0003–0005). Policy-preserving metadata edits have left hit rate unchanged.
- **Recurring failures:** Lowering the frequency ceiling to 2 and aging main-queue counters both lost hit rate. Enlarging the ghost to full capacity lost hit rate. Several "Winner" labels rest on weak evidence, and I flagged that in the open questions.
- **Open questions:** The get-path latency shift that appears in variants with unchanged get code is still unexplained and needs a baseline rerun. Whether the index speedups, fingerprint ghost gain, and half-main ghost cap hold across traces and combine is also unresolved.

The file is a regenerated projection, so deleting it and rerunning the cycle should reproduce it from the same digest.


---

- finished: 2026-10-20T11:56:02+00:00
- duration: 19.18s
- status: ok
- input_tokens: 4
- output_tokens: 3787
- cache_read_tokens: 41287
- cache_write_tokens: 15210
- cost_usd: 0.005349
