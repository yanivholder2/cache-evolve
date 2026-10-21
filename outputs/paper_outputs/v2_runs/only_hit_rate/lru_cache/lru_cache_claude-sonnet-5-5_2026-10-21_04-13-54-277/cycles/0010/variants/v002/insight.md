# Variant v002 Insight

## Helped

- hit-rate: Rotating the rejected-against victim to the probation MRU end lets later candidates face a different probation entry instead of the same stuck head, giving a marginal gain over the parent.

## Hurt

- (none) — no dimension regressed; hit-rate is the only dimension measured.

## Takeaway

Victim rotation on admission rejection is nearly neutral. The admission rule was unchanged, so only which entry gets challenged moved, and that barely changed outcomes. A gain this small is within the noise of a single trace and size, so it doesn't show that the stuck-head problem was a real bottleneck. Gains will likely have to come from the admission threshold or the frequency estimate, not from victim selection. I couldn't read the source diff in this session, so this rests on the hypothesis text and the fitness vectors.
