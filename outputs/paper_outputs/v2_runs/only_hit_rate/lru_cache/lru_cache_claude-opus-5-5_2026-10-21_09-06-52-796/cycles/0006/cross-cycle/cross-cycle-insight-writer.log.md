# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:41:28+00:00

## timeline

### 09:41:48 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Admission filtering beats recency tracking.** Every winner since cycle 1 has changed what gets into `main`, not how recency is tracked. S3-FIFO (probation queue plus ghost) has been the lineage parent ever since.\n- **Tightening the admission window paid off for several cycles in a row.** Shrinking the ghost won in cycles 3, 4 and 5. Shrinking `small` won in cycle 6. Each gain was smaller than the one before, and the curve is now close to flat.\n- **Ghost and `small` work as one combined admission window.** A ghost smaller than `small` adds almost nothing. A ghost larger than `small` acts as a cheap second stage of probation: it costs a ghost key instead of a resident slot, and the freed slots go to `main`.\n- **Once the ghost window is short, a ghost hit is a trustworthy reuse signal.** Sending ghost hits straight into `main` beats making those keys prove reuse again.\n- **Representation-only changes leave hit rate exactly as it was.** The fingerprint ring and CLOCK's coarse recency both did. They are safe to stack on any policy, but their memory and speed benefits are still unproven.\n\n## Recurring failures\n\n- **`metadata-size` and `per-op-time` have never been measured, in six cycles.** Every claim about memory or speed, including the representation track, the cost of the sketches and LIRS overhead, is unverified. This is the main blocker.\n- **Loosening admission costs hit rate.** That includes longer ghost windows, ARC-style adaptive resizing of `small`, and fast paths into `main` that skip proof of reuse.\n- **Admission can also be too strict.** Two-hit promotion bars and forcing ghost hits back through a tiny `small` evicted keys whose reuse gap was only slightly longer than the probation queue.\n- **Gates applied only to the ghost path leak.** Keys still reach `main` through `small`. Gates that compared against an entry other than the real eviction victim were also ineffective.\n- **Frequency admission is near its ceiling.** Count-min gates and full TinyLFU admission against the true victim (cycles 4 and 5) gained only marginally on top of the S3-FIFO filter, while adding per-access cost that has never been measured.\n- **No evidence beyond a single setup.** Every result comes from one trace (workload-a) at one cache size. A second size was requested in cycles 4, 5 and 6 and has never been delivered.\n- **Planned follow-ups keep getting skipped.** The hasher plus narrow-index graft has never been attempted. The fingerprint ring was tested once in cycle 5 but has not been ported onto the current parent.\n\n## Open questions\n\n- How should the combined window be split between resident `small` and the ghost? Should `small + ghost` stay roughly constant as slots move from `small` to the ghost?\n- How small can `small` get before keys reaching `main` with no frequency credit start to hurt hit rate?\n- Does widening `main`'s frequency cap (retention) stack with the `small`/ghost split, or do the two overlap?\n- Do the small, tuned gains hold at a second cache size or on another trace, or do they just fit workload-a's reuse distances?\n- Do the fingerprint ring and the hasher plus narrow-index graft actually cut memory and per-op time once they can be measured? Can a single hash serve both the fingerprint and the lookup?\n- Are there hit-rate levers outside admission (retention, eviction order inside `main`) that are worth more than the leftover admission tuning?\n\n## Per-policy notes\n\n- **S3-FIFO:** The lineage winner. Current parent: `small` at about a twentieth of capacity and the ghost at about an eighth. The fixed small split is better than adaptive sizing. Ghost bookkeeping has a correct sequence-number fix for stale erases, though the case it guards against almost never happens. Widening the frequency cap gave a small retention gain that is orthogonal to the window split.\n- **CLOCK (plain):** Matches LRU's hit rate, so exact LRU ordering is not needed. Its only appeal would be lower memory or latency, which is unmeasured. A possible `put` sweep latency spike when most entries are marked referenced is unexamined. Deprioritized.\n- **LIRS:** Beats LRU but trails S3-FIFO. Enlarging the HIR queue hurt. Its likely heavier nodes and a linear walk to enforce the ghost bound are unmeasured. Set aside until cost can be measured and the bound walk is O(1).\n- **TinyLFU / count-min admission:** Fixing the leak through `small` and comparing against the real victim was not enough. Gains stayed marginal on top of S3-FIFO. Drop unless measurement shows it is nearly free.\n- **ARC-style adaptivity:** Drifted away from the right split and lost hit rate. Dropped.\n- **Strict LRU plus representation changes (Fx hashing, `u32` links, MRU fast path):** Policy-neutral and never measured. They remain a pending graft onto the S3-FIFO parent.\n"
}
```

### 09:41:55 thinking



### 09:41:55 chat

I wrote `cross-cycle-insights.md` in the cycle 0006 cross-cycle directory, generated fresh from the ledger digest for cycles 1–6. It uses the four required sections in order: Recurring wins, Recurring failures, Open questions, Per-policy notes.

The main threads it carries forward:

- **Measurement has never happened.** No cycle has measured `metadata-size` or `per-op-time`, so every memory and speed claim is unverified. The file lists this first under recurring failures because it blocks judging most hypotheses.
- **What has paid off:**
  - Changing which keys get into `main` (admission filtering) has won every cycle; changing how recency is tracked has not.
  - Making the admission window tighter kept helping, but each gain was smaller and the curve is now close to flat.
  - The ghost and `small` act as one combined admission window, so they should be tuned together rather than separately.
- **What has not paid off:**
  - Loosening admission costs hit rate.
  - Making admission too strict (two-hit bars, forcing ghost hits back through `small`) also costs hit rate.
  - Gates applied only to the ghost path leak, because keys still reach `main` through `small`.
  - TinyLFU and count-min frequency gates gained only marginally.
  - All results come from one trace at one cache size; a second size was requested three times and never run.
- **Next to probe:**
  - How to split the combined window between `small` and the ghost.
  - Whether the wider frequency cap stacks with that split.
  - Whether the results hold at a second cache size.
  - The two representation changes still pending (fingerprint ring, hasher and narrow-index graft), once they can be measured.

Per-policy notes cover S3-FIFO (the current parent), CLOCK, LIRS, TinyLFU, ARC-style adaptivity, and strict LRU with representation changes.


---

- finished: 2026-10-21T09:41:56+00:00
- duration: 27.19s
- status: ok
- input_tokens: 4
- output_tokens: 2493
- cache_read_tokens: 38974
- cache_write_tokens: 12154
- cost_usd: 0.154903
