# Variant v002 Insight

> Note: I could not open the overlay source in this session. I had no way to list directories, guessed source paths did not exist, and `manifest.toml` was not at the variant root. The mechanism below comes from H02's stated change, which adds a coarse eviction epoch to each ghost entry and honours a ghost hit only when the entry is no more than 4 epochs old (about half the ghost window). I did not check the diff itself.

## Helped

- hit-rate: Requiring a ghost hit to be recent (age ≤ 4 epochs) stops stale fingerprints that survived direct-mapped collisions from earning stash-MRU placement. Those far-back returners now go through the normal BIP/probation path. That keeps protected stash capacity for keys whose reuse distance fits the cache, so they stay resident and hit. The gain was larger than the hits lost from long-reuse keys that no longer get the ghost shortcut.

## Hurt

## Takeaway

The ghost table's implicit aging through direct-mapped overwrite was not enough. Old ghost entries were still pulling keys with long reuse distances into MRU, and an explicit recency gate cutting at about half the ghost window gives a measurable hit-rate gain. Cycle 0006 also lost hits from far-back ghost evidence when it enlarged the window. This result points the same way: ghost evidence is worth more when it is recent than when there is more of it. The next lever is the cutoff itself, such as an age ≤ 2 or ≤ 6 epoch threshold, or a smaller table matched to the gated horizon. The result comes from one LIRS trace at one cache size.
