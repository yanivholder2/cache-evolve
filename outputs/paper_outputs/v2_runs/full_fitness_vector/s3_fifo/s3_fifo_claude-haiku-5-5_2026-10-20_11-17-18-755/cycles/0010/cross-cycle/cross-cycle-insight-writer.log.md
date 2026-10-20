# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T11:59:58+00:00

## timeline

### 12:00:07 thinking



### 12:00:14 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Id-keyed indexing for resident and ghost lookup is the most repeated direction (Cycles 0005–0008, merged index in 0006). It removes linear scans on the put path and shows no hit-rate change. Get and put speedups still need a baseline rerun before they are credited.\n- Fingerprint-keyed ghost storage has given a put-side benefit with no hit-rate change in several cycles (0003, 0004, 0005). Its cost is container-level bookkeeping, not per-entry size. Combining it with a ghost cap (0006) eroded that put gain.\n- Batched ghost expiry takes drain work off the per-eviction path (0007 window, 0010 batched prune). It is a useful put-path tool even where it does not change hit rate.\n- Policy-preserving data-structure changes (index swaps, counter relocation) leave hit rate unchanged. That is expected, and it makes them safe to keep when the cost case is clear.\n\n## Recurring failures\n\n- Weakening frequency signals loses hit rate. Lowering the frequency ceiling to 2 (0008), aging main-queue counters (0009), and promoting on `freq > 0` (0001) each cost hit rate on the trace tested. The frequency signal appears to earn its keep.\n- Enlarging the ghost toward full capacity lost hit rate (0004). Ghost capacity is not a free dial.\n- Victim choice after the rotation budget did not compose with a larger small queue (0002, dropped in 0003).\n- Hash-function swaps did not reduce get cost; the multiply-shift hasher made gets slower (0008).\n- Splitting per-entry state into side arrays saved bytes in the entry but grew the container, a net footprint loss (0004).\n- Linear-probing ghost tables showed a put-tail regression (0009). Tombstone tables at lower load factor (0010) addressed it only partially, since the change bundled two edits.\n- Get-path timing shifts appear in variants whose get code is unchanged (0001–0010). Treat them as shared noise, not as get-path effects.\n- Labeling is inconsistent. Several cycles labeled variants Winner when their own evidence showed no hit-rate gain and no clear, unambiguous cost gain (0002, 0003, 0009, 0010). Winner labels should be checked against the evidence before carrying a variant forward.\n- Nearly every hit-rate movement rests on one trace at one capacity. None of the hit-rate results is yet robust.\n\n## Open questions\n\n- Is the get-path shift real and reproducible with no code change? It has been unexplained for ten cycles and needs a baseline rerun.\n- Does the fingerprint ghost's put-latency gain hold across traces and capacities, and is its hash-set cost acceptable at larger sizes? Heap-side ghost bytes have not been measured.\n- Does the ghost cap's small hit-rate gain (0005, 0009) survive on other traces, and does it still help once expiry is batched (0010 found it flat)?\n- Do the ghost-storage variants (fingerprint, tombstone table, bit array, index) compose? None has been tested in combination, and they touch overlapping state.\n- Does the bit array's hit-rate uptick (0010) hold on other workloads, given its false-positive rate?\n- Does a much longer aging period, or per-entry unhit windows, recover any hit rate lost to frequency aging?\n\n## Per-policy notes\n\n- Small queue: the promotion threshold at `freq > 0` was dropped (0001). A larger share alone showed no hit-rate change (0006). A larger share paired with victim choice gave a small single-trace gain (0002), but the gain did not carry over when the two were stacked (0003).\n- Main queue: the rotation budget is the bounding knob. Post-budget victim choice is untested in the enlarged-small-queue regime. Frequency aging hurt hit rate (0009). The frequency ceiling of 3 should stay (0008).\n- Ghost queue: fingerprint storage is the most consistent put-side win. Full-capacity ghosts lost hit rate (0004). A half-main cap gave a small gain that did not reproduce once expiry was batched (0010). Batched expiry is worth keeping. The tombstone table and bit array are unverified on other traces.\n- Metadata and index: id-keyed maps are the lead direction for lookup cost. Counter relocation out of `Entry` is hit-rate neutral, but its footprint effect is unmeasured. Hasher changes are not a get-path lever.\n"
}
```

### 12:00:16 chat

I wrote `cross-cycle-insights.md` to the cycle 0010 cross-cycle output directory. It has the four required sections in order, built only from the inlined ledger digest for cycles 0001–0010. I did not read the previous file, and I did not invent any numbers.

The main synthesis:

- **Recurring wins:** id-keyed indexing is the most repeated direction, with no hit-rate cost. The fingerprint ghost has a put-side benefit across several cycles. Batched ghost expiry is useful on the put path.
- **Recurring failures:** weakening the frequency signal (ceiling 3→2, main-queue aging, `freq > 0` promotion) lost hit rate. Enlarging the ghost toward full capacity also lost hit rate. Get-path timing shifts in variants with unchanged get code are treated as noise. Several Winner labels are not supported by their own evidence, and most hit-rate movement rests on a single trace.
- **Open questions:** whether the get-path shift is real, whether the fingerprint ghost gain holds at other sizes, whether the ghost storage variants compose (they have not been tested together), and whether the bit array's hit-rate uptick survives other workloads.
- **Per-policy notes:** grouped by small queue, main queue, ghost queue, and metadata/index.


---

- finished: 2026-10-20T12:00:16+00:00
- duration: 18.65s
- status: ok
- input_tokens: 4
- output_tokens: 3700
- cache_read_tokens: 42286
- cache_write_tokens: 16098
- cost_usd: 0.005493
