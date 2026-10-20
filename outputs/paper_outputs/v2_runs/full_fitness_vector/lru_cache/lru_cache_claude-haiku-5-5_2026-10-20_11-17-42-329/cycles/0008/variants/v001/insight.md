# Variant v001 Insight

## Helped

- per-op-time: Both `get` and `put` latencies came down relative to the parent, consistent with the hypothesis routing every index read in `unlink`, `push_back`, `select_victim`, and `demote_prot_head` through one inlined `link_index` helper, so the flag-stripping mask is no longer repeated at each call site; the `get` hit path also sets REFERENCED with a single OR on the raw `prev` word, avoiding an index extraction there.

## Hurt

- (none)

## Takeaway

Keeping the packed flags in `prev` but confining the mask to one inlined helper, with a raw-OR REFERENCED set on hits, recovers the per-op cost the parent paid without giving back the padding saving. The parent's metadata-size and hit-rate were both held, so the change behaves as a policy-neutral latency fix on top of the packed layout. Whether it fully closes the gap to the pre-packing baseline cannot be read from this variant and its parent alone.
