# Variant v003 Insight

## Helped

- hit-rate (hm_0 @ 131072): A main victim that was hit during its main residency is now remembered in `mghost`. When it comes back, it goes straight into main with `freq = 1`. Before, it returned as a brand-new key into small, and with main full it needed two spaced reuses (freq ≥ 2) to graduate. That cost it extra misses before it was resident long-term. On hm_0, keys that were hit in main really do get reused again, so skipping that detour through small pays off.
- hit-rate (hm_0 @ 16384): Same mechanism. At the small size, main residency is short, so useful keys get pushed out of main between reuses more often. The `main_hit` gate picks out exactly those keys, and the gain holds up even though `mghost_cap` is tiny here.
- hit-rate (web_0 @ 16384): A smaller gain from the same effect. At this capacity, web_0's reused keys outlive main faster than they outlive their reuse interval, so returning them to main helps a little.

## Hurt

- hit-rate (web_0 @ 131072): This was the target cell, and it moved the wrong way by more than the hypothesis's ±0.001 refutation band. There are two reasons. First, `main_hit` stays set through the key's whole main residency, including every second-chance pass. So a key hit once early in a long residency at this size qualifies even if it is dead by the time it is evicted. Second, each `MainGhost` returnee skips the small filter, forces an `evict_main()` immediately, and then holds its slot for an extra tail sweep because of `freq = 1`. On web_0 the returnees are mostly not reused again, so they push out residents that would have been hit.

## Takeaway

On web_0 at the large size, "was hit at some point in main" is not good evidence of future reuse. The keys that pass the gate are stale, not hot. Readmitting them straight into main with a protective credit turns main's own victims into pollution. 0009-v002's ungated main ghost did nothing, while this gated version actively hurt. That points to the readmission path (skipping small, plus `freq = 1`) as what harms web_0 @ 131072, not the ghost's size. The hm_0 gains show that the same path is useful when main victims really do come back.

Because the web_0 @ 131072 cell moved past the band, the plateau conclusion from the refutation criterion does not apply. The cell responds to how main victims are readmitted, but this attempt pushed it the wrong way. If the idea is revisited, two candidate fixes are: decide `main_hit` by how recent the hit was (clear it on each second-chance decrement), or readmit `MainGhost` returnees with `freq = 0` or into small. Either could keep the hm_0 gain without polluting web_0.
