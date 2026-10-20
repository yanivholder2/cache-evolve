# Variant v002 Insight

## Helped

- (none): Balancing admission across both stash homes moved no dimension upward; the one measured axis went the wrong way.

## Hurt

- hit-rate: Choosing the less-full of the two homes at admission (power-of-two-choices) slightly lowered hit-rate versus the parent's always-primary insert. Spreading cold keys evenly across both buckets scatters keys that share a primary home into separate buckets, so a later reference no longer finds its key clustered where the baseline get path expects it — and the diffuse fill defers, rather than prevents, the eventual drop.

## Takeaway

Pre-emptively load-balancing stash admission is not equivalent to rescuing overflow victims: the parent's reactive one-hop relocation preserved primary-home clustering and only displaced victims that genuinely collided, whereas moving the balance decision to admission time traded that clustering away for evenness and cost a sliver of hit-rate. The collision-pressure framing was wrong — even fill does not raise cold-key dwell time here, it just relocates where keys sit, so the get path's primary-first locality matters more than balanced occupancy.
