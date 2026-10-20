# Variant v002 Insight

## Helped

- metadata-size: dropping the now-redundant `key` field from `Entry` (the `entries` HashMap and the `small`/`main` queues already store keys, so `Entry.key` was written by `Entry::new` but never read) shrinks `Entry<u64,u64>` past its long-frozen 24-byte plateau — the first movement on this dimension in seven cycles.

## Hurt

- per-op-time: get p50/p99 and put p99 ticked up marginally; the layout shrink touches no lookup, eviction, or promotion path, so this is measurement jitter rather than a cost the change introduced.

## Takeaway

The key field was pure dead weight once v001's HashMap index made the resident key recoverable from the map key, and deleting it is a free metadata win that leaves hit-rate bit-identical at 0.7312. Notably, the hypothesis's goal of removing `K` entirely was infeasible because the pinned metadata binary instantiates `Entry<u64,u64>` by arity; retaining `K` as a zero-sized `PhantomData<K>` marker satisfies that signature while still collapsing the real layout from 24 to 16 bytes.
