# Variant v003 Insight

## Helped

- hit-rate: Replacing `ClassicLRU` with `TimestampLRU` in every bucket decouples promotion candidacy from insertion order — the item with the most recent wall-clock access wins, which more accurately identifies the genuinely hottest items under workload-a's Zipfian distribution and reduces mis-promotion of cold items that happened to land at a favourable vector position.

## Hurt

_(no dimension moved in the wrong direction)_

## Takeaway

The `TimestampLRU` + `InsertPosition::Back` + 75/25 geometry combination replicates and extends the topology gains established in cycle/0004/v002: the larger normal-bucket pool keeps more of the Zipfian hot set in target/probing tiers, while `TimestampLRU` ensures that within each target bucket the item chosen for tier-promotion is the one most frequently accessed rather than the one most recently inserted. The critical difference from a `ClassicLRU + Back` pairing is that a freshly stash-promoted item does not automatically become the promotion candidate — existing bucket residents with more recent access timestamps compete on equal footing, preventing a cold arrival from displacing a genuinely hot incumbent on its very first touch.
