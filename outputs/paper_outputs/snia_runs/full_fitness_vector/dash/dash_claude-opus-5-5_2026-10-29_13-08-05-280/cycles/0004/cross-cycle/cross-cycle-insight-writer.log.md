# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-29T13:42:12+00:00

## timeline

### 13:42:30 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-29_13-08-05-280/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0004)\n\n## Recurring wins\n\n- **Deciding who enters is the strongest lever (cycles 1–4).** A reuse gate at stash→target (c1, simplified in c2), an occupancy guard on *full* targets (c3) and a guard on *full* probing buckets (c4) all raised hit-rate. Each one asks for one more piece of reuse evidence before a key may enter a full bucket.\n- **Fewer multi-bucket moves helps both hit-rate and latency (c1, c3, c4).** Promotions, demotion cascades and stash churn are the shared cost. Changes that *remove* moves improve hit-rate, put time and get tail together. Changes that only *defer* moves (c4 probing guard) improve hit-rate but not tail latency.\n- **Keep the common hit path lean (c1, c2).** Any per-hit bookkeeping (counter writes, compares, conditional stores) shows up in median get. The two-hit threshold alone gives almost all of the gate's gain.\n- **A guard needs enough residence time to work (c3, c4).** A guard helps only where candidates stay long enough to collect the extra hit. Target-side guards work at every cache size. Stash-side guards need a large-cache size gate.\n- **Metadata-size: find the field that sets the size first (c3, c4).** `timestamp: Instant` set Item's size because of alignment. Replacing it with a `u32` tick on top of the `u8` counter gave the first size gain (24 bytes).\n\n## Recurring failures\n\n- **Making it cheaper to come back.** Faster re-promotion, re-admission credits (c3) and MRU promote landing (c1) all added churn and pushed out fresh or reused items. Hit-rate stayed flat or fell.\n- **Placement based on free space (c1, c2).** Balanced or free-slot placement does not change hit-rate under pressure. When there is slack it fills slots with cold evictees. Adding a reuse flag to it (c2) did not help either.\n- **Extra protection stacked on the gate (c2).** MRU demotion landing and other reuse signals mostly repeat what the gate already captures, so they add little.\n- **Narrowing a field that does not set the size (c3).** Any field of 8 bytes or less disappeared into padding until `timestamp` shrank.\n- **Trusting latency-only Winners (c2–c4).** Latency moved repeatedly with no code mechanism behind it, including on put paths that were not touched. Noise on per-op time has never been measured.\n- **Tuning the stash threshold value alone (c4).** Little headroom is left there.\n\n## Open questions\n\n- How large is the per-op-time noise? Re-run an identical baseline. This has been pending since c2 and blocks every latency conclusion.\n- Do the size-gated stash guard and the full-probing guard add up, or does the target-side guard make the stash size gate unnecessary?\n- Can the full-probing cascade itself be made cheaper, so the probing guard's hit-rate gain comes without its get-tail cost?\n- Tick maintenance: should it be per-segment or per-bucket, or removed for policies that never read it, to avoid a contended global atomic while keeping 24 bytes?\n- Should guard thresholds scale with cache or stash size? Large cache sizes are still where hit-rate is won or lost.\n- What still limits web_0 at the large size? Would a stricter probing guard (one that resets the count on any non-hottest hit) filter better?\n- Can a cheap, write-free victim guard recover the small large-cache hm_0 gain that the victim comparison bought in c1/c2?\n\n## Per-policy notes\n\n- **Dash segment tiering (stash / target / probing):** where an item lands is tied to how fast it climbs tiers, and the rule \"hottest-in-target → probing\" makes this worse. Guards at full-bucket entry points are the proven pattern. Reuse-aware changes gain most on hm_0 and at large cache sizes.\n- **ClassicLRU (the evaluated policy):** it never reads the timestamp. Shrinking or removing the timestamp field therefore cannot change hit-rate, and the only cost to watch is the cost of maintaining the tick.\n- **`lfu_counter`:** it is useful as stash-only reuse evidence. Counting in normal tiers costs median get with little benefit. A `u8` width is enough.\n"
}
```

### 13:42:36 chat

I wrote `cross-cycle-insights.md` in the cycle 0004 cross-cycle folder, built fresh from the four-cycle ledger and using the four required sections in order.

The main conclusions across cycles 0001–0004:

- **What works:** asking for one extra sign of reuse before a key can enter a full bucket. This held at three points: the stash reuse gate, the occupancy guard on full targets, and the guard on full probing buckets. Changes that remove multi-bucket moves improve hit-rate and latency together. Changes that only delay those moves improve hit-rate but not tail latency.
- **What keeps failing:** making it cheaper for items to come back (re-admission credits, MRU landing), placing items by free space alone, adding more protection on top of the gate, and narrowing fields that don't set Item's size.
- **Latency results are still untrustworthy.** Latency changes with no code change to explain them showed up in cycles 0002–0004, and no cycle has re-run an identical baseline to measure the noise. This is listed as the top open question.
- **Item size:** the 16-byte timestamp field set the size. Replacing it with a 4-byte tick, on top of the 1-byte counter, brought Item to 24 bytes, the first size gain. ClassicLRU never reads the timestamp, so this can't change hit-rate; the only cost is keeping the tick updated.

The per-policy notes cover the Dash tier structure, ClassicLRU and `lfu_counter`.


---

- finished: 2026-10-29T13:42:36+00:00
- duration: 23.97s
- status: ok
- input_tokens: 4
- output_tokens: 2087
- cache_read_tokens: 38345
- cache_write_tokens: 10007
- cost_usd: 0.129481
