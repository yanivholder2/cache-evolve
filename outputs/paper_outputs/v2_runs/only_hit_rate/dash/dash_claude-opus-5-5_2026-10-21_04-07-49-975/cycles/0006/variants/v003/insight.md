# Variant v003 Insight

> Note: I could not list or open the overlay source in this session. I only had file read and write, with no directory listing or diff, and `manifest.toml` was not at the expected path. The mechanism below comes from H03's stated change, which is to raise `BIP_PERIOD` by 4× and change nothing else. I did not check the diff itself.

## Helped

- hit-rate: Making BIP's blind 1-in-`BIP_PERIOD` MRU admission four times rarer means far fewer fresh, unproven keys land in the protected stash MRU region and push out ex-residents and ghost-hit keys that were actually being reused. Now that the ghost table gives returning keys a targeted path to MRU, the random path was mostly admitting one-touch keys. Cutting it freed protected capacity for keys with evidence of reuse, and the extra hits from those keys outweighed the occasional useful key that BIP alone would have caught.

## Hurt

## Takeaway

Once the parent's ghost-hit MRU override existed, BIP's blanket random MRU inserts turned from the only escape route out of probation into a net cost. Protected stash capacity behaves as zero-sum, and evidence-based admission (ghost hits) uses it better than blind sampling. The gain is small, so blind admission was only a modest drain. The next step is to push `BIP_PERIOD` further, or remove BIP entirely and rely on the ghost alone, to see whether the trend continues or BIP still catches reuse beyond the ghost window. This is a single LIRS trace at a single cache size, so the effect may not hold for workloads where long-reuse keys fall outside the ghost window.
