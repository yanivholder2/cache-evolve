# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-28T13:05:40+00:00

## timeline

### 13:05:55 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changing the inputs to TinyLFU's admission contest pays off; changing eviction order alone has not been the lever. Winners so far change who competes (window), who the victim is (probation/SLRU), or how fresh the frequency evidence is (aging).\n- A window in front of the main cache is the backbone of every winner: first a fixed small window, then an adaptive (hill-climbed) window built on it. Window sizing matters most at small cache sizes.\n- SLRU (probation/protected) has improved web_0 consistently in both cycles. It is the strongest lever for scan-like, one-hit-heavy traffic and is the only component that has helped at web_0's large size.\n- Cap-free sketch aging has helped web_0 at both sizes. Removing the counter cap fixed the cycle-1 large-size regression, which confirms that the cap, not aging, was the cause.\n- The W-TinyLFU components look complementary rather than overlapping: each helps a different trace/size region, so layering them one at a time is promising.\n\n## Recurring failures\n\n- Saturating counters (cap of 15) combined with strict `>` admission: hot keys tie at the cap and tied newcomers get rejected. This approach is dropped.\n- Periods tied to `capacity` (sketch reset interval, climber sample period) misbehave at one end of the size range. Aging fires too early at small sizes, and the climber adapts too slowly at large sizes.\n- Aging periods counted per sketch increment over-age small caches, because each miss produces two increments. On hm_0 this halves steady frequent keys before they can build a lead.\n- Gains shrink at large sizes, especially on hm_0, where plain recency already captures most of the reuse set. Admission tweaks have little headroom there.\n- Metadata-size and per-op-time have never been measured, even though both cycles asked for them. Every \"win\" so far is hit-rate-only.\n\n## Open questions\n\n- Do SLRU and cap-free aging add to the adaptive window when stacked? The expectation is yes, mostly on web_0 at the large size, where the climber did nothing.\n- Can a capacity-independent or shorter climber sample period make adaptation useful at large sizes?\n- Does a per-request aging period, or a longer period for small caches, remove the hm_0 small-size aging regression?\n- Should admission ties break with `>=` or randomly, now that halving makes ties common?\n- What do the extra list, region tags, climber sampling and full-table sketch resets cost in metadata and per-op latency, including spikes?\n\n## Per-policy notes\n\n- Fixed-window W-TinyLFU (1%): a solid first fix for first-time-key rejection, but too small at small caches and with a gain that shrinks as capacity grows. Superseded by the adaptive window.\n- Adaptive-window W-TinyLFU (hill climber): the current lineage head. Strong at small sizes on both traces, inert at large sizes because sampling is capacity-tied.\n- SLRU main region: reliably helps web_0, flat on hm_0. This is the next layer to add on the adaptive window.\n- Sketch aging/halving: helps on web_0 only without a counter cap. Its period design is the main open issue, and it is harmful to hm_0 at small sizes under per-increment periods.\n- Trace character: web_0 (scan-heavy) rewards scan resistance and fresh frequency evidence. hm_0 (stable reuse) gains little beyond window sizing and is sensitive to over-aggressive aging.\n"
}
```

### 13:06:01 chat

I've written `cross-cycle-insights.md` in the output directory, rebuilt from scratch from the two-cycle ledger. It has the four required sections in order.

- **Recurring wins:**
  - The gains come from changing TinyLFU's admission contest, not its eviction order.
  - The window in front of the main cache is in every winner, first fixed at 1% and now adaptive.
  - The probation/protected split (SLRU) is the steady fix for web_0's scan-like traffic.
  - Sketch aging helps web_0 once the counter cap is removed.
  - These parts look like they add to each other rather than overlap.
- **Recurring failures:**
  - Capped counters combined with the strict `>` admission rule reject tied newcomers.
  - Periods tied to `capacity` misbehave at one end of the size range.
  - Counting the aging period per sketch increment ages small caches too fast and hurts hm_0.
  - Gains shrink at the large size, especially on hm_0.
  - Metadata-size and per-op-time have still not been measured, so every win so far is hit-rate only.
- **Open questions:**
  - Do SLRU and aging add to the adaptive window when stacked?
  - Can a climber sample period not tied to capacity help at the large size?
  - Does a per-request or small-cache-specific aging period stop the hm_0 small-size regression?
  - Should ties break with `>=` or at random instead of `>`?
  - What do the extra lists, sampling and sketch resets actually cost?
- **Per-policy notes:** grouped by fixed window, adaptive window (the current lineage head), SLRU and sketch aging, plus a note on how web_0 and hm_0 differ.


---

- finished: 2026-10-28T13:06:01+00:00
- duration: 20.66s
- status: ok
- input_tokens: 4
- output_tokens: 1861
- cache_read_tokens: 36123
- cache_write_tokens: 7489
- cost_usd: 0.104373
