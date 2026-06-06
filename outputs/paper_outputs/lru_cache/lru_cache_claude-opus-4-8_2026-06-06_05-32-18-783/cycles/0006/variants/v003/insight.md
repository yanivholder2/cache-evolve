# Variant v003 Insight

## Helped

- hit-rate: Widening `prot_cap` from `capacity * 4 / 5` to `capacity * 7 / 8` in `try_new` lets more single-hit-promoted entries stay resident in the protected segment before demotion, recovering a slice of the hit-rate that the parent's narrower protected share left on the table.

## Hurt

- (none) — per-op-time did not regress despite the wider protected segment; get and put p50 even ticked down, so the feared "geometry-widening starves the fast path" trade did not materialize at this step. metadata-size held at 24 bytes because node layout, links, and the packed `prot` flag were untouched.

## Takeaway

A modest one-eighth bump to the protected soft cap buys real hit-rate above the parent without the latency penalty seen at larger protected shares — the fast path stays cheap because the only change is a sizing fraction, not the promotion, eviction, or demote-on-overflow machinery. The starvation pattern is a function of how aggressively the protected segment is widened, not of widening per se, so there is headroom between the parent's geometry and the point where latency breaks down.
