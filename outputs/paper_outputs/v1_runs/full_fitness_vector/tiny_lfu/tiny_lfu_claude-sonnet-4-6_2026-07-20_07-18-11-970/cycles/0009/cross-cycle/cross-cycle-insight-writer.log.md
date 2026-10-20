# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0009\cross-cycle
- allow_edits: True
- started: 2026-07-20T06:21:00+00:00

## timeline

### 06:21:08 thinking

Let me analyze the ledger digest across all 9 cycles and write the cross-cycle-insights.md file. I need to find the output directory first.

### 06:21:08 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 06:21:08 tool: Glob

```json
{
  "pattern": "**/output/**"
}
```

### 06:21:12 tool: Bash

```json
{
  "command": "ls /c/Users/yaniv/Projects/cache-evolve/ 2>/dev/null || ls C:/Users/yaniv/Projects/cache-evolve/ 2>/dev/null || echo \"not found\"",
  "description": "List cache-evolve project directory"
}
```

### 06:21:14 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve"
}
```

### 06:21:17 thinking

The output directory is `C:\Users\yaniv\Projects\cache-evolve\outputs\tiny_lfu_2026-07-20_07-18-11-970\cycles\0009\cross-cycle`. I should write the file there.

### 06:21:17 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0009\\cross-cycle"
}
```

### 06:21:43 thinking

Now I have the output directory. Let me write the cross-cycle-insights.md file there. I'll analyze all 9 cycles' ledger data:

**Recurring wins (patterns that held up across multiple cycles):**
1. Amortised/sampled operations on hot paths — the 1-in-8 eviction probe (cycles 4-6) and amortised admission gate (cycle 6 winner) both worked by reducing overhead on the common path
2. Eliminating serial data dependencies in hash computation — bijective remix (cycle 5 winner) beat full second SipHash by removing the serial chain
3. Row independence in the sketch — two-seeded hash (cycle 4 v001) and sketch width doubling (cycle 7 v003) both improved hit rates by reducing collision noise
4. Protected segment and policy structure — proved effective when first introduced in cycle 8 v002 and refined further in cycle 9

**Recurring failures:**
1. Sketch decay — attempted multiple times across cycles 1, 2, and 3 with different interval strategies, but consistently underperformed
2. Aggressive counter layout changes — compacting counters to u32 or u16 in cycles 1-2 and 7-8 never yielded improvements despite latency costs
3. Flat sketch allocation and depth reduction — definitively ruled out after cycle 2
4. Best-of-2 victim selection — cycles 3 and 4 showed it's too expensive on every eviction, and cycle 7 v001 confirmed it adds nothing even at near-zero cost
5. Double-increment and proportional frequency amplification — closed after cycle 8 v003
6. Remix chains replacing independent hash seeds — cycle 6 v003 eliminated this approach
7. Struct size growth beyond cache line boundaries — consistently causes disproportionate latency penalties

**Open questions:**
1. Lazy promotion in the protected segment during gets to reduce put-path overhead — cycle 9's most promising direction
2. Clock-hand or generational eviction as alternatives to SLRU bookkeeping
3. Finding the optimal protected-zone ratio

I'm now looking at whether the protected segment ratio can be refined further below 20%, and whether bitset compaction combined with compound sketch might unlock latency improvements not yet visible in the current metrics. The window-to-protected-segment ratio and the promotion threshold from probationary to protected segments remain unexplored despite being mentioned in earlier cycles. For TinyLFU specifically, sketch decay is settled, the single-hash bit-slice remix is the right approach, four sketch rows handle collisions effectively, and width genuinely impacts hit rate — though u16 counters save heap space without improving latency at the scales I've tested. On eviction, best-of-2 victim selection by comparing LRU heads is closed since the head and its neighbor are equally cold.

For SLRU, the 20% protected fraction is near-optimal on Zipfian traces, the dual-list bookkeeping on the put path is the real cost driver, bitset flags outperform vector storage, and compounding with an amortized gate yields only marginal hit-rate gains. The sampling rate for admission gating is locked at 1-in-8 — 1-in-4 breaks latency. LRU as a baseline is surprisingly strong on Zipfian and LIRS traces because the working set is stable, frequency signals from accumulated counts are reliable, and decay actually destroys that signal. Now I'm ready to write this up.

