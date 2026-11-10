# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-11-10T10:00:30+00:00

## timeline

### 10:00:46 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Per-key evidence works better than global signals. Two cycles in a row (0004 and 0005) improved by gating the ghost→main path on what each key carried with it: first its small-queue freq at ghost time, then its ghost age. Both gains were small but real, and the age gate is the first to gain on both traces with no losing cell.\n- What reliably moves hit-rate is which keys get into main, and how strictly. Every cycle has kept the strict two-reuse rule plus the freq reset as the floor. No looser admission rule has beaten it on all traces.\n- Filtering in small or ghost costs less than filtering at main admission. Main stays full, so any key admitted or protected there pushes out an entry that would have been hit.\n- Windows that scale with `small_size` can reach the large cache size. Fixed-size mechanisms only moved results at the small size.\n- Cost-only refactors (O(1) index/slab, fast hasher, compact `Loc`) have kept hit/miss behaviour bit-identical every time. An oracle test enforces this, so they can safely be ported onto whichever policy parent is current.\n\n## Recurring failures\n\n- Changing a single constant for the threshold or the small/main split. hm_0 and web_0 react in opposite directions, and the effect flips sign across cache sizes, so a static tweak doesn't win on both (cycles 0001–0002).\n- Global rate or occupancy signals don't work as adaptive gates:\n  - Main occupancy fills and stays full, so the gate falls back to the baseline rule.\n  - The windowed ghost-return rate is inflated by the ghost being much larger than the small queue, and it turned lenient on exactly the wrong traces.\n  - An unweighted ARC-style split tends to drift toward a larger small queue.\n- Giving returnees flat credit, or extra protection in main. Ghost returns are mostly weak single-reuse keys, so the credit keeps dead keys in main for an extra lap.\n- Mechanisms that reset freq and erase the per-key signal the ghost gate reads, such as cycle 0005's relap: a failed relap loses its direct return to main.\n- Process failure: per-op-time and metadata-size have been missing from the fitness vector for five cycles running, even though hypotheses kept assuming they were reported. Several cost-only variants are still waiting on measurement.\n\n## Open questions\n\n- How much do the cost changes actually save in per-op-time, and what do they do to metadata-size? On 8-byte keys, alignment may cancel some layout savings. None of this can be answered until the cost dimensions are in the evaluation.\n- Sweeping the age window (`GHOST_RECENT_AGE_DIVISOR`): can it move web_0 at the large size, the one cell no policy variant has moved?\n- Does a relap stack with the age gate if failed relaps keep their freq-1 ghost signal or a relapped mark?\n- Should a key's second reuse in small count only if it is recent? Reuses spread over a long window looked like weak evidence.\n- Could a direct comparison of the misses avoided by promoting against the main entries displaced by promoting beat any proxy?\n- Adaptive variants never logged where their controlled values settled (`small_size`, threshold), so it's still unclear whether those controllers actually moved.\n\n## Per-policy notes\n\n- **S3-FIFO (sole policy so far)**: the current base is the O(1) index, the strict two-reuse graduation rule with freq reset, and a ghost→main path gated on each key's ghost-time freq and ghost age (cycle 0005 winner). The cost-only changes still need porting onto it.\n- **Trace behaviour**:\n  - hm_0 holds real reuses that get squeezed out of the small queue, so a quick ghost return is a meaningful signal there.\n  - web_0 is dominated by keys accessed exactly twice and by late one-off rereferences, so it punishes lenient admission.\n- **Cache size**: policy gains concentrate at the small size. At the large size the ghost already covers long reuse windows, so most filtering changes do little there unless their window scales with the queue.\n- **Gain size**: improvements in this lineage are small. Treat near-noise changes in the small/main split as no change.\n"
}
```

### 10:00:53 thinking



### 10:00:53 chat

I've written `cross-cycle-insights.md` in the output dir. It uses the four required sections, in order, and is built only from the five-cycle ledger.

**What the ledger shows:**
- **What keeps working:** gating each key's path back into main on what that key carries with it (its small-queue freq when it was ghosted, then how recently it returned). This gave small gains in cycles 4 and 5. Cycle 5's winner is the first variant to gain on both traces without losing in any cell. Across all five cycles, the strict two-reuse rule for promotion into main has never been beaten on every trace.
- **What keeps failing:**
  - changing a single constant (threshold or small/main split), because the two traces react in opposite directions;
  - adaptive gates driven by global signals (how full main is, how often keys come back from the ghost queue, a split that adapts the way ARC does);
  - giving keys returning from ghost a flat extra credit;
  - any change that resets freq and so wipes out the signal the per-key gate reads.
- **The biggest gap is in the process:** per-op-time and metadata-size have been missing from the fitness vector for five cycles running, even though several hypotheses assumed they were being reported. Several cost-only variants are still waiting to be measured, so it's still unknown whether any of them are faster.
- **Next steps:** sweep the age window, especially to see whether it can move web_0 at the large cache size, the one cell no policy change has moved. Retry the extra pass through small, but let keys that fail it keep their signal for a direct return to main.

The ledger only covers S3-FIFO, so the per-policy section is S3-FIFO notes, split by trace and by cache size.


---

- finished: 2026-11-10T10:00:54+00:00
- duration: 23.67s
- status: ok
- input_tokens: 4
- output_tokens: 2106
- cache_read_tokens: 38866
- cache_write_tokens: 10459
- cost_usd: 0.134184
