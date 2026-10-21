# Variant v003 Insight

## Helped

- hit-rate: Inserting at the middle of the bucket and promoting a hit by only one slot (transpose) keeps one-shot keys from displacing proven ones. That scan resistance raised hit rate over the parent.

## Hurt

- per-op-time (get): The hypothesis expected a two-slot swap to make get cheaper. Both median and tail get latency got worse. The hit path was not bottlenecked on element shifting, and the swap adds a data-dependent write.
- per-op-time (put): Median put latency rose. Inserting at index len/2 means shifting the upper half of the bucket on every miss, which is more work than the parent's single-slot overwrite. Put p99 improved slightly, so the effect is mostly in the median.

## Takeaway

Position-based recency (mid-insertion plus transpose) bought hit rate but gave back the cheap hit and put paths of the parent's reference-bit scheme. The result supports the earlier finding that get time is dominated by the lookup, not by reordering. Putting a new entry in the middle of a wide bucket also costs more than it saves. I could not read the overlay sources in this session, so this is based on the hypothesis text and the fitness vectors only.
