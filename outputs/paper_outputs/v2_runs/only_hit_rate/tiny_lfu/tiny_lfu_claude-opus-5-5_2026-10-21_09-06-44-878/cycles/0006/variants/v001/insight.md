# Variant v001 Insight

## Helped

## Hurt

- hit-rate: The 20-slot LRU window (`window_cap = capacity / 100`) does give every newcomer a short residency before it faces the TinyLFU contest. It cost slightly more than it gained. Window slots come out of the main region (`main_cap = 2028`), and `protected_cap` is now computed from `main_cap`, so it also shrinks. That leaves about 20 fewer frequency-filtered slots, and about 16 fewer of them protected. The only extra hits the window can add are reuses that occur within roughly 20 insertions of a key's arrival. On workload-a, those extra hits did not cover the hits lost from the smaller main region, so the variant ended slightly below the parent.

## Takeaway

A W-TinyLFU window in front of the SLRU main region did not move workload-a off its plateau. This was the case even with a clean, dedup'd sketch and the unchanged strict-`>` contest. The rationale assumed that "short-reuse keys never get a single residency" was costing hit-rate. In practice, few keys on this trace come back within a 20-insertion window, so tie-loss of first-time newcomers is not where the remaining misses are. A 1% window is the cheapest version of this fix and shows nothing to recover. Whether a larger window would help is still untested, because it would take further slots from the frequency-filtered main region.
