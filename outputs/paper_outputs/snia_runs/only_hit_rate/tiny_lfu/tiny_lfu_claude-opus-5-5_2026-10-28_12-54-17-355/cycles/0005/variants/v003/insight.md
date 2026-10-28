# Variant v003 Insight

## Helped

- hit-rate: web_0 improved at both sizes, the clearest gain this lineage has had in a while. The reason is mostly the admission contest, not the protection of recency-driven entries. `sampled_main_victim()` returns the *minimum* estimate of the 4 oldest probation nodes, and `cand_freq > victim_freq` is tested against that minimum. That lowers the bar a window candidate has to clear, so a warm window LRU now displaces a genuinely cold probation entry more often, instead of losing to whatever happened to sit at the probation head. On a frequency-skewed trace like web_0, that is a direct gain in frequency-based selection.
- hit-rate: the direct-evict paths (`win.len < window_cap` and the window-empty branch) now also shed the coldest of the sample. So while the window climber grows the window, main gives up its least-valuable tail entry, not just its oldest one. This adds to the web_0 gain without touching the climbers.

## Hurt

- hit-rate: no cell regressed. The hypothesis's target, hm_0, barely moved at either size, so its stated mechanism is effectively refuted. Newly admitted and newly demoted entries are pushed to the probation *tail* (MRU), and the sample only walks the 4 nodes nearest the *head*. A just-admitted, recency-driven entry is therefore outside the sample until it has aged to the front of probation, and by then it is no longer "just admitted". At these cache sizes probation holds thousands of entries, so a 4-node head window almost never contains the entries the rationale wanted to protect.

## Takeaway

Sampling the probation head for the lowest-frequency victim is a safe change that never hurts. It pays off through the admission contest on frequency-heavy workloads (web_0), not through the recency rescue it was designed for (hm_0). The reason is structural: the sample sits at the LRU end, while fresh admissions sit at the MRU end. hm_0's loss under strict-LRU probation is therefore not about which of the oldest few entries gets evicted. Any future fix for hm_0 has to act on where new entries land or how long they stay in probation, not on which of the oldest few is picked. Raising `VICTIM_SAMPLE` would mainly strengthen the web_0-style frequency effect.
