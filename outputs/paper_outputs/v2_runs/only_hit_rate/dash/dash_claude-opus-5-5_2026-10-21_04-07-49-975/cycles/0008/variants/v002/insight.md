# Variant v002 Insight

> Note: I could not inspect the overlay source in this session. Directory listing was unavailable, and the paths I guessed for the source and `manifest.toml` did not exist. The mechanism below comes from H02's stated change: the ghost-hit recency cutoff in the stash insertion path goes from age ≤ 4 epochs to age ≤ 2 epochs, and nothing else changes. I did not verify the diff itself.

## Helped

- hit-rate: Ghost hits recorded 3–4 epochs back no longer get stash-MRU placement and now take the fresh-key path. This gave a small gain, so on this trace that band of ghost evidence was slightly more often a false "will reuse soon" signal than a true one. The gain is within noise-sized territory for a single trace, though.

## Hurt

## Takeaway

Halving the ghost recency gate from ~0.5× to ~0.25× of the ghost window barely moved hit-rate. The steep gains from shortening the ghost horizon (4× → 1× → gated ~0.5×) have flattened out, and the optimum is now on a plateau around 2–4 epochs, not further down a slope. Most of the benefit came from having an explicit recency gate at all, not from its exact value. So the next useful lever is probably elsewhere, such as shrinking the ghost table to match the ≤ 2-epoch horizon (likely neutral on hit-rate and cheaper) or graded placement by ghost age, not more cutoff tuning. The result comes from one LIRS trace at one cache size.
