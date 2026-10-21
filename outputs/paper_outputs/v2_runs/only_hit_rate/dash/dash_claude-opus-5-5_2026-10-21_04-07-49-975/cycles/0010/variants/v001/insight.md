# Variant v001 Insight

> Note: I could not open the overlay source or the manifest in this session. Only file read and write tools were available, so I could not list directories, and the manifest paths I guessed did not exist. The mechanism below comes from H01's stated change: cut the direct-mapped ghost table from about `capacity / 2` to `capacity / 4` entries, keep the per-entry layout, epoch length and age ≤ 4 gate the same, and re-spread the index over the smaller table. I did not check the diff itself.

## Helped

- hit-rate: Cutting the ghost table to a quarter did not cost hit-rate. It moved slightly up again, which is still within noise and well inside H01's tolerance band. Slots are now overwritten about twice as often, but those overwrites still seem to fall mostly on fingerprints that the 4-epoch gate would have rejected anyway. So the recent ghost evidence that changes placement fits in about a quarter-window of fingerprints.

## Hurt

## Takeaway

H01 predicted that collisions would start costing hits at quarter size. They did not, so v002's half size is not the floor for a direct-mapped ghost on this workload. The useful ghost signal is smaller than a quarter of the cache, and the age ≤ 4 gate is still not the binding limit. Two shrinks in a row have each moved hit-rate up by about the same small amount. That pattern points to noise, or to a slight benefit from evicting stale ghosts, rather than to real lost evidence. Two caveats: the metadata-size saving is still unmeasured, because the fitness vector reports only hit-rate on one LIRS trace at one size; and the collision threshold may appear at other cache sizes or on other traces, so this result does not cover those.
