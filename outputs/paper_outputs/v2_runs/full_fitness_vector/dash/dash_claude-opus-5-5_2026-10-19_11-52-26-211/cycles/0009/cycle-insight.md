# Cycle 0009 Insight

## Cohort outcome

This cycle tested two state-neutral ways to make probes cheaper, plus one hit-rate experiment. v001 (from cycle-7 v002) put each tier's slots into one flat per-segment array and dispatched the policy statically. v002 (from cycle-8 v003) added inline one-byte key tags to the concrete bucket header. Both confirmed their main claim: get p50 moved for the first time in several cycles, and hit-rate stayed exactly the same. v002 was the cleaner win, improving get and put at both median and tail. v001 bought its get-median gain with a clearly worse put path. v003 (wider, fewer stash buckets, from H03) produced no insight, so the stash-geometry question is still untested.

## Patterns

- Cycle 8 was right about the cause: the get-median floor is the dependent load into each bucket's heap item buffer, not hashing and not dispatch at the bucket level. Both winners removed or avoided that load, and both moved p50. The two do it differently. Flattening removes the hop for every probe. Tags skip it for probes that miss, which are most probes, and they help the tail more because full-miss chains touch several buckets.
- Avoiding a load is cheaper than restructuring around it. Tags left the mutation paths almost untouched. Tags also let the "already present?" check before an insert skip the buffer, so put improved too. The flat array made recency ordered by position inside a fixed slice, so every insert, remove, reorder and demotion turned into in-place shifting. That pushed put p50 and especially put p99 up, and nudged get p99 up through move-on-hit.
- Recency ordered by position is now the main mutation cost in any inline layout. Any design that stores slots inline needs recency that doesn't move items: per-slot age/clock bytes or a small index permutation.
- v002's gain is a lower bound. Its tag array is sized for the maximum slot count, not the real capacity, and `max_size` was not shrunk, so the header stride roughly doubled. It improved anyway.
- Put-p99 baselines are still unreliable. One winner's put gain and the other's put loss may both be partly regression toward an earlier level.

## Open questions

- Once tag arrays are sized to real capacity and `max_size` is a `u8`, how much more get latency comes back?
- Do tags and a flat layout stack? For example, tags inline next to a flat slot array, with recency that doesn't move items. Or does the tag filter already capture most of what flattening gives?
- With probes now cheaper, does target-only rescue's per-resident scan cost shrink enough that its hit-rate is worth having?
- Untested: does wider stash associativity (H03) raise hit-rate without rescue cascades?

## Continuation

Make v002 (tagged headers) the new latency parent. First, right-size its tags and header to recover stride. Then re-stack target-only rescue onto it and re-measure the hit-rate vs get-latency trade. Pursue flat/inline storage again only together with recency that doesn't move items, and drop position-shifting recency in fixed slices. Retry the stash-geometry hypothesis, since this cycle produced no result for it, and re-measure the put-p99 baseline before crediting write-path changes.
