# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-28T13:29:36+00:00

## timeline

### 13:29:53 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Structural changes to the admission/eviction contest keep paying off: the 1% window (c1), adaptive window (c2), SLRU main region (c3), and sampled probation victim (c5) were all clear wins. Each one changed who competes, or which resident is the victim.\n- Sharper frequency-based selection is the dependable source of gains, mostly on web_0. Examples: SLRU, the coldest-of-N victim, and less sketch collision noise.\n- Strict `>` admission, where the victim wins ties, is confirmed better than recency or `>=` tie-breaks (c4). Ties only became a problem when a counter cap or halving made them common.\n- Building in steps on the current winner, one axis per variant, has kept attribution clear. The lineage now runs: window → adaptive window → SLRU → adaptive protected split (marginal) → sampled victim.\n\n## Recurring failures\n\n- Changes to feedback-loop or controller timing have failed every cycle from c3 to c5. This covers climber sample periods (capacity-tied, constant, and turnover-tied), alternating dual climbers, and the ARC-style ghost controller. The causes repeat: noisy deltas, lazy rebalancing that hides the effect of a retarget, integer step truncation, warmup bias, and unbalanced ghost inputs that ratchet the window down to its minimum.\n- Periods tied to `capacity` (aging reset, climber sample) misbehave at one end of the size range or the other.\n- Global periodic sketch halving helped on its own (c2) but hurt once combined with the adaptive window (c3). It wears down the main region's frequency lead and creates ties. Fixing how the period was counted did not remove the interaction.\n- Counter caps combined with strict `>` cause tie-driven rejections of hot newcomers (c1).\n- Hypotheses meant to rescue recency-driven entries on hm_0 have repeatedly had their stated mechanism refuted (c3–c5).\n- Metadata-size and per-op-time have gone unmeasured for five cycles in a row, despite every continuation asking for them. Every \"win\" so far is a hit-rate-only win.\n\n## Open questions\n\n- Does single-count sketch accounting (the correct base) add to the sampled-victim win? The two gains come through different routes.\n- How far can the victim sample size grow before the per-op estimate cost outweighs web_0 gains?\n- Can hm_0 be helped on purpose through an explicit re-entry/ghost bonus for recently evicted keys, or by changing where admissions land in probation or how long they stay?\n- Is the 80/20 protected split already near optimal? The adaptive split's win was marginal.\n- Is controller work worth resuming at all? If so, only with a balanced, batched, counterfactual signal, small steps, and suppression until the cache is full.\n- Could aging that targets only stale demoted entries fix inflated counts without broad erosion?\n- What do the window, SLRU, climber, and sampling actually cost in metadata and latency?\n\n## Per-policy notes\n\n- TinyLFU baseline: its weakness is the admission contest. Once the cache is full it rejects every first-time key.\n- Window (fixed / adaptive): the fixed 1% window is too small at small sizes. Adaptive helps the small size on both traces and does nothing at large sizes. It does not fix web_0 at the large size.\n- SLRU main region: drives web_0 gains at both sizes, and was the first change to move web_0 at the large size. It costs hm_0 slightly, because the small probation segment cuts short recency-driven stays.\n- Sketch aging/halving: the regression at c1 came from the cap, not the aging. Cap-free aging helps standalone but hurts when combined with the adaptive window. Dropped on this lineage.\n- Sketch accounting: a miss followed by an insert used to count twice, which inflated one-time keys and defeated frequency floors. hm_0 small had been relying on that accidental bias. Single-count is correct but costs hm_0 small.\n- Victim selection: coldest of the oldest few probation entries is a clean win on web_0. Further victim-choice tweaks at the LRU end are not expected to help hm_0.\n- Traces: web_0 is scan-heavy and responds to frequency filtering. hm_0 is recency-driven, gains little at the large size (LRU already captures most of it), and regresses at the small size under almost every change. It is the persistent hard point.\n"
}
```

### 13:30:01 chat

I've written `cross-cycle-insights.md` in the cycle 0005 cross-cycle directory, built only from the five-cycle ledger, with the four required sections in order.

**Recurring wins:** Structural changes to who competes for admission and which resident gets evicted won every time they were tried:
- the fixed 1% window (cycle 1)
- the adaptive window (cycle 2)
- the probation/protected split (cycle 3)
- picking the coldest of the oldest few probation entries as the victim (cycle 5)

Gains came mostly on web_0, through sharper selection by frequency. Strict `>` admission, where the resident wins ties, is confirmed as the better rule.

**Recurring failures:**
- **Controller timing:** changes to how often or how the window controllers adjust failed every cycle from 3 to 5.
- **Capacity-tied periods:** periods tied to cache capacity misbehave at one end of the size range.
- **Global sketch halving:** it helped on its own but hurt once combined with the adaptive window.
- **hm_0 rescues:** every attempt to help hm_0's recency-driven entries had its predicted mechanism disproved.
- **Cost never measured:** metadata size and per-operation time have gone unmeasured for five cycles in a row, so every win so far is a hit-rate-only win.

**Open questions:**
- Does the count-each-access-once sketch fix add to the cycle 5 win?
- How large can the victim sample get before its per-operation cost outweighs the gain?
- Would an explicit bonus for recently evicted keys that return help hm_0?
- Is the 80/20 split already close to optimal?
- Is any controller work worth resuming at all?

**Per-policy notes:** Short notes on the baseline, the window, the split, aging, sketch counting, victim selection, and the two traces. hm_0 at the small cache size stands out as the point that keeps regressing.


---

- finished: 2026-10-28T13:30:01+00:00
- duration: 25.35s
- status: ok
- input_tokens: 4
- output_tokens: 2248
- cache_read_tokens: 39314
- cache_write_tokens: 10996
- cost_usd: 0.140807
