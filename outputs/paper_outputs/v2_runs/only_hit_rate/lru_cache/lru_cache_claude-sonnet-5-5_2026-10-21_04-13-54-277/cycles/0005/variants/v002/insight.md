# Variant v002 Insight

## Helped

- (none) hit-rate did not move relative to the parent; the new `push_front` demotion path produced no measurable gain.

## Hurt

- (none) hit-rate did not regress; the change was effectively neutral on this trace.

## Takeaway

Placing demoted protected nodes at the probation LRU end (via the new `push_front`) instead of the MRU end made no difference to hit-rate. With a 95% protected share, demotions are rare and the demoted entries are seldom re-referenced before eviction either way, so their position in probation does not matter. Probation ordering of demoted keys is not a useful lever here. The added helper is extra code with no payoff, and later work should look elsewhere, such as the admission or promotion policy.
