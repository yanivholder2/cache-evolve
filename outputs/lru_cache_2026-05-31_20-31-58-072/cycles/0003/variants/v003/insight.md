# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Extending the ghost ring to record Hot evictions and lifting ghost-hit
re-admissions to `referenced = true` was inert on lirs:multi1 at size
512: the dimension landed exactly where the parent did. The likeliest
mechanical reason is that pass 2 almost never fires under this trace —
v002's Cold-only ghost was already capturing every eviction the sweep
actually produced, so widening the recall channel added no new ghost
hits. The pass-2 `referenced` bit tweak is similarly dormant because a
ghost-hit slot is admitted Hot, and pass 1 skips Hot slots regardless of
their ref bit; the bit would only matter on a subsequent sweep where the
slot got demoted, which this workload does not reach. The lesson is that
under multi1/512, the working-set keys CLOCK-Pro-lite loses are lost on
the Cold pass, not on Hot-eviction pass 2 — so further hit-rate gains
have to come from changes that bite on pass 1 (ring sizing, Cold
admission, or demotion stickiness), not from broadening the ghost's
provenance.
