# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Quadrupling the ghost-list window (`GhostList::new(4 * settings.capacity())`) lets a key's first miss stay recorded across roughly four times as many later distinct misses, so a second miss on a long-stale key now admits it and displaces a resident entry. The second-miss gate loses its selectivity, and workload-a regresses against the parent's window.

## Takeaway

The parent's window of one cache capacity is not the limiting factor on workload-a; a longer memory of first misses admits stale recurrences that the shorter window correctly treats as one-shot. The lever from v002 is the recurrence gate itself, and its useful reach is tighter than H03 assumed. The metadata-size dimension the hypothesis asked for was not reported, so the memory cost of the wider window remains unmeasured.
