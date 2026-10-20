# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-19T12:07:42+00:00

## timeline

### 12:07:55 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Moving away from strict LRU recency order has improved hit-rate on workload-a in both cycles. Second-touch promotion, second-chance bits and quick demotion of new keys all beat plain LRU.\n- A hit path with no relinking (CLOCK, GCLOCK, SIEVE) keeps get latency low in both cycles. Hand-based designs have the cheapest get.\n- Node layout is the dependable lever for metadata-size. Fewer links or narrower `u32` index links shrink the node and usually help speed through better locality. Adding small policy state (`u8` freq, `bool` visited) in padding bytes costs no metadata.\n- Speed gains have come from memory locality (compact nodes, fewer pointer writes) rather than shorter code paths.\n- Simple single-queue designs can match or beat multi-segment ones. SIEVE (one queue, one bit) has the best result so far and improved on every dimension.\n\n## Recurring failures\n\n- Changing only where plain LRU inserts or orders keys has run out of room. It was dropped after cycle 0001.\n- Scan defenses stacked on each other don't add up. BIP head insertion inside SLRU probation lost hit-rate because it shrank the window in which new keys can earn a second touch.\n- Deeper per-slot credit on a hand-based policy (GCLOCK 2-bit) buys hit-rate with longer sweeps, which hurts put p99.\n- The cost of hand sweeps keeps appearing in the put tail: it was a concern for CLOCK in cycle 0001 and was confirmed for GCLOCK and SIEVE in cycle 0002.\n\n## Open questions\n\n- Can the sweep be bounded without losing the frequency or visited signal? Options include a capped number of steps per insert, a smaller `FREQ_MAX` or a faster decrement.\n- Can a SIEVE/GCLOCK hybrid (small visited counter) gain hit-rate without GCLOCK's tail cost?\n- Do the rankings hold at other cache sizes or on other traces? Every result so far comes from workload-a at one size (2048), so the leaders may be overfit.\n- Would changing the SLRU probation/protected split or adding an admission filter beat SLRU's current split, given that changing the insertion position did not?\n- Some small speed differences (for example BIP's in cycle 0001) may be benchmark noise. This has not been checked.\n\n## Per-policy notes\n\n- **LRU (baseline):** a weak hit-rate reference on workload-a. Further tweaks to its ordering alone are not worth pursuing.\n- **SLRU:** a solid winner in cycle 0001 thanks to the probation/protected split and `u32` links. Adding BIP inside probation hurt hit-rate. If revisited, tune the segment sizes or admission rather than the insertion position.\n- **CLOCK:** gained hit-rate unexpectedly, smallest node, relink-free hits. Its sweep cost is the known weak spot.\n- **GCLOCK (2-bit):** more hit-rate than CLOCK but clearly worse put p99 from longer sweeps. Useful mainly as a reference for what frequency credit costs.\n- **BIP:** a winner as a standalone change to LRU (loop resistance), with metadata flat or slightly up. It is harmful when combined with SLRU probation.\n- **SIEVE:** the current main parent: best hit-rate and better on all dimensions, though by a narrow margin. Its smallest gain is at put p99 because of long runs of visited nodes, so the next target is bounding the sweep.\n"
}
```

### 12:08:01 chat

I wrote `cross-cycle-insights.md` from cycles 0001 and 0002, using the four required sections in order.

- **Recurring wins:**
  - Leaving strict LRU ordering has raised hit-rate on the workload-a trace in both cycles.
  - Designs that don't relink nodes on a hit (CLOCK, GCLOCK, SIEVE) keep get latency low.
  - Smaller nodes with narrower links are the reliable way to cut memory overhead. Small policy flags fit in spare bytes at no cost.
  - Simple one-queue designs, SIEVE above all, have matched or beaten multi-segment ones.
- **Recurring failures:**
  - Tweaking only how plain LRU orders keys is no longer worth pursuing.
  - Stacking two scan defenses doesn't help: BIP inside SLRU's probation segment lost hit-rate.
  - Per-slot counters on hand-based policies lengthen the hand sweep, which slows the worst-case put. That problem has shown up in both cycles.
- **Open questions:**
  - Can the sweep be capped without losing the visited or frequency signal?
  - Would a SIEVE/GCLOCK hybrid gain hit-rate without GCLOCK's slow worst-case puts?
  - Do the rankings hold beyond the one trace at size 2048? Every result so far comes from that single setup, so the leaders may be overfit.
  - Would retuning SLRU's segment sizes or adding an admission filter work better than changing where new keys are inserted?
  - Are some small speed differences just benchmark noise?
- **Per-policy notes:** one short entry each for LRU, SLRU, CLOCK, GCLOCK, BIP and SIEVE. SIEVE is marked as the current lead parent, with capping its sweep as the next target.

The file contains no numbers.


---

- finished: 2026-10-19T12:08:02+00:00
- duration: 20.03s
- status: ok
- input_tokens: 4
- output_tokens: 1966
- cache_read_tokens: 34757
- cache_write_tokens: 7431
- cost_usd: 0.105735
