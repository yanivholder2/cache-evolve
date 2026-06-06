# Variant v002 Insight

## Helped

- hit-rate: Demoting the coldest node in a 5-wide protected-LRU window (`sample_prot_demotee`) instead of the LRU head unconditionally spares an occasional frequency-hot but recency-stale protected resident from being kicked out on overflow, nudging hit-rate up by a hair over the parent.

## Hurt

- (none observed): hit-rate is the only measured dimension and it did not regress; the extra up-to-five `sketch.estimate` lookups per overflow demotion carry no cost the fitness vector can see.

## Takeaway

Applying the sketch signal to the protected-tier *exit* (which node to demote), having already gated *entry*, is directionally right but nearly inert on workload-a at size 2048: the demotion victim only differs from the LRU head when a colder node sits within five hops of `prot_head`, and on this trace protected recency order tracks sketch frequency closely enough that the window rarely picks a different node. The exit lever is real but its room to move is tiny here.
