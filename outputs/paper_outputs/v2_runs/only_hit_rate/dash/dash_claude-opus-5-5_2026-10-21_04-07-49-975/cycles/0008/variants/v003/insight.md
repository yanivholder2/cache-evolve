# Variant v003 Insight

> Note: I could not open the overlay source in this session. I had no way to list directories, and guessed source paths and `manifest.toml` did not exist. The mechanism below comes from H03's stated change, which replaces v002's binary ghost-age gate with three tiers: age ≤ 2 goes to stash MRU, 2 < age ≤ 4 goes to the stash midpoint, and anything older or a ghost miss takes the fresh-key/BIP path. I did not check the diff itself.

## Helped

- hit-rate: Ghost returners aged 3–4 epochs now land mid-stash instead of at MRU. They get a shorter protected lifetime, so those that reuse soon still hit, and those that don't age out sooner without pushing ex-residents off the MRU end. The gain is only slightly larger than the hits lost when a few of these keys are evicted before their next reuse.

## Hurt

## Takeaway

Grading placement by ghost age gives only a marginal gain over v002's binary ≤ 4-epoch gate. Most of the value of recent ghost evidence was already captured by v002's gate. This change only redistributes the 3–4 epoch band, which seems to be a small share of ghost hits on this trace. The direction agrees with "recent evidence earns more protection", but the effect is close to single-trace noise at one cache size. Larger placement gains are more likely from changing who reaches the stash at all than from fine-tuning the insertion position inside it.
