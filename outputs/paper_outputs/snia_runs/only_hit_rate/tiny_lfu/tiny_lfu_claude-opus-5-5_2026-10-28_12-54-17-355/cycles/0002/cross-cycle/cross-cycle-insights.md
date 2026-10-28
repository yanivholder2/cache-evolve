## Recurring wins

- Changing the inputs to TinyLFU's admission contest pays off; changing eviction order alone has not been the lever. Winners so far change who competes (window), who the victim is (probation/SLRU), or how fresh the frequency evidence is (aging).
- A window in front of the main cache is the backbone of every winner: first a fixed small window, then an adaptive (hill-climbed) window built on it. Window sizing matters most at small cache sizes.
- SLRU (probation/protected) has improved web_0 consistently in both cycles. It is the strongest lever for scan-like, one-hit-heavy traffic and is the only component that has helped at web_0's large size.
- Cap-free sketch aging has helped web_0 at both sizes. Removing the counter cap fixed the cycle-1 large-size regression, which confirms that the cap, not aging, was the cause.
- The W-TinyLFU components look complementary rather than overlapping: each helps a different trace/size region, so layering them one at a time is promising.

## Recurring failures

- Saturating counters (cap of 15) combined with strict `>` admission: hot keys tie at the cap and tied newcomers get rejected. This approach is dropped.
- Periods tied to `capacity` (sketch reset interval, climber sample period) misbehave at one end of the size range. Aging fires too early at small sizes, and the climber adapts too slowly at large sizes.
- Aging periods counted per sketch increment over-age small caches, because each miss produces two increments. On hm_0 this halves steady frequent keys before they can build a lead.
- Gains shrink at large sizes, especially on hm_0, where plain recency already captures most of the reuse set. Admission tweaks have little headroom there.
- Metadata-size and per-op-time have never been measured, even though both cycles asked for them. Every "win" so far is hit-rate-only.

## Open questions

- Do SLRU and cap-free aging add to the adaptive window when stacked? The expectation is yes, mostly on web_0 at the large size, where the climber did nothing.
- Can a capacity-independent or shorter climber sample period make adaptation useful at large sizes?
- Does a per-request aging period, or a longer period for small caches, remove the hm_0 small-size aging regression?
- Should admission ties break with `>=` or randomly, now that halving makes ties common?
- What do the extra list, region tags, climber sampling and full-table sketch resets cost in metadata and per-op latency, including spikes?

## Per-policy notes

- Fixed-window W-TinyLFU (1%): a solid first fix for first-time-key rejection, but too small at small caches and with a gain that shrinks as capacity grows. Superseded by the adaptive window.
- Adaptive-window W-TinyLFU (hill climber): the current lineage head. Strong at small sizes on both traces, inert at large sizes because sampling is capacity-tied.
- SLRU main region: reliably helps web_0, flat on hm_0. This is the next layer to add on the adaptive window.
- Sketch aging/halving: helps on web_0 only without a counter cap. Its period design is the main open issue, and it is harmful to hm_0 at small sizes under per-increment periods.
- Trace character: web_0 (scan-heavy) rewards scan resistance and fresh frequency evidence. hm_0 (stable reuse) gains little beyond window sizing and is sensitive to over-aggressive aging.