### 06:21:43 chat

Now I have the output directory. Let me write the cross-cycle-insights.md based on all 9 cycles of ledger data.

### 06:22:20 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\tiny_lfu_2026-07-20_07-18-11-970\\cycles\\0009\\cross-cycle\\cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Amortised sampling on the eviction/gate path beats eager probing.** The 1-in-8 fire rate proved durable across cycles 4–6: it preserves latency neutrality while retaining a meaningful hit-rate signal. Anything that fires new sketch probes more often than 1-in-8 on the common put path regresses p50 or p99.\n- **Eliminating serial hash data dependencies pays off.** The bijective multiply-xorshift remix (cycle 5 winner) beat a full second SipHash by removing an intra-iteration dependency rather than just reducing hash count. The principle: ILP loss from chained hash data dominates the arithmetic savings of fewer invocations.\n- **Sketch row independence is a genuine accuracy lever, not a theoretical nicety.** Both the two-seeded hash split (cycle 4) and the 8× width doubling (cycle 7) improved hit rate by reducing collision noise, confirming that the 4× baseline was collision-limited on this Zipfian trace.\n- **Structural policy changes (SLRU segment split) opened a new, orthogonal axis.** The protected-segment variant (cycle 8) was the first non-sketch, non-hash change to deliver a hit-rate gain; it compounded cleanly with prior sketch and gate tuning, proving policy structure and estimation accuracy are independent levers.\n- **Compact flag representation (bitset over Vec) is a safe, cost-neutral improvement.** Cycle 9's bitset compaction reduced backing heap 8× with no regression, establishing it as the correct default for any future flag-per-node design.\n\n## Recurring failures\n\n- **Sketch decay in any tested form.** Both the width/2 cadence (cycle 1) and the capacity×10 cadence (cycle 3) hurt hit rate and latency on the LIRS trace. The stable working set makes accumulated frequency a reliable admission signal; blanket half-decay degrades it uniformly and the full-table sweep pressures L2/L3. **Axis is closed.**\n- **Counter-width narrowing (u16) as a latency lever.** Tested in cycles 7 and 8; per-access zero-extension overhead consistently cancels the L2 footprint savings at these sketch sizes. **Do not revisit unless per-access cast overhead is eliminated.**\n- **Flat sketch allocation and reduced sketch depth.** Cycle 2 showed flat allocation creates cache-set conflicts that dwarf pointer-indirection savings; halving depth degrades collision suppression enough to cascade into eviction errors. **Both are definitively closed.**\n- **Best-of-2 victim selection at the LRU cold end.** Tested at full cost (cycle 3), amortised 1-in-8 (cycle 4 v002 was eviction amortisation, not victim selection), and near-zero overhead (cycle 7 v001). The LRU head and its immediate neighbour are equally cold relative to the working set — the admission gate, not victim identity, is the operative filter. **Axis is closed.**\n- **Proportional frequency amplification (double-increment on hits).** Cycle 8 v003: cold residents accumulate the boosted signal at the same relative rate as incomers accumulate misses; the `incoming_freq > victim_freq` gate is structurally unaffected and the overhead falls on the majority hit path. **Closed.**\n- **Bijective remix chains replacing independent hash seeds.** Cycle 6 v003: a common hash root produces correlated row positions, raising collision noise, and the sequential multiply–rotate–XOR chain stalls out-of-order execution worse than a second independent SipHash. **Closed.**\n- **Growing the main cache struct past cache-line boundaries.** Multiple cycles showed that even adding 8 bytes to `TinyLfuCache` can double p50/p99 on code paths that never execute the new logic, via cache-line boundary shift. Any new field must fit existing alignment padding or demonstrate the trade-off is worth it.\n\n## Open questions\n\n- **Can lazy (get-only) promotion reduce put-path dual-list overhead in the SLRU design?** Cycle 9 noted that every full-cache put currently walks and relinks two chains; deferring segment promotion to `get` only may eliminate this cost while preserving the hit-rate benefit of the segment split.\n- **Is there a clock-hand or generational eviction policy that delivers SLRU-like admission filtering with lower per-operation bookkeeping than dual-list maintenance?**\n- **What is the cycle-level compound ceiling?** The current stack (bijective-remix sketch + amortised admission gate + protected segment + bitset flags) has not been evaluated as a single integrated variant; each layer was added incrementally and the true combined fitness is unknown.\n- **Do structural policy parameters (window/protected ratio, probationary-to-protected promotion threshold) still have unexplored headroom?** The protected fraction was pinned at ≤20% and confirmed near-optimal for a Zipfian trace, but the promotion threshold and window-to-main ratio have never been varied across any cycle.\n- **Can the admission gate and the protected segment be decoupled?** Cycle 9 observed they are partially redundant — the gate already shields hot keys, so the segment adds only a marginal increment while taxing put-path latency. A cleaner separation may reveal which mechanism carries the majority of the benefit.\n\n## Per-policy notes\n\n- **TinyLFU / Count-Min Sketch:** Four rows are load-bearing — removing two rows (cycle 2) cascaded into eviction errors. Width is a genuine hit-rate lever; 8× outperforms 4× on this Zipfian trace. The correct hash strategy is bijective-remix for the second row pair (not a full second SipHash, not a pure single-hash bit-slice). u16 counters do not improve latency at current sketch sizes. Sketch decay in any form degrades rather than improves accuracy on this workload.\n- **LRU / probationary zone:** The LIRS trace's stable working set makes raw LRU a strong baseline; frequency accumulated over time is a reliable signal that aggressive resets destroy. The 1-in-8 amortised probe rate is the empirically validated boundary between latency-safe and latency-harmful gate firing.\n- **SLRU / protected segment:** 20% protected fraction is near-optimal for a Zipfian trace; 30% backfires by filling slots with lukewarm items and shrinking the probationary pool. Bitset storage is the correct representation for per-node flags. The dominant cost of the dual-list design is the put-path conditional relinking, not the eviction-path flag lookup. Compounding the protected segment with the amortised gate and 8× sketch yields a winner, but the marginal contribution of each layer diminishes as prior layers already handle the same failure modes.\n- **Eviction candidate selection:** Best-of-2 victim comparison is closed — the LRU head and neighbour are fungible cold candidates. The admission gate (`incoming_freq > victim_freq`) is the operative lever, not which cold node is chosen as the victim.\n"
}
```

### 06:22:28 chat

`cross-cycle-insights.md` written to the cycle 0009 cross-cycle output directory. Here's a summary of what was synthesized across all nine cycles:

**Recurring wins** — amortised 1-in-8 sampling on the eviction path; eliminating serial hash data dependencies (bijective remix over chained hashes); sketch row independence as a real accuracy lever; SLRU protected segment as an orthogonal policy axis; bitset flag storage as cost-neutral compaction.

**Recurring failures** — sketch decay in any cadence (definitively closed); u16 counter narrowing; flat sketch allocation and depth reduction; best-of-2 victim selection at the LRU cold end (the head and its neighbour are equally cold); double-increment frequency amplification; bijective remix chains replacing independent seeds; struct size growth past cache-line boundaries.

**Open questions** — lazy/get-only promotion to reduce SLRU put-path overhead; clock-hand or generational eviction as a lower-overhead alternative to dual-list bookkeeping; the true compound ceiling of all current winning mechanisms together; window/promotion-threshold parameters still untouched; whether the admission gate and protected segment are genuinely additive or partially redundant.

**Per-policy notes** — grouped observations on the sketch, probationary LRU, SLRU segment, and eviction candidate selection axes.


---

- finished: 2026-07-20T06:22:29+00:00
- duration: 88.41s
- status: ok
- input_tokens: 7
- output_tokens: 3536
- cache_read_tokens: 124786
- cache_write_tokens: 12371
- cost_usd: 0.144108
