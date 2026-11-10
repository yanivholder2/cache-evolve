# Variant v003 Insight

## Helped

- hit-rate (hm_0 @ 131072): This is the only cell that rose, and only slightly. At this size the stale ceiling (`main_size / 2`) is wide in absolute terms. The freq-1 returnees it re-routes are the oldest ones, and keeping them out of main protects main residents that hm_0 re-references over long distances. Here, holding them in small for one more reuse costs little.

## Hurt

- hit-rate (web_0 @ 16384): This is the trace the hypothesis was aimed at, and it went slightly down, not up. Late freq-1 returnees on web_0 turn out to be real reuses more often than one-offs. Sending them back to small makes them win a second hit within one small-queue turnover before they can graduate under the main-full freq ≥ 2 rule. The ones that miss that window are ghosted again and lose their next hit.
- hit-rate (web_0 @ 131072): The same small loss shows up at the large size. So the ceiling scaling with `main_size` did reach this cell as intended, but it pushed it the wrong way. The large-size web_0 stall is not caused by stale freq-1 keys flooding main.
- hit-rate (hm_0 @ 16384): A slight dip. Each stale freq-1 returnee re-admitted to small also runs `evict_small` and pushes another small resident out to the ghost. At the small size, that displacement costs more than any main pollution it avoids.

## Takeaway

A freq ≥ 1 ghost entry is a key that was hit in small but could not graduate because main was full (it had freq 1, below the main-full threshold of 2). That reuse is still good evidence late in the ghost window, so an age ceiling on this branch removes useful admissions instead of filtering noise. Because every cell moved so little, freq-1 ghost returnees older than half the window are rare. This branch is therefore a weak lever in either direction. web_0's losses come from the freq-0 / first-admission path, not from freq-1 keys re-entering main. The age signal paid off as a floor on freq-0 returnees (v002) but not as a ceiling on freq-1 returnees: on this branch, ghost age tells the policy little beyond what freq already does.
