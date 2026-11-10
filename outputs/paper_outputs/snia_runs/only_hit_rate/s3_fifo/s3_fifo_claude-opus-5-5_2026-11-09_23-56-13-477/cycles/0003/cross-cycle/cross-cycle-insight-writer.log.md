# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-11-10T09:47:36+00:00

## timeline

### 09:47:50 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- The baseline strict two-reuse graduation rule plus a freq reset on graduation has survived three cycles. No threshold, split, gate or credit variant has beaten it on every trace.\n- The winning variant in every cycle has been the one that regressed least, not one that clearly improved. Policy gains have been marginal, and the policy looks like it has plateaued on these traces.\n- Of all the policy levers, stricter admission into a full main is the only one that has produced a clear gain. Which keys get into main matters more than how big the queues are.\n- The O(1) index/slab refactor has kept hit/miss behaviour identical twice (cycles 1 and 3, the second time enforced by an oracle test). It is a safe structure to build further policy work on.\n\n## Recurring failures\n\n- Changing one constant at a time (graduation threshold, static small/main split) splits the traces: hm_0, mainly at the small size, gains, while web_0 and hm_0 at the large size lose. The early \"hm_0 wants a looser threshold\" result came from cold-start or small-cache conditions.\n- Adaptive signals that go flat or are biased have failed. Main occupancy saturates, so gating on it falls back to the baseline. Ghost-return rate is inflated by the ghost being bigger than the small queue, so it turned lenient on exactly the traces where strictness helps.\n- Rewarding the ghost path in any way (loosening admission on ghost returns, giving returnees credit) costs hit-rate. In steady state the ghost mostly holds weak single-reuse keys.\n- Extra protection inside a full main (graduate credit, second chances) just evicts other useful entries. The cost of a bad admission is paid when the key enters main and can't be repaired later.\n- Adaptive ARC-style splitting moved hit-rate only by about noise, with the sign flipping across traces. Without size weighting it tends to drift toward a larger small queue.\n- Per-op-time and metadata-size have not been measured for three cycles running, despite being requested each time. Cost-side work is still unvalidated.\n\n## Open questions\n\n- How much does the O(1) structure actually cut per-op-time, and how much metadata-size does it add, compared with linear scans? This blocks any decision about cost trade-offs.\n- Can per-key evidence beat the global rate proxies that have failed so far? Candidates are small-queue freq at the time a key is ghosted, or a direct comparison of the misses avoided by promoting a key against the cost of the main entry it displaces.\n- Where do the adaptive variables (`small_size`, the switched threshold) settle on each trace? This has never been logged, and logging it would explain why the adaptive variants failed.\n- Is the remaining hit-rate gap closable at all on hm_0/web_0, or should the remaining hypothesis budget go to reducing cost and metadata?\n\n## Per-policy notes\n\n- S3-FIFO (the only policy explored): the current parent is the O(1) index/slab structure with the occupancy-gated two-reuse rule and freq reset. Because main saturates, this effectively behaves like the baseline two-reuse rule.\n- S3-FIFO small/main split: static values from 10% to 25% and the adaptive split have all been roughly neutral. Treat the split as a low-yield knob.\n- S3-FIFO ghost queue: it is useful as a filter but not as a reward path or a global signal. It is about 3× the small queue in size, and on the linear structure each scan adds per-miss cost.\n- Trace behaviour: web_0 consistently punishes admitting keys accessed exactly twice or reused at medium frequency. hm_0 tolerates looser admission only at the small cache size.\n"
}
```

### 09:47:58 thinking



### 09:47:58 chat

I've written `cross-cycle-insights.md` from the three-cycle ledger, using the four required sections. It doesn't include any numbers, because the ledger only gives qualitative results.

- **What keeps working:** the baseline rule (a key moves to the main queue on its second reuse, and its frequency count is reset when it does) has never been beaten on every trace. Making entry into a full main queue stricter is the only change that has clearly improved hit-rate. Each cycle's winner was simply the variant that lost least, so the policy looks like it has stopped improving on these traces.
- **What keeps failing:**
  - Changing one setting at a time helps one trace and hurts the other.
  - Adaptive signals that go flat or are skewed don't work: main-queue fullness stays maxed out, and the ghost-return rate is inflated.
  - Rewarding keys that come back from the ghost queue costs hit-rate.
  - Giving entries extra protection once they are in a full main queue just pushes out other useful entries.
  - The adaptive small/main split moves hit-rate by about as much as noise.
- **Unmeasured costs:** time per operation and metadata size have gone unmeasured for three cycles in a row, despite being asked for each time. The faster index rewrite has been shown to give identical hits and misses, but nobody knows yet whether it is actually faster or how much extra memory it uses.
- **Open questions for later cycles:**
  - Can evidence carried by each key beat the global rates that have failed so far?
  - Where do the adaptive settings end up on each trace? This was never logged.
  - Should the remaining effort go into reducing cost and metadata instead of hit-rate?
- **Per-policy notes:** everything so far is S3-FIFO, so the notes cover the current base version, why the split size is a weak lever, what the ghost queue is and isn't good for, and how web_0 and hm_0 each respond to looser entry into main.

Files are in /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0003/cross-cycle:
- cross-cycle-insights.md


---

- finished: 2026-11-10T09:47:59+00:00
- duration: 22.90s
- status: ok
- input_tokens: 4
- output_tokens: 2069
- cache_read_tokens: 36293
- cache_write_tokens: 8949
- cost_usd: 0.120655
